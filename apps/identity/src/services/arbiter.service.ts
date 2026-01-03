/**
 * AgentKern Identity - Arbiter Service
 *
 * Wrapper for N-API bridge to Rust Arbiter logic.
 * Provides governance operations: kill switch, audit logs, chaos testing.
 *
 * Per DECISION_RECORD_BRIDGE.md: N-API for hot path (0ms latency)
 */

import { Injectable, Logger, OnModuleInit } from '@nestjs/common';
import * as path from 'path';
import * as fs from 'fs';

// Type definitions for bridge responses
export interface KillSwitchStatus {
  active: boolean;
  terminated_count: number;
}

export interface KillRecord {
  id: string;
  timestamp: string;
  target_id: string;
  target_type: 'Agent' | 'Swarm' | 'Region' | 'Global';
  reason: string;
  termination_type: 'Graceful' | 'Forced' | 'HardwareKill';
  initiated_by?: string;
  success: boolean;
  error?: string;
}

export interface AuditStatistics {
  total_records: number;
  approved_count: number;
  denied_count: number;
  review_count: number;
  logged_count: number;
  high_risk_count: number;
  avg_risk_score: number;
}

export interface ChaosStats {
  total_ops: number;
  latency_injections: number;
  error_injections: number;
}

// Bridge interface (loaded from native module)
interface NativeBridge {
  arbiterKillSwitchActivate(reason: string, agentId?: string): Promise<string>;
  arbiterKillSwitchStatus(): Promise<string>;
  arbiterKillSwitchDeactivate(): Promise<string>;
  arbiterQueryAudit(): Promise<string>;
  arbiterChaosStats(): string;
}

@Injectable()
export class ArbiterService implements OnModuleInit {
  private readonly logger = new Logger(ArbiterService.name);
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
      this.logger.log('⚖️ Arbiter N-API Bridge loaded successfully');

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
          `🚨 Failed to load Arbiter N-API bridge: ${errorMessage}`,
        );
        this.logger.warn(
          '⚠️ DEPRECATED: ArbiterService operating in degraded mode. See EPISTEMIC_HEALTH.md',
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
      const testResult = await this.bridge.arbiterKillSwitchStatus();
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

  // =========================================================================
  // Kill Switch Operations
  // =========================================================================

  /**
   * Activate kill switch - terminate specific agent or all agents
   */
  /**
   * @deprecated When bridge is not loaded, this returns an error.
   * This is a temporary fallback. The Rust Arbiter implementation should always be available in production.
   * See EPISTEMIC_HEALTH.md for architectural integration status.
   */
  async activateKillSwitch(
    reason: string,
    agentId?: string,
  ): Promise<KillRecord | { error: string }> {
    if (!this.bridgeLoaded) {
      this.logger.warn(
        '⚠️ DEPRECATED: Bridge not loaded, ArbiterService operating in degraded mode. See EPISTEMIC_HEALTH.md',
      );
      return { error: 'Bridge not loaded - Arbiter functionality unavailable' };
    }

    try {
      const result = await this.bridge.arbiterKillSwitchActivate(
        reason,
        agentId,
      );
      return JSON.parse(result) as KillRecord;
    } catch (error) {
      this.logger.error(`Failed to activate kill switch: ${error}`);
      return { error: String(error) };
    }
  }

  /**
   * Get kill switch status
   */
  async getKillSwitchStatus(): Promise<KillSwitchStatus> {
    if (!this.bridgeLoaded) {
      return { active: false, terminated_count: 0 };
    }

    try {
      const result = await this.bridge.arbiterKillSwitchStatus();
      return JSON.parse(result) as KillSwitchStatus;
    } catch (error) {
      this.logger.error(`Failed to get kill switch status: ${error}`);
      return { active: false, terminated_count: 0 };
    }
  }

  /**
   * Deactivate kill switch (lift emergency)
   */
  async deactivateKillSwitch(): Promise<{ active: boolean }> {
    if (!this.bridgeLoaded) {
      return { active: false };
    }

    try {
      const result = await this.bridge.arbiterKillSwitchDeactivate();
      const parsed = JSON.parse(result) as { success: boolean };
      return { active: !parsed.success };
    } catch (error) {
      this.logger.error(`Failed to deactivate kill switch: ${error}`);
      return { active: false };
    }
  }

  // =========================================================================
  // Audit Operations
  // =========================================================================

  /**
   * Query audit statistics
   * @param _limit - Limit for future implementation (currently not used by bridge)
   */

  async getAuditStatistics(
    _limit: number = 100,
  ): Promise<AuditStatistics | null> {
    if (!this.bridgeLoaded) {
      return null;
    }

    try {
      const result = await this.bridge.arbiterQueryAudit();
      return JSON.parse(result) as AuditStatistics;
    } catch (error: unknown) {
      const errorMessage =
        error instanceof Error ? error.message : String(error);
      this.logger.error(`Failed to query audit: ${errorMessage}`);
      return null;
    }
  }

  // =========================================================================
  // Chaos Testing Operations
  // =========================================================================

  /**
   * Get chaos testing statistics
   */
  getChaosStats(): ChaosStats {
    if (!this.bridgeLoaded) {
      return { total_ops: 0, latency_injections: 0, error_injections: 0 };
    }

    try {
      const result = this.bridge.arbiterChaosStats();
      return JSON.parse(result) as ChaosStats;
    } catch (error) {
      this.logger.error(`Failed to get chaos stats: ${error}`);
      return { total_ops: 0, latency_injections: 0, error_injections: 0 };
    }
  }
}
