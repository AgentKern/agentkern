/**
 * AgentKernIdentity - Gate Service
 *
 * Wrapper for N-API bridge to Rust Gate logic.
 * Provides hot-path operations: prompt guard, context guard, policy verification.
 *
 * Per DECISION_RECORD_BRIDGE.md: N-API for hot path (0ms latency)
 */

import { Injectable, Logger, OnModuleInit } from '@nestjs/common';
import * as path from 'path';
import * as fs from 'fs';
import { GatePolicyRepository } from '../repositories/gate-policy.repository';
import { GatePolicyEntity } from '../entities/gate-policy.entity';
import { ComplianceEngine } from './compliance.engine';

// Type definitions for bridge responses
export interface PromptAnalysis {
  threat_level: 'None' | 'Low' | 'Medium' | 'High' | 'Critical';
  attacks: string[];
  matched_patterns: string[];
  confidence: number;
  action: 'Allow' | 'AllowWithLog' | 'Review' | 'Block' | 'BlockAndAlert';
  latency_us: number;
}

export interface ContextScanResult {
  safe: boolean;
  injections_found: number;
  suspicious_chunks: number[];
  latency_us: number;
}

export interface Attestation {
  platform: string;
  quote: number[];
  measurement: number[];
  user_data: number[];
  timestamp: number;
  cert_chain: string[];
}

export interface VerificationResult {
  request_id: string;
  allowed: boolean;
  evaluated_policies: string[];
  blocking_policies: string[];
  symbolic_risk_score: number;
  neural_risk_score?: number;
  final_risk_score: number;
  reasoning: string;
  latency: {
    total_us: number;
    symbolic_us: number;
    neural_us?: number;
  };
}

// Bridge interface (loaded from native module)
interface NativeBridge {
  attest(nonce: string): string;
  guardPrompt(prompt: string): string;
  guardContext(chunks: string[]): string;
  verify(
    agentId: string,
    action: string,
    contextJson?: string,
  ): Promise<string>;
  registerPolicy(policyYaml: string): Promise<string>;
}

@Injectable()
export class GateService implements OnModuleInit {
  private readonly logger = new Logger(GateService.name);
  private bridge!: NativeBridge;
  private bridgeLoaded = false;

  constructor(private readonly policyRepository: GatePolicyRepository) {}

  async onModuleInit(): Promise<void> {
    await Promise.resolve(); // Ensure async lifecycle hook

    const isProduction = process.env.NODE_ENV === 'production';
    const bridgePath = this.resolveBridgePath();

    try {
      // Verify bridge file exists before attempting to load
      if (!fs.existsSync(bridgePath)) {
        throw new Error(
          `Bridge file not found at: ${bridgePath}. Run: cd packages/foundation/bridge && pnpm build`,
        );
      }

      // Load bridge
      // eslint-disable-next-line @typescript-eslint/no-require-imports
      this.bridge = require(bridgePath) as NativeBridge;
      this.bridgeLoaded = true;
      this.logger.log('🌉 N-API Bridge loaded successfully');

      // Verify bridge is operational
      await this.verifyBridge();

      // Initial policy sync
      this.syncPolicies().catch((e) =>
        this.logger.error(`Initial policy sync failed: ${e}`),
      );
    } catch (error: unknown) {
      const errorMessage =
        error instanceof Error ? error.message : String(error);

      if (isProduction) {
        // PRODUCTION: Fail-fast - bridge is mandatory
        this.logger.error(
          `🚨 CRITICAL: Failed to load N-API bridge in production: ${errorMessage}`,
        );
        this.logger.error(
          '🚨 Application cannot start without operational bridge',
        );
        throw new Error(
          `N-API bridge is required in production but failed to load: ${errorMessage}`,
        );
      } else {
        // DEVELOPMENT: Allow degraded mode with warnings
        this.logger.error(
          `🚨 SECURITY DEGRADATION: Failed to load N-API bridge: ${errorMessage}`,
        );
        this.logger.warn(
          '⚠️ GateService will operate in FAIL-CLOSED mode (blocking all prompts)',
        );
        this.logger.warn(
          '⚠️ To fix: cd packages/foundation/bridge && pnpm build',
        );
      }
    }
  }

  /**
   * Resolve bridge path with proper error handling
   * Tries multiple possible locations (development vs production)
   */
  private resolveBridgePath(): string {
    const possiblePaths = [
      // Development: from source
      path.resolve(
        __dirname,
        '../../../../packages/foundation/bridge/index.node',
      ),
      // Production: from dist (after build)
      path.resolve(
        __dirname,
        '../../../packages/foundation/bridge/index.node',
      ),
      // Docker/container: absolute path
      '/app/packages/foundation/bridge/index.node',
    ];

    for (const testPath of possiblePaths) {
      if (fs.existsSync(testPath)) {
        return testPath;
      }
    }

    throw new Error(
      `Bridge not found in any expected location: ${possiblePaths.join(', ')}`,
    );
  }

