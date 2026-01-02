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
import { GatePolicyRepository } from '../repositories/gate-policy.repository';
import { GatePolicyEntity } from '../entities/gate-policy.entity';

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
}

@Injectable()
export class GateService implements OnModuleInit {
  private readonly logger = new Logger(GateService.name);
  private bridge!: NativeBridge;
  private bridgeLoaded = false;

  constructor(private readonly policyRepository: GatePolicyRepository) {}

  async onModuleInit(): Promise<void> {
    await Promise.resolve(); // Ensure async lifecycle hook
    try {
      // Path to native module (relative to apps/identity/dist)
      // Correct path: packages/foundation/bridge/index.node
      const bridgePath = path.resolve(
        __dirname,
        '../../../../packages/foundation/bridge/index.node',
      );
      // eslint-disable-next-line @typescript-eslint/no-require-imports
      this.bridge = require(bridgePath) as NativeBridge;
      this.bridgeLoaded = true;
      this.logger.log('🌉 N-API Bridge loaded successfully');
    } catch (error) {
      // CRITICAL: Log as ERROR, not WARN - this is a security degradation
      this.logger.error(
        `🚨 SECURITY DEGRADATION: Failed to load N-API bridge: ${error}`,
      );
      this.logger.error(
        '🚨 GateService will operate in FAIL-CLOSED mode (blocking all prompts)',
      );
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
   * Returns null ONLY on parse errors; bridge unavailability is handled by shouldBlockPrompt()
   */
  guardPrompt(prompt: string): PromptAnalysis | null {
    if (!this.bridgeLoaded) {
      // FAIL-CLOSED: Return null to signal security check unavailable
      // Callers MUST handle null as "block" for security-critical paths
      this.logger.error(
        'SECURITY: Bridge not loaded, prompt guard unavailable',
      );
      return null;
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
    return this.policyToResponse(entity);
  }

  /**
   * Check PCI-DSS compliance (stub)
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
    const issues: Array<{
      code: string;
      severity: 'info' | 'warning' | 'error' | 'critical';
      message: string;
      path?: string;
    }> = [];

    // Check for unencrypted card numbers
    const stringData = JSON.stringify(data);
    if (/\b\d{13,19}\b/.test(stringData)) {
      issues.push({
        code: 'PCI-DSS-3.4',
        severity: 'critical',
        message: 'Potential unencrypted PAN detected',
        path: 'data',
      });
    }

    return {
      compliant: issues.length === 0,
      standard: 'PCI-DSS v4.0',
      issues,
      checkedAt: new Date().toISOString(),
    };
  }

  /**
   * Check HIPAA compliance (stub)
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
    const issues: Array<{
      code: string;
      severity: 'info' | 'warning' | 'error' | 'critical';
      message: string;
      path?: string;
    }> = [];

    // Check for PHI fields without encryption
    const phiFields = ['ssn', 'medical_record', 'health_plan', 'diagnosis'];
    for (const field of phiFields) {
      if (field in data) {
        issues.push({
          code: 'HIPAA-164.312',
          severity: 'warning',
          message: `PHI field '${field}' detected - ensure encryption at rest`,
          path: field,
        });
      }
    }

    return {
      compliant: issues.filter((i) => i.severity === 'critical').length === 0,
      standard: 'HIPAA Privacy Rule',
      issues,
      checkedAt: new Date().toISOString(),
    };
  }

  /**
   * Check GDPR compliance (stub)
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
    const issues: Array<{
      code: string;
      severity: 'info' | 'warning' | 'error' | 'critical';
      message: string;
      path?: string;
    }> = [];

    // Check for consent
    if (!('consent' in data) && !('gdpr_consent' in data)) {
      issues.push({
        code: 'GDPR-Art6',
        severity: 'warning',
        message: 'No consent field found - ensure lawful basis for processing',
      });
    }

    return {
      compliant: issues.filter((i) => i.severity === 'critical').length === 0,
      standard: 'GDPR',
      issues,
      checkedAt: new Date().toISOString(),
    };
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
