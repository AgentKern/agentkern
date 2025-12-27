/**
 * AgentKern Gateway - Gate Service
 * 
 * Business logic for policy verification (Guardrails).
 * Per ENGINEERING_STANDARD: Neuro-Symbolic Guards (Code + AI).
 */

import { Injectable } from '@nestjs/common';

interface Policy {
  id: string;
  name: string;
  rules: PolicyRule[];
  jurisdictions: string[];
  priority: number;
  enabled: boolean;
}

interface PolicyRule {
  id: string;
  condition: string;
  action: 'allow' | 'deny' | 'review' | 'audit';
  message?: string;
}

interface VerificationResult {
  allowed: boolean;
  evaluatedPolicies: string[];
  blockingPolicies: string[];
  riskScore: number;
  reasoning?: string;
  latencyMs: number;
}

interface CarbonBudget {
  dailyLimitGrams: number;
  monthlyLimitGrams?: number;
  blockOnExceed: boolean;
}

interface CarbonUsage {
  totalCo2Grams: number;
  totalEnergyKwh: number;
  actionCount: number;
  periodStart: string;
  periodEnd: string;
}

interface Attestation {
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
    // In production, this calls the Rust TEE module via FFI/gRPC
    return {
      platform: 'simulated',
      quote: Buffer.from(`simulated-quote-${nonce}`).toString('base64'),
      measurement: 'f2d4e5a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9',
      nonce,
      timestamp: Date.now(),
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
