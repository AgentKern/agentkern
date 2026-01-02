/**
 * AgentKernIdentity - Agent Sandbox Service
 *
 * Provides agent isolation, budget enforcement, and kill switch capabilities
 * for secure AI agent execution. Implements the Autonomous AI Agent Security
 * Framework requirements.
 *
 * ## Architecture Overview
 *
 * This service manages agent lifecycle through a state machine:
 * ```
 * ACTIVE -> SUSPENDED -> ACTIVE (reactivatable)
 * ACTIVE -> RATE_LIMITED -> ACTIVE (auto-recovers after success)
 * ACTIVE -> BUDGET_EXCEEDED -> ACTIVE (auto-recovers on period reset)
 * ACTIVE -> TERMINATED (permanent, cannot reactivate)
 * ```
 *
 * ## Kill Switch Behavior
 *
 * The global kill switch has IMMEDIATE effect:
 * 1. Sets `globalKillSwitchCache = true` (in-memory for fast checks)
 * 2. Persists to `SystemConfigEntity` (survives restarts)
 * 3. Terminates ALL active agents asynchronously
 *
 * **In-Flight Request Handling:**
 * - New requests: Immediately blocked at `checkAction()` (sync check)
 * - In-flight requests: NOT interrupted (external systems must handle)
 * - This is by design: Interrupting mid-transaction could corrupt state
 *
 * ## Reputation System
 *
 * Score range: 0-1000 (starts at 500)
 * - Success: +1 point (gradual trust building)
 * - Failure: -10 points (faster degradation)
 * - Violation: -100 points (severe penalty)
 * - Auto-suspend: After 3 violations (configurable)
 * - Low-rep block: Score < 100 (configurable)
 *
 * @see ENGINEERING_STANDARD.md Section 5: Agent Safety
 */

import { Injectable, Logger, OnModuleInit } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import { AuditLoggerService, AuditEventType } from './audit-logger.service';
import { GateService } from './gate.service';
import { TrustService } from './trust.service';
import { AgentRecordEntity } from '../entities/agent-record.entity';
import { SystemConfigEntity } from '../entities/system-config.entity';

import {
  AgentStatus,
  AgentBudget,
  AgentReputation,
  AgentRecord,
} from '../domain/agent.entity';

// ============================================================================
// CONFIGURABLE CONSTANTS (Epistemic Clarity: These are design decisions)
// ============================================================================

/**
 * Minimum reputation score required for agent to execute actions.
 * Below this threshold, agent is effectively suspended.
 * @rationale 100 allows recovery from 1 violation (start at 500, -100 = 400 > 100)
 */
const MIN_REPUTATION_THRESHOLD = 100;

/**
 * Number of violations before automatic suspension.
 * @rationale 3 violations = pattern of misbehavior, not one-off error
 */
