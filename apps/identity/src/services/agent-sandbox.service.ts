/**
 * AgentKernIdentity - Agent Sandbox Service
 *
 * Provides agent isolation, budget enforcement, and kill switch capabilities
 * for secure AI agent execution. Implements the Autonomous AI Agent Security
 * Framework requirements.
 *
 * Features:
 * - Agent execution isolation
 * - Budget/cost tracking and enforcement
 * - Emergency kill switch
 * - Reputation scoring
 * - Rate limiting per agent
 */

import { Injectable, Logger, OnModuleInit } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import { AuditLoggerService, AuditEventType } from './audit-logger.service';
import { AgentRecordEntity } from '../entities/agent-record.entity';
import { SystemConfigEntity } from '../entities/system-config.entity';

import {
  AgentStatus,
  AgentBudget,
  AgentUsage,
  AgentReputation,
  AgentRecord,
} from '../domain/agent.entity';

// Action request for sandbox execution
export interface SandboxActionRequest {
  agentId: string;
  action: string;
  target: {
    service: string;
    endpoint: string;
    method: string;
  };
  estimatedTokens?: number;
  estimatedCost?: number;
}

// Sandbox execution result
export interface SandboxResult {
  allowed: boolean;
  agentStatus: AgentStatus;
  reason?: string;
  remainingBudget?: {
    tokens: number;
    apiCalls: number;
    costUsd: number;
  };
}

@Injectable()
export class AgentSandboxService implements OnModuleInit {
  private readonly logger = new Logger(AgentSandboxService.name);

  // Agent registry
  private agents: Map<string, AgentRecord> = new Map();

  // Global kill switch (cached, persisted to database)
  private globalKillSwitchCache = false;

  // Default budgets
  private defaultBudget: AgentBudget = {
    maxTokens: 1000000, // 1M tokens per day
    maxApiCalls: 10000, // 10k API calls per day
    maxCostUsd: 100, // $100 per day
    periodSeconds: 86400, // 24 hours
  };

  // Rate limiting
  private rateLimits: Map<string, { count: number; resetAt: number }> = new Map();
  private readonly RATE_LIMIT_WINDOW = 60000; // 1 minute
  private readonly RATE_LIMIT_MAX = 100; // 100 requests per minute

  constructor(
    private readonly configService: ConfigService,
    private readonly auditLogger: AuditLoggerService,
    @InjectRepository(AgentRecordEntity)
    private readonly agentRepository: Repository<AgentRecordEntity>,
    @InjectRepository(SystemConfigEntity)
    private readonly configRepository: Repository<SystemConfigEntity>,
  ) {}

  async onModuleInit(): Promise<void> {
    // Load default budget from config
    const maxTokens = this.configService.get<number>('AGENT_MAX_TOKENS');
    if (maxTokens) this.defaultBudget.maxTokens = maxTokens;

    const maxApiCalls = this.configService.get<number>('AGENT_MAX_API_CALLS');
    if (maxApiCalls) this.defaultBudget.maxApiCalls = maxApiCalls;

    const maxCost = this.configService.get<number>('AGENT_MAX_COST_USD');
    if (maxCost) this.defaultBudget.maxCostUsd = maxCost;

    // Load kill switch state from database
    const killSwitchConfig = await this.configRepository.findOne({
      where: { key: 'global_kill_switch' },
    });
    this.globalKillSwitchCache = killSwitchConfig?.value === 'true';

    // Load agents from database into memory cache
    const dbAgents = await this.agentRepository.find();
    for (const agent of dbAgents) {
      this.agents.set(agent.id, {
        ...agent,
        createdAt: agent.createdAt.toISOString(),
        lastActiveAt: agent.lastActiveAt.toISOString(),
        terminatedAt: agent.terminatedAt?.toISOString(),
      });
    }

    this.logger.log('🔒 Agent Sandbox Service initialized');
    this.logger.log(`   Loaded ${dbAgents.length} agents from database`);
    this.logger.log(`   Kill switch: ${this.globalKillSwitchCache ? 'ACTIVE' : 'inactive'}`);
    this.logger.log(`   Default budget: ${JSON.stringify(this.defaultBudget)}`);
  }

