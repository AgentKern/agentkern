/**
 * AgentKern Identity - Synapse Service
 *
 * Wrapper for N-API bridge to Rust Synapse logic.
 * Provides agent state management operations: get/update state.
 *
 * Per DECISION_RECORD_BRIDGE.md: N-API for hot path (0ms latency)
 */

import { Injectable, Logger, OnModuleInit } from '@nestjs/common';
import * as path from 'path';
import * as fs from 'fs';

// Type definitions for bridge responses
export interface AgentState {
  agent_id: string;
  state: Record<string, unknown>;
  version: number;
}

export interface StateUpdateResult {
  success: boolean;
  version?: number;
  error?: string;
}

export interface SimilarityResult {
  node_id: string;
  score: number;
}

// Bridge interface (loaded from native module)
interface NativeBridge {
  synapseGetState(agentId: string): Promise<string>;
  synapseUpdateState(agentId: string, stateJson: string): Promise<string>;
  synapseStoreMemory(agentId: string, text: string): Promise<string>;
  synapseQueryMemory(text: string, limit: number): Promise<string>;
}

@Injectable()
export class SynapseService implements OnModuleInit {
  private readonly logger = new Logger(SynapseService.name);
  private bridge!: NativeBridge;
  private bridgeLoaded = false;

  async onModuleInit(): Promise<void> {
    await Promise.resolve();

    const isProduction = process.env.NODE_ENV === 'production';
    const bridgePath = this.resolveBridgePath();

    try {
      // Verify bridge file exists
      if (!fs.existsSync(bridgePath)) {
        throw new Error(
          `Bridge file not found at: ${bridgePath}. Run: cd packages/foundation/bridge && pnpm build`,
        );
      }

      // Load bridge
      // eslint-disable-next-line @typescript-eslint/no-require-imports
      this.bridge = require(bridgePath) as NativeBridge;
      this.bridgeLoaded = true;
      this.logger.log('🧠 Synapse N-API Bridge loaded successfully');

      // Verify bridge is operational
      await this.verifyBridge();
    } catch (error: unknown) {
      const errorMessage =
        error instanceof Error ? error.message : String(error);

      if (isProduction) {
        this.logger.error(
          `🚨 CRITICAL: Failed to load N-API bridge in production: ${errorMessage}`,
        );
        throw new Error(
          `N-API bridge is required in production but failed to load: ${errorMessage}`,
        );
      } else {
        this.logger.error(
          `🚨 Failed to load Synapse N-API bridge: ${errorMessage}`,
        );
        this.logger.warn(
          '⚠️ DEPRECATED: SynapseService operating in degraded mode. See EPISTEMIC_HEALTH.md',
        );
        this.logger.warn(
          '⚠️ To fix: cd packages/foundation/bridge && pnpm build',
        );
      }
    }
  }

  /**
   * Resolve bridge path with proper error handling
   */
  private resolveBridgePath(): string {
    const possiblePaths = [
      path.resolve(
        __dirname,
        '../../../../packages/foundation/bridge/index.node',
      ),
      path.resolve(
        __dirname,
        '../../../packages/foundation/bridge/index.node',
      ),
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
   * Verify bridge is operational
   */
  private async verifyBridge(): Promise<void> {
    try {
      // Test with a simple call
      const testResult = await this.bridge.synapseGetState('test-verify');
      if (!testResult) {
        throw new Error('Bridge returned null for test call');
      }
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
   * Get agent state
   * 
   * @deprecated When bridge is not loaded, this returns null.
   * This is a temporary fallback. The Rust Synapse implementation should always be available in production.
   * See EPISTEMIC_HEALTH.md for architectural integration status.
   */
  async getState(agentId: string): Promise<AgentState | null> {
    if (!this.bridgeLoaded) {
      this.logger.warn(
        '⚠️ DEPRECATED: Bridge not loaded, SynapseService operating in degraded mode. See EPISTEMIC_HEALTH.md',
      );
      return null;
    }

    try {
      const result = await this.bridge.synapseGetState(agentId);
      return JSON.parse(result) as AgentState;
    } catch (error) {
      this.logger.error(`Failed to get state for ${agentId}: ${error}`);
      return null;
    }
  }

  /**
   * Update agent state (CRDT-based merge)
   */
  async updateState(
    agentId: string,
    updates: Record<string, unknown>,
  ): Promise<StateUpdateResult> {
    if (!this.bridgeLoaded) {
      return { success: false, error: 'Bridge not loaded' };
    }

    try {
      const stateJson = JSON.stringify(updates);
      const result = await this.bridge.synapseUpdateState(agentId, stateJson);
      const parsed = JSON.parse(result) as { error?: string; version?: number };

      if (parsed.error) {
        return { success: false, error: parsed.error };
      }

      return { success: true, version: parsed.version };
    } catch (error) {
      this.logger.error(`Failed to update state for ${agentId}: ${error}`);
      return { success: false, error: String(error) };
    }
  }

  /**
   * Delete specific keys from agent state
   */
  async deleteKeys(
    agentId: string,
    keys: string[],
  ): Promise<StateUpdateResult> {
    if (!this.bridgeLoaded) {
      return { success: false, error: 'Bridge not loaded' };
    }

    // Use update with null values to signal deletion
    const deletions: Record<string, null> = {};
    for (const key of keys) {
      deletions[key] = null;
    }

    return this.updateState(agentId, deletions);
  }

  /**
   * Store agent memory (embed + vector store)
   */
  async storeMemory(
    agentId: string,
    text: string,
  ): Promise<{ id?: string; error?: string }> {
    if (!this.bridgeLoaded) {
      return { error: 'Bridge not loaded' };
    }

    try {
      const result = await this.bridge.synapseStoreMemory(agentId, text);
      const parsed = JSON.parse(result) as { error?: string; id?: string };
      if (parsed.error) {
        return { error: parsed.error };
      }
      if (!parsed.id) {
        return { error: 'Memory storage failed: no ID returned' };
      }
      return { id: parsed.id };
    } catch (error: unknown) {
      const errorMessage =
        error instanceof Error ? error.message : String(error);
      this.logger.error(
        `Failed to store memory for ${agentId}: ${errorMessage}`,
      );
      return { error: errorMessage };
    }
  }

  /**
   * Query similar memories
   */
  async queryMemory(
    text: string,
    limit: number = 5,
  ): Promise<SimilarityResult[]> {
    if (!this.bridgeLoaded) {
      this.logger.warn('Bridge not loaded, returning empty memory query');
      return [];
    }

    try {
      const result = await this.bridge.synapseQueryMemory(text, limit);
      // Rust returns Vec<SimilarityResult>, which serializes to array
      const parsed = JSON.parse(result) as
        | SimilarityResult[]
        | { error?: string };
      if (Array.isArray(parsed)) {
        return parsed;
      }
      if (parsed.error) {
        this.logger.error(`Memory query failed: ${parsed.error}`);
      }
      return [];
    } catch (error) {
      this.logger.error(`Failed to query memory: ${error}`);
      return [];
    }
  }
}
