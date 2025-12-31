/**
 * AgentKern Gateway - Gate Service
 * 
 * Business logic for policy verification (Guardrails).
 * Per ENGINEERING_STANDARD: Neuro-Symbolic Guards (Code + AI).
 * 
 * ⚠️ WARNING: MOCK IMPLEMENTATION (EPISTEMIC DEBT DETECTED)
 * This service is currently a standalone Node.js mock.
 * It does NOT connect to the `packages/gate` Rust crate.
 * TEE Attestation is SIMULATED. Do not use in production without Rust Bridge.
 */

import { Injectable } from '@nestjs/common';

export interface Policy {
  id: string;
  name: string;
  rules: PolicyRule[];
  jurisdictions: string[];
  priority: number;
  enabled: boolean;
}

export interface PolicyRule {
  id: string;
  condition: string;
  action: 'allow' | 'deny' | 'review' | 'audit';
  message?: string;
}

export interface VerificationResult {
  allowed: boolean;
  evaluatedPolicies: string[];
  blockingPolicies: string[];
  riskScore: number;
  reasoning?: string;
  latencyMs: number;
}

export interface CarbonBudget {
  dailyLimitGrams: number;
  monthlyLimitGrams?: number;
  blockOnExceed: boolean;
}

export interface CarbonUsage {
  totalCo2Grams: number;
  totalEnergyKwh: number;
  actionCount: number;
  periodStart: string;
  periodEnd: string;
}

export interface Attestation {
  platform: 'intel_tdx' | 'amd_sev_snp' | 'simulated';
  quote: string;
  measurement: string;
  nonce: string;
  timestamp: number;
}

@Injectable()
export class GateService {
  // In-memory policy store (replace with Redis/database in production)
  // In-memory store
  private readonly policies = new Map<string, Policy>();
  private readonly carbonBudgets = new Map<string, CarbonBudget>();
  private readonly carbonUsage = new Map<string, CarbonUsage>();

  // Bridge to Rust Logic
  // eslint-disable-next-line @typescript-eslint/no-var-requires
  private readonly bridge = require('../../../../packages/bridge/index.node');

  async verify(
    agentId: string,
    action: string,
    context: Record<string, unknown> = {},
  ): Promise<VerificationResult> {
    const startTime = Date.now();
    const evaluatedPolicies: string[] = [];
    const blockingPolicies: string[] = [];
    let riskScore = 0;

    // Evaluate all enabled policies
    for (const policy of this.policies.values()) {
      if (!policy.enabled) continue;
      
      evaluatedPolicies.push(policy.id);
      
      // Simple rule evaluation (replace with DSL parser in production)
      for (const rule of policy.rules) {
        const isBlocking = this.evaluateRule(rule, action, context);
        if (isBlocking && rule.action === 'deny') {
          blockingPolicies.push(policy.id);
          riskScore = Math.max(riskScore, 100);
        }
      }
    }

    // Calculate risk score based on action type (simplified)
    if (action.includes('delete') || action.includes('remove')) {
      riskScore = Math.max(riskScore, 70);
    } else if (action.includes('transfer') || action.includes('payment')) {
      riskScore = Math.max(riskScore, 50);
    }

    return {
      allowed: blockingPolicies.length === 0,
      evaluatedPolicies,
      blockingPolicies,
      riskScore,
      reasoning: blockingPolicies.length > 0 
        ? `Blocked by policies: ${blockingPolicies.join(', ')}`
        : 'All policies passed',
      latencyMs: Date.now() - startTime,
    };
  }

  async registerPolicy(policy: Policy): Promise<Policy> {
    const policyWithDefaults: Policy = {
      ...policy,
      enabled: policy.enabled ?? true,
    };
    this.policies.set(policy.id, policyWithDefaults);
    return policyWithDefaults;
  }

  async getPolicies(): Promise<Policy[]> {
    return Array.from(this.policies.values());
  }

  async attest(nonce: string): Promise<Attestation> {
    // 🌉 BRIDGE: Calling Rust Kernel (packages/gate via N-API)
    const rawAttestation = this.bridge.attest(nonce);
    const attestation = JSON.parse(rawAttestation);
    
    return {
      platform: attestation.platform === 'IntelTdx' ? 'intel_tdx' : 'simulated',
      quote: Buffer.from(attestation.quote).toString('base64'),
      measurement: Buffer.from(attestation.measurement).toString('hex'),
      nonce,
      timestamp: attestation.timestamp,
    };
  }

  async getCarbonBudget(agentId: string): Promise<CarbonBudget> {
    return this.carbonBudgets.get(agentId) || {
      dailyLimitGrams: 1000,
      monthlyLimitGrams: 25000,
      blockOnExceed: false,
    };
  }

  async setCarbonBudget(agentId: string, budget: CarbonBudget): Promise<void> {
    this.carbonBudgets.set(agentId, budget);
  }

  async getCarbonUsage(agentId: string): Promise<CarbonUsage> {
    return this.carbonUsage.get(agentId) || {
      totalCo2Grams: 0,
      totalEnergyKwh: 0,
      actionCount: 0,
      periodStart: new Date().toISOString(),
      periodEnd: new Date().toISOString(),
    };
  }

  private evaluateRule(
    rule: PolicyRule,
    action: string,
    context: Record<string, unknown>,
  ): boolean {
    // Simple condition matching (replace with proper DSL evaluation)
    // Example: "action == 'transfer' && amount > 10000"
    try {
      // For now, just check if action matches a pattern in condition
      if (rule.condition.includes(action)) {
        return true;
      }
      return false;
    } catch {
      return false;
    }
  }
}