  /**
   * Register a new agent
   */
  async registerAgent(
    agentId: string,
    name: string,
    version: string,
    customBudget?: Partial<AgentBudget>,
  ): Promise<AgentRecord> {
    if (this.agents.has(agentId)) {
      return this.agents.get(agentId)!;
    }

    const now = new Date();
    const agent: AgentRecord = {
      id: agentId,
      name,
      version,
      status: AgentStatus.ACTIVE,
      budget: { ...this.defaultBudget, ...customBudget },
      usage: {
        tokensUsed: 0,
        apiCallsUsed: 0,
        costUsd: 0,
        periodStart: now,
      },
      reputation: {
        score: 500, // Start with neutral score
        successfulActions: 0,
        failedActions: 0,
        violations: 0,
        lastUpdated: now.toISOString(),
      },
      createdAt: now.toISOString(),
      lastActiveAt: now.toISOString(),
    };

    // Keep in cache
    this.agents.set(agentId, agent);

    // Persist to DB
    const entity = this.agentRepository.create({
      ...agent,
      createdAt: new Date(agent.createdAt),
      lastActiveAt: new Date(agent.lastActiveAt),
    });
    await this.agentRepository.save(entity);

    this.logger.log(`Registered agent: ${agentId} (${name} v${version})`);

    return agent;
  }

  /**
   * Check if an action is allowed within sandbox constraints
   */
  async checkAction(request: SandboxActionRequest): Promise<SandboxResult> {
    // Check global kill switch
    if (this.globalKillSwitchCache) {
      return {
        allowed: false,
        agentStatus: AgentStatus.TERMINATED,
        reason: 'Global kill switch activated',
      };
    }

    // Get or create agent record
    let agent = this.agents.get(request.agentId);
    if (!agent) {
      agent = await this.registerAgent(request.agentId, 'Unknown', '0.0.0');
    }

    // Check agent status
    if (agent.status !== AgentStatus.ACTIVE) {
      return {
        allowed: false,
        agentStatus: agent.status,
        reason: `Agent is ${agent.status}`,
      };
    }

    // Check rate limit
    if (!this.checkRateLimit(request.agentId)) {
      agent.status = AgentStatus.RATE_LIMITED;
      return {
        allowed: false,
        agentStatus: AgentStatus.RATE_LIMITED,
        reason: 'Rate limit exceeded',
      };
    }

    // Reset budget if period expired
    this.resetBudgetIfNeeded(agent);

    // Check budget
    const budgetResult = this.checkBudget(agent, request);
    if (!budgetResult.allowed) {
      agent.status = AgentStatus.BUDGET_EXCEEDED;
      return budgetResult;
    }

    // Check reputation (low reputation = more restrictions)
    if (agent.reputation.score < 100) {
      return {
        allowed: false,
        agentStatus: agent.status,
        reason: 'Reputation too low - agent suspended',
      };
    }

    // Update last active time
    agent.lastActiveAt = new Date().toISOString();
    
    // Defer DB update for performance
    this.syncToDb(agent);

    return {
      allowed: true,
      agentStatus: agent.status,
      remainingBudget: {
        tokens: (agent.budget.maxTokens || 0) - agent.usage.tokensUsed,
        apiCalls: (agent.budget.maxApiCalls || 0) - agent.usage.apiCallsUsed,
        costUsd: (agent.budget.maxCostUsd || 0) - agent.usage.costUsd,
      },
    };
  }

  /**
   * Record successful action (improves reputation)
   */
  async recordSuccess(agentId: string, tokensUsed?: number, cost?: number): Promise<void> {
    const agent = this.agents.get(agentId);
    if (!agent) return;

    agent.usage.apiCallsUsed++;
    if (tokensUsed) agent.usage.tokensUsed += tokensUsed;
    if (cost) agent.usage.costUsd += cost;

    agent.reputation.successfulActions++;
    agent.reputation.score = Math.min(1000, agent.reputation.score + 1);
    agent.reputation.lastUpdated = new Date().toISOString();

    // Restore rate-limited agents after success
    if (agent.status === AgentStatus.RATE_LIMITED) {
      agent.status = AgentStatus.ACTIVE;
    }

    await this.syncToDb(agent);
  }

  /**
   * Record failed action (degrades reputation)
   */
  async recordFailure(agentId: string, reason: string): Promise<void> {
    const agent = this.agents.get(agentId);
    if (!agent) return;

    agent.reputation.failedActions++;
    agent.reputation.score = Math.max(0, agent.reputation.score - 10);
    agent.reputation.lastUpdated = new Date().toISOString();

    this.syncToDb(agent);

    this.auditLogger.logSecurityEvent(
      AuditEventType.SUSPICIOUS_ACTIVITY,
      `Agent action failed: ${reason}`,
      { agentId, newScore: agent.reputation.score },
    );
  }