  /**
   * Verify bridge is operational by calling a test function
   */
  private async verifyBridge(): Promise<void> {
    try {
      // Test with a simple call that should always work
      const testResult = this.bridge.guardPrompt('test');
      if (!testResult) {
        throw new Error('Bridge returned null for test call');
      }
      // Verify it's valid JSON
      JSON.parse(testResult);
      this.logger.log('✅ Bridge verification successful');
    } catch (error: unknown) {
      const errorMessage =
        error instanceof Error ? error.message : String(error);
      throw new Error(`Bridge verification failed: ${errorMessage}`);
    }
  }

  /**
   * Check if bridge is operational
   */
  isOperational(): boolean {
    return this.bridgeLoaded;
  }

  /**
   * Guard prompt against injection attacks (0ms latency)
   * 
   * Production-ready: Fails-fast if bridge unavailable in production.
   * Development: Returns null for graceful degradation.
   * 
   * @throws Error in production if bridge is not loaded
   * @returns PromptAnalysis or null (development only)
   */
  guardPrompt(prompt: string): PromptAnalysis | null {
    if (!this.bridgeLoaded) {
      const isProduction = process.env.NODE_ENV === 'production';
      
      if (isProduction) {
        // PRODUCTION: Fail-fast - security cannot be compromised
        this.logger.error(
          '🚨 CRITICAL: Prompt guard unavailable - bridge not loaded in production',
        );
        throw new Error(
          'N-API bridge is required for prompt guard in production but is not loaded',
        );
      } else {
        // DEVELOPMENT: Allow degraded mode with warnings
        this.logger.error(
          '🚨 SECURITY DEGRADATION: Bridge not loaded, prompt guard unavailable',
        );
        this.logger.warn(
          '⚠️ DEPRECATED: GateService operating without Rust bridge. See EPISTEMIC_HEALTH.md',
        );
        this.logger.warn(
          '⚠️ To fix: cd packages/foundation/bridge && pnpm build',
        );
        return null;
      }
    }

    try {
      const result = this.bridge.guardPrompt(prompt);
      return JSON.parse(result) as PromptAnalysis;
    } catch (error) {
      this.logger.error(`Prompt guard failed: ${error}`);
      return null;
    }
  }

  /**
   * Guard RAG context chunks against injection (0ms latency)
   */
  guardContext(chunks: string[]): ContextScanResult | null {
    if (!this.bridgeLoaded) {
      this.logger.warn('Bridge not loaded, skipping context guard');
      return null;
    }

    try {
      const result = this.bridge.guardContext(chunks);
      return JSON.parse(result) as ContextScanResult;
    } catch (error) {
      this.logger.error(`Context guard failed: ${error}`);
      return null;
    }
  }

  /**
   * Generate TEE attestation
   */
  attest(nonce: string): Attestation | null {
    if (!this.bridgeLoaded) {
      this.logger.warn('Bridge not loaded, skipping attestation');
      return null;
    }

    try {
      const result = this.bridge.attest(nonce);
      return JSON.parse(result) as Attestation;
    } catch (error) {
      this.logger.error(`Attestation failed: ${error}`);
      return null;
    }
  }

  /**
   * Verify agent action against policies
   */
  async verify(
    agentId: string,
    action: string,
    context?: Record<string, unknown>,
  ): Promise<VerificationResult | null> {
    if (!this.bridgeLoaded) {
      this.logger.warn('Bridge not loaded, skipping policy verification');
      return null;
    }

    try {
      const contextJson = context ? JSON.stringify(context) : undefined;
      const result = await this.bridge.verify(agentId, action, contextJson);
      return JSON.parse(result) as VerificationResult;
    } catch (error) {
      this.logger.error(`Policy verification failed: ${error}`);
      return null;
    }
  }

  /**
   * Quick check if a prompt should be blocked
   *
   * SECURITY: Implements FAIL-CLOSED pattern
   * If bridge is unavailable or analysis fails, returns TRUE (block)
   */
  shouldBlockPrompt(prompt: string): boolean {
    const analysis = this.guardPrompt(prompt);
    if (!analysis) {
      // FAIL-CLOSED: Block if security check unavailable
      this.logger.error(
        'SECURITY: Blocking prompt due to unavailable security check (fail-closed)',
      );
      return true;
    }
    return (
      analysis.threat_level === 'High' || analysis.threat_level === 'Critical'
    );
  }

  // =========================================================================
  // HTTP API Methods (for GateController)
  // =========================================================================

