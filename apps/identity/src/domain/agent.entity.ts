/**
 * AgentKernIdentity - Agent Domain Entities
 */

export enum AgentStatus {
  ACTIVE = 'active',
  SUSPENDED = 'suspended',
  TERMINATED = 'terminated',
  RATE_LIMITED = 'rate_limited',
  BUDGET_EXCEEDED = 'budget_exceeded',
}

export class AgentBudget {
  maxTokens?: number;
  maxApiCalls?: number;
  maxCostUsd?: number;
  periodSeconds: number;
}

export class AgentUsage {
  tokensUsed: number;
  apiCallsUsed: number;
  costUsd: number;
  periodStart: Date | string;
}

export class AgentReputation {
  score: number;
  successfulActions: number;
  failedActions: number;
  violations: number;
  lastUpdated: string | Date;
}

export class AgentRecord {
  id: string;
  name: string;
  version: string;
  status: AgentStatus;
  budget: AgentBudget;
  usage: AgentUsage;
  reputation: AgentReputation;
  createdAt: string | Date;
  lastActiveAt: string | Date;
  terminatedAt?: string | Date;
  terminationReason?: string;
}
