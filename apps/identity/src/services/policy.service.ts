/**
 * AgentProof - Policy Service
 * 
 * Enterprise policy management for controlling agent actions.
 */

import { Injectable, Logger } from '@nestjs/common';
import { PolicyRuleDto, PolicyAction } from '../dto/dashboard.dto';

export interface Policy {
  id: string;
  name: string;
  description: string;
  rules: PolicyRuleDto[];
  targetAgents: string[];
  targetPrincipals: string[];
  active: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface PolicyEvaluationContext {
  agentId: string;
  principalId: string;
  action: string;
  target: { service: string; endpoint: string; method: string };
  parameters?: Record<string, unknown>;
  amount?: number;
  time?: Date;
}

export interface PolicyEvaluationResult {
  allowed: boolean;
  matchedPolicy?: string;
  matchedRule?: string;
  action: PolicyAction;
  message?: string;
}

@Injectable()
export class PolicyService {
  private readonly logger = new Logger(PolicyService.name);
  
  // In-memory policy storage (use database in production)
  private policies: Map<string, Policy> = new Map();

  constructor() {
    // Create default policies
    this.createDefaultPolicies();
  }

  /**
   * Create a new policy
   */
  createPolicy(
    name: string,
    description: string,
    rules: PolicyRuleDto[],
    targetAgents: string[] = [],
    targetPrincipals: string[] = [],
  ): Policy {
    const policy: Policy = {
      id: crypto.randomUUID(),
      name,
      description,
      rules,
      targetAgents,
      targetPrincipals,
      active: true,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };

    this.policies.set(policy.id, policy);
    this.logger.log(`Created policy: ${policy.name} (${policy.id})`);

    return policy;
  }

  /**
   * Get all policies
   */
  getAllPolicies(): Policy[] {
    return Array.from(this.policies.values());
  }

  /**
   * Get policy by ID
   */
  getPolicy(id: string): Policy | null {
    return this.policies.get(id) || null;
  }

  /**
   * Update a policy
   */
  updatePolicy(id: string, updates: Partial<Omit<Policy, 'id' | 'createdAt'>>): Policy | null {
    const policy = this.policies.get(id);
    if (!policy) return null;

    const updated: Policy = {
      ...policy,
      ...updates,
      updatedAt: new Date().toISOString(),
    };

    this.policies.set(id, updated);
    return updated;
  }

  /**
   * Delete a policy
   */
  deletePolicy(id: string): boolean {
    return this.policies.delete(id);
  }

  /**
   * Activate/deactivate a policy
   */
  setActive(id: string, active: boolean): Policy | null {
    return this.updatePolicy(id, { active });
  }

  /**
   * Evaluate all policies against a context
   */
  evaluatePolicies(context: PolicyEvaluationContext): PolicyEvaluationResult {
    const activePolicies = Array.from(this.policies.values()).filter(p => p.active);

    for (const policy of activePolicies) {
      // Check if policy applies to this agent/principal
      if (!this.policyApplies(policy, context)) continue;

      // Evaluate each rule
      for (const rule of policy.rules) {
        if (this.evaluateCondition(rule.condition, context)) {
          return {
            allowed: rule.action === PolicyAction.ALLOW,
            matchedPolicy: policy.name,
            matchedRule: rule.name,
            action: rule.action,
            message: this.getActionMessage(rule.action, rule),
          };
        }
      }
    }

    // Default: allow if no policy matches
    return {
      allowed: true,
      action: PolicyAction.ALLOW,
    };
  }

  private policyApplies(policy: Policy, context: PolicyEvaluationContext): boolean {
    // If no targets specified, policy applies to all
    if (policy.targetAgents.length === 0 && policy.targetPrincipals.length === 0) {
      return true;
    }

    // Check agent match
    if (policy.targetAgents.length > 0 && !policy.targetAgents.includes(context.agentId)) {
      return false;
    }

    // Check principal match
    if (policy.targetPrincipals.length > 0 && !policy.targetPrincipals.includes(context.principalId)) {
      return false;
    }

    return true;
  }

  private evaluateCondition(condition: string, context: PolicyEvaluationContext): boolean {
    try {
      // Simple condition parser (in production, use a proper expression evaluator)
      // Supported: action == 'x', amount > 1000, method == 'POST'
      
      if (condition === 'true') return true;
      if (condition === 'false') return false;

      // Action condition
      const actionMatch = condition.match(/action\s*==\s*['"](.+)['"]/);
      if (actionMatch && context.action === actionMatch[1]) {
        return true;
      }

      // Amount condition
      const amountMatch = condition.match(/amount\s*([><=]+)\s*(\d+)/);
      if (amountMatch && context.amount !== undefined) {
        const operator = amountMatch[1];
        const value = parseInt(amountMatch[2], 10);
        switch (operator) {
          case '>': return context.amount > value;
          case '<': return context.amount < value;
          case '>=': return context.amount >= value;
          case '<=': return context.amount <= value;
          case '==': return context.amount === value;
        }
      }

      // Method condition
      const methodMatch = condition.match(/method\s*==\s*['"](.+)['"]/);
      if (methodMatch && context.target.method === methodMatch[1]) {
        return true;
      }

      // Service condition
      const serviceMatch = condition.match(/service\s*==\s*['"](.+)['"]/);
      if (serviceMatch && context.target.service === serviceMatch[1]) {
        return true;
      }

      return false;
    } catch (error) {
      this.logger.warn(`Failed to evaluate condition: ${condition}`, error);
      return false;
    }
  }

  private getActionMessage(action: PolicyAction, rule: PolicyRuleDto): string {
    switch (action) {
      case PolicyAction.ALLOW:
        return 'Action allowed by policy';
      case PolicyAction.DENY:
        return `Action denied by rule: ${rule.name}`;
      case PolicyAction.REQUIRE_CONFIRMATION:
        return 'Action requires manual confirmation';
      case PolicyAction.RATE_LIMIT:
        return `Action rate limited to ${rule.rateLimit} per minute`;
      default:
        return 'Unknown action';
    }
  }

  private createDefaultPolicies(): void {
    // High-value transaction policy
    this.createPolicy(
      'High-Value Transactions',
      'Require confirmation for transactions over $10,000',
      [
        {
          name: 'Require confirmation above threshold',
          condition: 'amount > 10000',
          action: PolicyAction.REQUIRE_CONFIRMATION,
        },
      ],
    );

    // DELETE method policy
    this.createPolicy(
      'Destructive Actions',
      'Rate limit DELETE operations',
      [
        {
          name: 'Rate limit DELETE',
          condition: "method == 'DELETE'",
          action: PolicyAction.RATE_LIMIT,
          rateLimit: 10,
        },
      ],
    );

    this.logger.log(`Initialized ${this.policies.size} default policies`);
  }
}