  /**
   * Analyze prompt for injection attacks (HTTP API version)
   */
  async analyzePrompt(prompt: string): Promise<{
    safe: boolean;
    threatLevel: 'none' | 'low' | 'medium' | 'high' | 'critical';
    threatType?: string;
    attacks: string[];
    matchedPatterns: string[];
    latencyUs: number;
    score: number;
    reason?: string;
  }> {
    await Promise.resolve(); // Ensure async execution
    const analysis = this.guardPrompt(prompt);

    if (!analysis) {
      // FAIL-CLOSED: Return unsafe if bridge unavailable
      return {
        safe: false,
        threatLevel: 'critical',
        threatType: 'security_unavailable',
        score: 100,
        reason: 'Security check unavailable - fail-closed mode',
        attacks: [],
        matchedPatterns: [],
        latencyUs: 0,
      };
    }

    const threatLevelMap: Record<
      string,
      'none' | 'low' | 'medium' | 'high' | 'critical'
    > = {
      None: 'none',
      Low: 'low',
      Medium: 'medium',
      High: 'high',
      Critical: 'critical',
    };

    return {
      safe: analysis.threat_level === 'None' || analysis.threat_level === 'Low',
      threatLevel: threatLevelMap[analysis.threat_level] || 'medium',
      threatType: analysis.attacks.length > 0 ? analysis.attacks[0] : undefined,
      score: Math.round(analysis.confidence * 100),
      reason: analysis.matched_patterns.join(', ') || undefined,
      attacks: analysis.attacks || [],
      matchedPatterns: analysis.matched_patterns || [],
      latencyUs: 0,
    };
  }

  /**
   * Convert entity to API response format
   */
  private policyToResponse(entity: GatePolicyEntity): {
    id: string;
    name: string;
    description?: string;
    active: boolean;
    rules: Array<{
      id: string;
      condition: string;
      action: 'allow' | 'deny' | 'audit' | 'escalate';
      priority?: number;
    }>;
    createdAt: string;
    updatedAt?: string;
  } {
    return {
      id: entity.id,
      name: entity.name,
      description: entity.description ?? undefined,
      active: entity.active,
      rules: entity.rules,
      createdAt: entity.createdAt.toISOString(),
      updatedAt: entity.updatedAt?.toISOString(),
    };
  }

  /**
   * List all policies (from database)
   */
  async listPolicies(): Promise<
    Array<{
      id: string;
      name: string;
      description?: string;
      active: boolean;
      rules: Array<{
        id: string;
        condition: string;
        action: 'allow' | 'deny' | 'audit' | 'escalate';
        priority?: number;
      }>;
      createdAt: string;
      updatedAt?: string;
    }>
  > {
    const entities = await this.policyRepository.findAll();
    return entities.map((e) => this.policyToResponse(e));
  }

  /**
   * Get policy by ID (from database)
   */
  async getPolicy(id: string): Promise<{
    id: string;
    name: string;
    description?: string;
    active: boolean;
    rules: Array<{
      id: string;
      condition: string;
      action: 'allow' | 'deny' | 'audit' | 'escalate';
      priority?: number;
    }>;
    createdAt: string;
    updatedAt?: string;
  }> {
    const entity = await this.policyRepository.findById(id);
    if (!entity) {
      throw new Error(`Policy ${id} not found`);
    }
    return this.policyToResponse(entity);
  }

  /**
   * Create policy (persisted to database)
   */
  async createPolicy(dto: {
    name: string;
    description?: string;
    rules: Array<{
      id: string;
      condition: string;
      action: 'allow' | 'deny' | 'audit' | 'escalate';
      priority?: number;
    }>;
  }): Promise<{
    id: string;
    name: string;
    description?: string;
    active: boolean;
    rules: Array<{
      id: string;
      condition: string;
      action: 'allow' | 'deny' | 'audit' | 'escalate';
      priority?: number;
    }>;
    createdAt: string;
  }> {
    const entity = await this.policyRepository.create({
      name: dto.name,
      description: dto.description,
      rules: dto.rules,
    });

    // Sync to Bridge
    this.syncPolicies().catch((e) =>
      this.logger.error(`Policy sync failed after create: ${e}`),
    );

    return this.policyToResponse(entity);
  }