  /**
   * Record security violation (severely degrades reputation)
   */
  async recordViolation(agentId: string, violation: string): Promise<void> {
    const agent = this.agents.get(agentId);
    if (!agent) return;

    agent.reputation.violations++;
    agent.reputation.score = Math.max(0, agent.reputation.score - 100);
    agent.reputation.lastUpdated = new Date().toISOString();

    // Auto-suspend on multiple violations
    if (agent.reputation.violations >= 3) {
      await this.suspendAgent(agentId, `Multiple violations: ${violation}`);
    }

    this.syncToDb(agent);

    this.auditLogger.logSecurityEvent(
      AuditEventType.SECURITY_ALERT,
      `Agent security violation: ${violation}`,
      { agentId, violations: agent.reputation.violations, newScore: agent.reputation.score },
    );
  }

  /**
   * Suspend an agent temporarily
   */
  async suspendAgent(agentId: string, reason: string): Promise<boolean> {
    const agent = this.agents.get(agentId);
    if (!agent) return false;

    agent.status = AgentStatus.SUSPENDED;
    await this.syncToDb(agent);
    this.logger.warn(`⚠️ Agent suspended: ${agentId} - ${reason}`);

    this.auditLogger.logSecurityEvent(
      AuditEventType.SECURITY_ALERT,
      `Agent suspended: ${reason}`,
      { agentId },
    );

    return true;
  }

  /**
   * Terminate an agent permanently (kill switch)
   */
  async terminateAgent(agentId: string, reason: string): Promise<boolean> {
    const agent = this.agents.get(agentId);
    if (!agent) return false;

    agent.status = AgentStatus.TERMINATED;
    agent.terminatedAt = new Date().toISOString();
    agent.terminationReason = reason;

    await this.syncToDb(agent);
    this.logger.error(`🚨 AGENT TERMINATED: ${agentId} - ${reason}`);

    this.auditLogger.logSecurityEvent(
      AuditEventType.SECURITY_ALERT,
      `AGENT TERMINATED: ${reason}`,
      { agentId, terminatedAt: agent.terminatedAt },
    );

    return true;
  }

  /**
   * Reactivate a suspended agent
   */
  async reactivateAgent(agentId: string): Promise<boolean> {
    const agent = this.agents.get(agentId);
    if (!agent) return false;

    if (agent.status === AgentStatus.TERMINATED) {
      this.logger.warn(`Cannot reactivate terminated agent: ${agentId}`);
      return false;
    }

    agent.status = AgentStatus.ACTIVE;
    await this.syncToDb(agent);
    this.logger.log(`Agent reactivated: ${agentId}`);

    return true;
  }

  /**
   * Activate global kill switch (stops ALL agents)
   * Persisted to database for cross-instance consistency
   */
  async activateGlobalKillSwitch(reason: string): Promise<void> {
    this.globalKillSwitchCache = true;
    
    // Persist to database
    await this.configRepository.upsert(
      {
        key: 'global_kill_switch',
        value: 'true',
        description: `Activated: ${reason}`,
        valueType: 'boolean',
      },
      ['key'],
    );

    this.logger.error(`🚨 GLOBAL KILL SWITCH ACTIVATED: ${reason}`);

    await this.auditLogger.logSecurityEvent(
      AuditEventType.KILL_SWITCH_ACTIVATED,
      `GLOBAL KILL SWITCH ACTIVATED: ${reason}`,
      { activeAgents: this.agents.size },
    );

    // Terminate all active agents
    for (const [agentId, agent] of this.agents) {
      if (agent.status === AgentStatus.ACTIVE) {
        agent.status = AgentStatus.TERMINATED;
        agent.terminatedAt = new Date().toISOString();
        agent.terminationReason = `Global kill switch: ${reason}`;
        await this.syncToDb(agent);
      }
    }
  }

  /**
   * Deactivate global kill switch
   * Persisted to database for cross-instance consistency
   */
  async deactivateGlobalKillSwitch(): Promise<void> {
    this.globalKillSwitchCache = false;

    // Persist to database
    await this.configRepository.upsert(
      {
        key: 'global_kill_switch',
        value: 'false',
        description: 'Deactivated',
        valueType: 'boolean',
      },
      ['key'],
    );

    this.logger.log('Global kill switch deactivated');
  }

  /**
   * Get agent status
   */
  getAgentStatus(agentId: string): AgentRecord | null {
    return this.agents.get(agentId) || null;
  }

