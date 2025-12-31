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
  verify(agentId: string, action: string, contextJson?: string): Promise<string>;
}

@Injectable()
export class GateService implements OnModuleInit {
  private readonly logger = new Logger(GateService.name);
  private bridge!: NativeBridge;
  private bridgeLoaded = false;

  async onModuleInit(): Promise<void> {
    try {
      // Path to native module (relative to apps/identity)
      const bridgePath = path.resolve(__dirname, '../../../../packages/bridge/index.node');
      this.bridge = require(bridgePath) as NativeBridge;
      this.bridgeLoaded = true;
      this.logger.log('🌉 N-API Bridge loaded successfully');
    } catch (error) {
      this.logger.error(`Failed to load N-API bridge: ${error}`);
      this.logger.warn('⚠️ Operating in DEGRADED mode (no native prompt guard)');
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
   */
  guardPrompt(prompt: string): PromptAnalysis | null {
    if (!this.bridgeLoaded) {
      this.logger.warn('Bridge not loaded, skipping prompt guard');
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
   */
  shouldBlockPrompt(prompt: string): boolean {
    const analysis = this.guardPrompt(prompt);
    if (!analysis) return false; // Degraded mode: allow
    return analysis.threat_level === 'High' || analysis.threat_level === 'Critical';
  }
}
