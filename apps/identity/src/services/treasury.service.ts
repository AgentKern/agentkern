/**
 * AgentKern Identity - Treasury Service
 *
 * Wrapper for N-API bridge to Rust Treasury logic.
 * Provides agent payment operations: balances, transfers, budgets, carbon tracking.
 *
 * Per DECISION_RECORD_BRIDGE.md: N-API for hot path (0ms latency)
 */

import { Injectable, Logger, OnModuleInit } from '@nestjs/common';
import * as path from 'path';
import * as fs from 'fs';

// Type definitions for bridge responses
export interface AgentBalance {
  agent_id: string;
  balance: {
    value: number;
    decimals: number;
  };
  currency: string;
  pending: {
    value: number;
    decimals: number;
  };
  updated_at: string;
  total_deposited: {
    value: number;
    decimals: number;
  };
  total_withdrawn: {
    value: number;
    decimals: number;
  };
}

export interface TransferResult {
  transaction_id: string;
  status: 'Pending' | 'Completed' | 'Failed' | 'Cancelled';
  timestamp: string;
  error?: string;
}

export interface BudgetRemaining {
  agent_id: string;
  remaining: number | null;
  message?: string;
}

export interface CarbonUsage {
  total_co2_grams: string;
  total_energy_kwh: string;
  total_water_liters: string;
  action_count: number;
  period_start: string;
  period_end: string;
}

export interface CarbonOffset {
  transaction_id: string;
  tons: number;
  cost: number;
  provider: string;
  certificate_url: string;
  timestamp: string;
}

// Bridge interface (loaded from native module)
interface NativeBridge {
  treasuryGetBalance(agentId: string): string;
  treasuryDeposit(agentId: string, amount: number): string;
  treasuryTransfer(
    fromAgent: string,
    toAgent: string,
    amount: number,
    reference?: string,
  ): Promise<string>;
  treasuryGetBudget(agentId: string): string;
  treasuryGetCarbon(agentId: string): string;
  treasuryPurchaseOffset(agentId: string, tons: number): string;
}

@Injectable()
export class TreasuryService implements OnModuleInit {
  private readonly logger = new Logger(TreasuryService.name);
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
      this.logger.log('💰 Treasury N-API Bridge loaded successfully');

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
          `🚨 Failed to load Treasury N-API bridge: ${errorMessage}`,
        );
        this.logger.warn(
          '⚠️ DEPRECATED: TreasuryService operating in degraded mode. See EPISTEMIC_HEALTH.md',
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
      const testResult = this.bridge.treasuryGetBalance('test-verify');
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
   * Get agent balance
   */
  async getBalance(agentId: string): Promise<AgentBalance | null> {
    await Promise.resolve();
    if (!this.bridgeLoaded) {
      this.logger.warn('Bridge not loaded, returning null for getBalance');
      return null;
    }

    try {
      const result = this.bridge.treasuryGetBalance(agentId);
      return JSON.parse(result) as AgentBalance;
    } catch (error) {
      this.logger.error(`Failed to get balance for ${agentId}: ${error}`);
      return null;
    }
  }

  /**
   * Deposit funds to agent balance
   */
  async deposit(
    agentId: string,
    amount: number,
  ): Promise<AgentBalance | { error: string }> {
    await Promise.resolve();
    if (!this.bridgeLoaded) {
      return { error: 'Bridge not loaded' };
    }

    try {
      const result = this.bridge.treasuryDeposit(agentId, amount);
      return JSON.parse(result) as AgentBalance | { error: string };
    } catch (error) {
      this.logger.error(`Failed to deposit to ${agentId}: ${error}`);
      return { error: String(error) };
    }
  }

  /**
   * Purchase carbon offset
   */
  async purchaseOffset(
    agentId: string,
    tons: number,
  ): Promise<CarbonOffset | { error: string }> {
    await Promise.resolve();
    if (!this.bridgeLoaded) {
      return { error: 'Bridge not loaded' };
    }

    try {
      const result = this.bridge.treasuryPurchaseOffset(agentId, tons);
      return JSON.parse(result) as CarbonOffset;
    } catch (error) {
      this.logger.error(`Failed to purchase offset for ${agentId}: ${error}`);
      return { error: String(error) };
    }
  }

  /**
   * Transfer funds between agents (async due to 2-phase commit)
   */
  async transfer(
    fromAgent: string,
    toAgent: string,
    amount: number,
    reference?: string,
  ): Promise<TransferResult | { error: string }> {
    if (!this.bridgeLoaded) {
      return { error: 'Bridge not loaded' } as { error: string };
    }

    try {
      const result = await this.bridge.treasuryTransfer(
        fromAgent,
        toAgent,
        amount,
        reference,
      );
      return JSON.parse(result) as TransferResult;
    } catch (error) {
      this.logger.error(
        `Failed to transfer from ${fromAgent} to ${toAgent}: ${error}`,
      );
      return { error: String(error) };
    }
  }

  /**
   * Get remaining budget for agent
   */
  async getBudget(agentId: string): Promise<BudgetRemaining> {
    await Promise.resolve();
    if (!this.bridgeLoaded) {
      return {
        agent_id: agentId,
        remaining: null,
        message: 'Bridge not loaded',
      };
    }

    try {
      const result = this.bridge.treasuryGetBudget(agentId);
      return JSON.parse(result) as BudgetRemaining;
    } catch (error) {
      this.logger.error(`Failed to get budget for ${agentId}: ${error}`);
      return { agent_id: agentId, remaining: null, message: String(error) };
    }
  }

  /**
   * Get carbon footprint (daily usage)
   */
  async getCarbon(agentId: string): Promise<CarbonUsage | null> {
    await Promise.resolve();
    if (!this.bridgeLoaded) {
      return null;
    }

    try {
      const result = this.bridge.treasuryGetCarbon(agentId);
      return JSON.parse(result) as CarbonUsage;
    } catch (error) {
      this.logger.error(`Failed to get carbon for ${agentId}: ${error}`);
      return null;
    }
  }
}