const VIOLATION_AUTO_SUSPEND_THRESHOLD = 3;

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

  // ========================================================================
  // STATE: In-memory caches (backed by database for persistence)
  // ========================================================================

  /**
   * Agent registry cache. Loaded from DB on init, synced on changes.
   * @design In-memory for fast lookups (every request checks this)
   */
  private agents: Map<string, AgentRecord> = new Map();

  /**
   * Global kill switch cache. Persisted to SystemConfigEntity.
   * @design Cached here because EVERY request checks this first.
   * DB roundtrip would add ~1ms latency to every single request.
   */
  private globalKillSwitchCache = false;

  /**
   * In-flight request counter per agent.
   * Used to track how many requests are currently being processed.
   * @design Helps with graceful shutdown and debugging.
   */
  private inFlightRequests: Map<string, number> = new Map();

  // ========================================================================
  // CONFIGURATION
  // ========================================================================

  /** Default budget applied to new agents */
  private defaultBudget: AgentBudget = {
    maxTokens: 1000000, // 1M tokens per day
    maxApiCalls: 10000, // 10k API calls per day
    maxCostUsd: 100, // $100 per day
    periodSeconds: 86400, // 24 hours
  };

  /** Rate limits per agent (ephemeral, resets on restart) */
  private rateLimits: Map<string, { count: number; resetAt: number }> =
    new Map();
  private readonly RATE_LIMIT_WINDOW = 60000; // 1 minute
  private readonly RATE_LIMIT_MAX = 100; // 100 requests per minute

  constructor(
    private readonly configService: ConfigService,
    private readonly auditLogger: AuditLoggerService,
    private readonly gateService: GateService,
    private readonly trustService: TrustService,
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
    this.logger.log(
      `   Kill switch: ${this.globalKillSwitchCache ? 'ACTIVE' : 'inactive'}`,
    );
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
   * Check if an action is allowed within sandbox constraints.
   *
   * This is the primary entry point for all agent action requests.
   * Checks are performed in this order (fail-fast):
   *
   * 1. **Global Kill Switch** — Immediate block, no recovery
   * 2. **Agent Status** — Must be ACTIVE
   * 3. **Rate Limit** — Per-minute request limit
   * 4. **Budget** — Token/API/cost limits
   * 5. **Reputation** — Minimum score threshold
   *
   * @param request - The action request to validate
   * @returns SandboxResult with allowed flag and remaining budget
   *
   * @example
   * ```typescript
   * const result = await sandbox.checkAction({
   *   agentId: 'agent-123',
   *   action: 'send_email',
   *   target: { service: 'email', endpoint: '/send', method: 'POST' },
   *   estimatedTokens: 1000,
   * });
   * if (!result.allowed) {
   *   throw new ForbiddenException(result.reason);
   * }
   * // Proceed with action, then call recordSuccess() or recordFailure()
   * ```
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
    if (agent.reputation.score < MIN_REPUTATION_THRESHOLD) {
      return {
        allowed: false,
        agentStatus: agent.status,
        reason: `Reputation too low (${agent.reputation.score} < ${MIN_REPUTATION_THRESHOLD}) - agent suspended`,
      };
    }

    // 🛡️ HOT PATH: Prompt Injection Guard (N-API, 0ms latency)
    // Per DECISION_RECORD_BRIDGE.md: Rust-native prompt guard on every action
    const promptAnalysis = this.gateService.guardPrompt(request.action);
    if (promptAnalysis) {
      if (
        promptAnalysis.threat_level === 'High' ||
        promptAnalysis.threat_level === 'Critical'
      ) {
        // Record violation and block
        await this.recordViolation(
          request.agentId,
          `Prompt injection detected: ${promptAnalysis.attacks.join(', ')}`,
        );
        return {
          allowed: false,
          agentStatus: agent.status,
          reason: `Prompt injection detected (${promptAnalysis.threat_level}): ${promptAnalysis.matched_patterns.join(', ')}`,
        };
      }

      // Log suspicious prompts for audit (Medium/Low threat)
      if (promptAnalysis.threat_level === 'Medium') {
        this.logger.warn(
          `Suspicious prompt from ${request.agentId}: ${promptAnalysis.matched_patterns.join(', ')}`,
        );
      }
    }

    // Update last active time
    agent.lastActiveAt = new Date().toISOString();

    // Track in-flight request
    this.incrementInFlight(request.agentId);

    // Defer DB update for performance
    void this.syncToDb(agent);

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
   * Record successful action (improves reputation).
   *
   * MUST be called after a successful action that was approved via checkAction().
   * This updates usage metrics and slightly improves reputation.
   *
   * @param agentId - The agent that completed the action
   * @param tokensUsed - Actual tokens consumed (for budget tracking)
   * @param cost - Actual cost incurred (for budget tracking)
   */
  async recordSuccess(
    agentId: string,
    tokensUsed?: number,
    cost?: number,
  ): Promise<void> {
    const agent = this.agents.get(agentId);
    if (!agent) return;

    // Decrement in-flight counter
    this.decrementInFlight(agentId);

    // Update budget usage (stays in AgentSandboxService)
    agent.usage.apiCallsUsed++;
    if (tokensUsed) agent.usage.tokensUsed += tokensUsed;
    if (cost) agent.usage.costUsd += cost;

    // Delegate reputation tracking to TrustService
    const trustScore =
      await this.trustService.recordTransactionSuccess(agentId);
    agent.reputation.successfulActions++;
    agent.reputation.score = trustScore.score * 10; // Convert 0-100 to 0-1000
    agent.reputation.lastUpdated = new Date().toISOString();

    // Restore rate-limited agents after success
    if (agent.status === AgentStatus.RATE_LIMITED) {
      agent.status = AgentStatus.ACTIVE;
    }

    await this.syncToDb(agent);
  }

  /**
   * Record failed action (degrades reputation).
   *
   * MUST be called when an action that was approved via checkAction() fails.
   * This degrades reputation and logs the failure for audit.
   *
   * @param agentId - The agent that failed the action
   * @param reason - Human-readable reason for failure (logged for audit)
   */
  async recordFailure(agentId: string, reason: string): Promise<void> {
    const agent = this.agents.get(agentId);
    if (!agent) return;

    // Decrement in-flight counter
    this.decrementInFlight(agentId);

    // Delegate reputation tracking to TrustService
    const trustScore = await this.trustService.recordTransactionFailure(
      agentId,
      reason,
    );
    agent.reputation.failedActions++;
    agent.reputation.score = trustScore.score * 10; // Convert 0-100 to 0-1000
    agent.reputation.lastUpdated = new Date().toISOString();

    void this.syncToDb(agent);

    void this.auditLogger.logSecurityEvent(
      AuditEventType.SUSPICIOUS_ACTIVITY,
      `Agent action failed: ${reason}`,
      { agentId, newScore: agent.reputation.score },
    );
  }

  /**
   * Record security violation (severely degrades reputation).
   *
   * Called when an agent attempts something malicious (e.g., prompt injection,
   * unauthorized access, policy violation). This is more severe than recordFailure().
   *
   * **Auto-suspend:** After ${VIOLATION_AUTO_SUSPEND_THRESHOLD} violations, agent is suspended.
   *
   * @param agentId - The agent that committed the violation
   * @param violation - Description of the violation (logged for audit)
   */
  async recordViolation(agentId: string, violation: string): Promise<void> {
    const agent = this.agents.get(agentId);
    if (!agent) return;

    // Delegate reputation tracking to TrustService
    const trustScore = await this.trustService.recordPolicyViolation(
      agentId,
      violation,
    );
    agent.reputation.violations++;
    agent.reputation.score = trustScore.score * 10; // Convert 0-100 to 0-1000
    agent.reputation.lastUpdated = new Date().toISOString();

    // Auto-suspend on multiple violations
    if (agent.reputation.violations >= VIOLATION_AUTO_SUSPEND_THRESHOLD) {
      await this.suspendAgent(agentId, `Multiple violations: ${violation}`);
    }

    void this.syncToDb(agent);

    void this.auditLogger.logSecurityEvent(
      AuditEventType.SECURITY_ALERT,
      `Agent security violation: ${violation}`,
      {
        agentId,
        violations: agent.reputation.violations,
        newScore: agent.reputation.score,
      },
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

    void this.auditLogger.logSecurityEvent(
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

    void this.auditLogger.logSecurityEvent(
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
   * Activate global kill switch (stops ALL agents).
   *
   * ## Behavior
   *
   * 1. **Immediate cache update:** `globalKillSwitchCache = true`
   *    - All new requests are blocked IMMEDIATELY (sync check in checkAction)
   *
   * 2. **Database persistence:** Writes to SystemConfigEntity
   *    - Survives service restarts
   *    - Applies to all instances in a cluster
   *
   * 3. **Agent termination:** All ACTIVE agents are set to TERMINATED
   *    - This is permanent; agents cannot be reactivated
   *    - Termination is logged to audit trail
   *
   * ## In-Flight Request Handling
   *
   * **Critical Design Decision:** In-flight requests are NOT interrupted.
   *
   * Rationale:
   * - Interrupting mid-transaction could leave external systems in inconsistent state
   * - Example: Agent is halfway through a bank transfer - interrupting = lost money
   * - The checkAction() gate is synchronous, so new requests are blocked immediately
   * - In-flight requests will complete, then the agent is terminated
   *
   * If you need to interrupt in-flight requests, implement a cancellation token
   * pattern in the calling code.
   *
   * @param reason - Human-readable reason (stored in DB and audit log)
   *
   * @example
   * ```typescript
   * // Emergency shutdown
   * await sandbox.activateGlobalKillSwitch('Breach detected in production');
   *
   * // Check current in-flight requests
   * const inFlight = sandbox.getInFlightStats();
   * console.log(`${inFlight.total} requests still in progress`);
   * ```
   */
  async activateGlobalKillSwitch(reason: string): Promise<void> {
    // STEP 1: Immediately block new requests (sync)
    this.globalKillSwitchCache = true;

    // STEP 2: Persist to database (async, but we await for consistency)
    await this.configRepository.upsert(
      {
        key: 'global_kill_switch',
        value: 'true',
        description: `Activated: ${reason}`,
        valueType: 'boolean',
      },
      ['key'],
    );

    // Get in-flight stats for logging
    const inFlightStats = this.getInFlightStats();

    this.logger.error(`🚨 GLOBAL KILL SWITCH ACTIVATED: ${reason}`);
    this.logger.warn(
      `   In-flight requests: ${inFlightStats.total} (will complete before termination)`,
    );

    await this.auditLogger.logSecurityEvent(
      AuditEventType.KILL_SWITCH_ACTIVATED,
      `GLOBAL KILL SWITCH ACTIVATED: ${reason}`,
      {
        activeAgents: this.agents.size,
        inFlightRequests: inFlightStats.total,
        inFlightByAgent: inFlightStats.byAgent,
      },
    );

    // STEP 3: Terminate all active agents
    // Note: This does NOT cancel in-flight requests - by design
    for (const [, agent] of this.agents) {
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
    const periodEnd = new Date(
      periodStart.getTime() + agent.budget.periodSeconds * 1000,
    );

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
        terminatedAt: agent.terminatedAt
          ? new Date(agent.terminatedAt)
          : undefined,
      });
    } catch (error) {
      this.logger.error(`Failed to sync agent ${agent.id} to DB: ${error}`);
    }
  }

  // ========================================================================
  // IN-FLIGHT REQUEST TRACKING
  // ========================================================================

  /**
   * Increment in-flight counter when checkAction() approves a request.
   * @internal Called automatically by checkAction()
   */
  private incrementInFlight(agentId: string): void {
    const current = this.inFlightRequests.get(agentId) || 0;
    this.inFlightRequests.set(agentId, current + 1);
  }

  /**
   * Decrement in-flight counter when request completes (success or failure).
   * @internal Called automatically by recordSuccess() and recordFailure()
   */
  private decrementInFlight(agentId: string): void {
    const current = this.inFlightRequests.get(agentId) || 0;
    this.inFlightRequests.set(agentId, Math.max(0, current - 1));
  }

  /**
   * Get current in-flight request statistics.
   *
   * Useful for:
   * - Monitoring active load
   * - Graceful shutdown (wait for in-flight to complete)
   * - Debugging kill switch behavior
   *
   * @returns Object with total and per-agent breakdown
   */
  getInFlightStats(): { total: number; byAgent: Record<string, number> } {
    const byAgent: Record<string, number> = {};
    let total = 0;

    for (const [agentId, count] of this.inFlightRequests) {
      if (count > 0) {
        byAgent[agentId] = count;
        total += count;
      }
    }

    return { total, byAgent };
  }

  /**
   * Check if global kill switch is active.
   * @returns true if kill switch is active
   */
  isKillSwitchActive(): boolean {
    return this.globalKillSwitchCache;
  }
}