  /**
   * Get all agents
   */
  getAllAgents(): AgentRecord[] {
    return Array.from(this.agents.values());
  }

  /**
   * Get agent reputation
   */
  getAgentReputation(agentId: string): AgentReputation | null {
    return this.agents.get(agentId)?.reputation || null;
  }

  /**
   * Update agent budget
   */
  updateAgentBudget(agentId: string, budget: Partial<AgentBudget>): boolean {
    const agent = this.agents.get(agentId);
    if (!agent) return false;

    agent.budget = { ...agent.budget, ...budget };
    this.logger.log(`Updated budget for agent ${agentId}`);

    return true;
  }

  // ============ Private Methods ============

  private checkRateLimit(agentId: string): boolean {
    const now = Date.now();
    const limit = this.rateLimits.get(agentId);

    if (!limit || now > limit.resetAt) {
      this.rateLimits.set(agentId, {
        count: 1,
        resetAt: now + this.RATE_LIMIT_WINDOW,
      });
      return true;
    }

    if (limit.count >= this.RATE_LIMIT_MAX) {
      return false;
    }

    limit.count++;
    return true;
  }

  private resetBudgetIfNeeded(agent: AgentRecord): void {
    const now = new Date();
    const periodStart = new Date(agent.usage.periodStart);
    const periodEnd = new Date(periodStart.getTime() + agent.budget.periodSeconds * 1000);

    if (now > periodEnd) {
      agent.usage = {
        tokensUsed: 0,
        apiCallsUsed: 0,
        costUsd: 0,
        periodStart: now,
      };

      // Restore budget-exceeded agents
      if (agent.status === AgentStatus.BUDGET_EXCEEDED) {
        agent.status = AgentStatus.ACTIVE;
      }
    }
  }

  private checkBudget(
    agent: AgentRecord,
    request: SandboxActionRequest,
  ): SandboxResult {
    const estimatedTokens = request.estimatedTokens || 0;
    const estimatedCost = request.estimatedCost || 0;

    // Check tokens
    if (
      agent.budget.maxTokens &&
      agent.usage.tokensUsed + estimatedTokens > agent.budget.maxTokens
    ) {
      return {
        allowed: false,
        agentStatus: AgentStatus.BUDGET_EXCEEDED,
        reason: 'Token budget exceeded',
        remainingBudget: {
          tokens: agent.budget.maxTokens - agent.usage.tokensUsed,
          apiCalls: (agent.budget.maxApiCalls || 0) - agent.usage.apiCallsUsed,
          costUsd: (agent.budget.maxCostUsd || 0) - agent.usage.costUsd,
        },
      };
    }

    // Check API calls
    if (
      agent.budget.maxApiCalls &&
      agent.usage.apiCallsUsed + 1 > agent.budget.maxApiCalls
    ) {
      return {
        allowed: false,
        agentStatus: AgentStatus.BUDGET_EXCEEDED,
        reason: 'API call budget exceeded',
        remainingBudget: {
          tokens: (agent.budget.maxTokens || 0) - agent.usage.tokensUsed,
          apiCalls: agent.budget.maxApiCalls - agent.usage.apiCallsUsed,
          costUsd: (agent.budget.maxCostUsd || 0) - agent.usage.costUsd,
        },
      };
    }

    // Check cost
    if (
      agent.budget.maxCostUsd &&
      agent.usage.costUsd + estimatedCost > agent.budget.maxCostUsd
    ) {
      return {
        allowed: false,
        agentStatus: AgentStatus.BUDGET_EXCEEDED,
        reason: 'Cost budget exceeded',
        remainingBudget: {
          tokens: (agent.budget.maxTokens || 0) - agent.usage.tokensUsed,
          apiCalls: (agent.budget.maxApiCalls || 0) - agent.usage.apiCallsUsed,
          costUsd: agent.budget.maxCostUsd - agent.usage.costUsd,
        },
      };
    }

    return {
      allowed: true,
      agentStatus: agent.status,
    };
  }

  private async syncToDb(agent: AgentRecord): Promise<void> {
    try {
      await this.agentRepository.save({
        ...agent,
        createdAt: new Date(agent.createdAt),
        lastActiveAt: new Date(agent.lastActiveAt),
        terminatedAt: agent.terminatedAt ? new Date(agent.terminatedAt) : undefined,
      });
    } catch (error) {
      this.logger.error(`Failed to sync agent ${agent.id} to DB: ${error}`);
    }
  }
}