  /**
   * Sync all policies from DB to Rule Engine
   */
  async syncPolicies(): Promise<void> {
    if (!this.bridgeLoaded) return;

    const policies = await this.policyRepository.findAll();
    this.logger.log(`Syncing ${policies.length} policies to Gate Engine...`);

    for (const p of policies) {
      // Convert to Rust Policy structure (JSON is valid YAML)
      const rustPolicy = {
        id: p.id,
        name: p.name,
        description: p.description || '',
        priority: 100, // Default priority
        enabled: p.active,
        jurisdictions: [], // Empty = Global
        rules: p.rules.map((r) => ({
          id: r.id,
          condition: r.condition,
          action: r.action,
          // Map metadata to Rust fields if available
          message: (r.metadata?.message as string) || undefined,
          risk_score: (r.metadata?.risk_score as number) || undefined,
        })),
      };

      try {
        const yaml = JSON.stringify(rustPolicy);
        const result = await this.bridge.registerPolicy(yaml);
        const parsed = JSON.parse(result) as {
          error?: string;
          success?: boolean;
        };
        if (parsed.error) {
          this.logger.error(
            `Failed to register policy ${p.id}: ${parsed.error}`,
          );
        }
      } catch (e) {
        this.logger.error(`Bridge error syncing policy ${p.id}: ${e}`);
      }
    }
  }

  /**
   */
  async checkPciCompliance(data: Record<string, unknown>): Promise<{
    compliant: boolean;
    standard: string;
    issues: Array<{
      code: string;
      severity: 'info' | 'warning' | 'error' | 'critical';
      message: string;
      path?: string;
    }>;
    checkedAt: string;
  }> {
    await Promise.resolve(); // Ensure async execution
    return ComplianceEngine.checkPciDss(data);
  }

  /**
   */
  async checkHipaaCompliance(data: Record<string, unknown>): Promise<{
    compliant: boolean;
    standard: string;
    issues: Array<{
      code: string;
      severity: 'info' | 'warning' | 'error' | 'critical';
      message: string;
      path?: string;
    }>;
    checkedAt: string;
  }> {
    await Promise.resolve(); // Ensure async execution
    return ComplianceEngine.checkHipaa(data);
  }

  /**
   */
  async checkGdprCompliance(data: Record<string, unknown>): Promise<{
    compliant: boolean;
    standard: string;
    issues: Array<{
      code: string;
      severity: 'info' | 'warning' | 'error' | 'critical';
      message: string;
      path?: string;
    }>;
    checkedAt: string;
  }> {
    await Promise.resolve(); // Ensure async execution
    return ComplianceEngine.checkGdpr(data);
  }

  /**
   * List WASM actors (stub)
   */
  async listWasmActors(): Promise<
    Array<{
      name: string;
      version: string;
      capabilities: Array<{
        name: string;
        inputSchema?: Record<string, unknown>;
        outputSchema?: Record<string, unknown>;
      }>;
      sizeBytes: number;
      loadedAt: string;
      invocations: number;
      avgLatencyUs: number;
    }>
  > {
    await Promise.resolve(); // Ensure async execution
    return [
      {
        name: 'prompt-guard',
        version: '1.0.0',
        capabilities: [
          {
            name: 'prompt_guard',
            inputSchema: {
              type: 'object',
              properties: { prompt: { type: 'string' } },
            },
          },
        ],
        sizeBytes: 245760,
        loadedAt: new Date().toISOString(),
        invocations: 0,
        avgLatencyUs: 50,
      },
    ];
  }

  /**
   * Get WASM actor by name (stub)
   */
  async getWasmActor(name: string): Promise<{
    name: string;
    version: string;
    capabilities: Array<{
      name: string;
      inputSchema?: Record<string, unknown>;
      outputSchema?: Record<string, unknown>;
    }>;
    sizeBytes: number;
    loadedAt: string;
    invocations: number;
    avgLatencyUs: number;
  }> {
    const actors = await this.listWasmActors();
    const actor = actors.find((a) => a.name === name);
    if (!actor) {
      throw new Error(`WASM actor ${name} not found`);
    }
    return actor;
  }

  /**
   * Register WASM actor (stub)
   */
  async registerWasmActor(dto: {
    name: string;
    version: string;
    wasmBase64: string;
    capabilities: Array<{
      name: string;
      inputSchema?: Record<string, unknown>;
      outputSchema?: Record<string, unknown>;
    }>;
  }): Promise<{
    name: string;
    version: string;
    capabilities: Array<{
      name: string;
      inputSchema?: Record<string, unknown>;
      outputSchema?: Record<string, unknown>;
    }>;
    sizeBytes: number;
    loadedAt: string;
    invocations: number;
    avgLatencyUs: number;
  }> {
    await Promise.resolve(); // Ensure async execution
    this.logger.log(`Registering WASM actor: ${dto.name} v${dto.version}`);

    return {
      name: dto.name,
      version: dto.version,
      capabilities: dto.capabilities,
      sizeBytes: Buffer.from(dto.wasmBase64, 'base64').length,
      loadedAt: new Date().toISOString(),
      invocations: 0,
      avgLatencyUs: 0,
    };
  }
}
