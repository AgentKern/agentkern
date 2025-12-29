/**
 * AgentKern SDK - Ports (Hexagonal Architecture)
 * 
 * Per MANIFESTO Part I: Control Plane uses Hexagonal (Ports & Adapters).
 * These are the abstract interfaces that define how the SDK interacts with external systems.
 */

import type {
  AgentIdentity,
  LiabilityProof,
  Policy,
  VerificationResult,
  AgentState,
  IntentPath,
  CoordinationRequest,
  CoordinationResult,
  BusinessLock,
} from './types';

// ============================================================================
// IDENTITY PORT (AgentKern-Identity)
// ============================================================================

/**
 * Port for Identity operations - The "Passport" interface.
 */
export interface IdentityPort {
  /**
   * Register a new agent identity.
   */
  register(name: string, capabilities?: string[]): Promise<AgentIdentity>;

  /**
   * Retrieve an agent identity by ID.
   */
  getIdentity(agentId: string): Promise<AgentIdentity | null>;

  /**
   * Sign an action to create a liability proof.
   */
  signAction(agentId: string, action: string, payload: unknown): Promise<LiabilityProof>;

  /**
   * Verify a liability proof signature.
   */
  verifyProof(proof: LiabilityProof): Promise<boolean>;

  /**
   * Get the current agent's trust score.
   */
  getTrustScore(agentId: string): Promise<number>;
}

// ============================================================================
// GATE PORT (AgentKern-Gate) - Logic & Guardrails
// ============================================================================

/**
 * Port for Gate operations - The "Guardrails" interface.
 * Per ENGINEERING_STANDARD.md: Neuro-Symbolic Guards.
 */
export interface GatePort {
  /**
   * Verify if an action is allowed by current policies.
   */
  verify(agentId: string, action: string, context?: Record<string, unknown>): Promise<VerificationResult>;

  /**
   * Register a new policy.
   */
  registerPolicy(policy: Policy): Promise<void>;

  /**
   * Get all active policies.
   */
  getPolicies(): Promise<Policy[]>;

  /**
   * Check if a specific policy allows an action.
   */
  checkPolicy(policyId: string, action: string, context?: Record<string, unknown>): Promise<boolean>;
}

// ============================================================================
// SYNAPSE PORT (AgentKern-Synapse) - Memory & State
// ============================================================================

/**
 * Port for Synapse operations - The "Memory" interface.
 * Per ARCHITECTURE.md: Uses CRDTs for eventual consistency.
 */
export interface SynapsePort {
  /**
   * Get the current state for an agent.
   */
  getState(agentId: string): Promise<AgentState | null>;

  /**
   * Update the state for an agent (CRDT merge).
   */
  setState(agentId: string, state: Record<string, unknown>): Promise<AgentState>;

  /**
   * Start a new intent path.
   */
  startIntent(agentId: string, intent: string, expectedSteps: number): Promise<IntentPath>;

  /**
   * Record a step in the current intent path.
   */
  recordStep(agentId: string, action: string, result?: string): Promise<IntentPath>;

  /**
   * Check if the agent has drifted from their original intent.
   */
  checkDrift(agentId: string): Promise<{ drifted: boolean; score: number }>;
}

// ============================================================================
// ARBITER PORT (AgentKern-Arbiter) - Coordination & Traffic Control
// ============================================================================

/**
 * Port for Arbiter operations - The "Traffic Control" interface.
 * Per ARCHITECTURE.md: Implements Atomic Business Locks via Raft consensus.
 */
export interface ArbiterPort {
  /**
   * Request coordination for a resource.
   */
  requestCoordination(request: CoordinationRequest): Promise<CoordinationResult>;

  /**
   * Acquire a lock on a resource.
   */
  acquireLock(agentId: string, resource: string, priority?: number): Promise<BusinessLock | null>;

  /**
   * Release a lock on a resource.
   */
  releaseLock(agentId: string, resource: string): Promise<boolean>;

  /**
   * Get the current lock status for a resource.
   */
  getLockStatus(resource: string): Promise<BusinessLock | null>;

  /**
   * Get the queue position for a pending coordination request.
   */
  getQueuePosition(agentId: string, resource: string): Promise<number>;
}

// ============================================================================
// SOVEREIGN PORT (AgentKern-Sovereign) - Data Residency
// ============================================================================

/**
 * Port for Sovereign operations - Data Residency Controller.
 * Per GLOBAL_GAPS.md: Handles PIPL/GDPR/data localization.
 */
export interface SovereignPort {
  /**
   * Check if data can be transferred between regions.
   */
  canTransfer(fromRegion: string, toRegion: string, dataType: string): Promise<boolean>;

  /**
   * Get the required data residency for a specific operation.
   */
  getRequiredResidency(operation: string, jurisdiction: string): Promise<string>;

  /**
   * Validate that an operation complies with regional regulations.
   */
  validateCompliance(operation: string, data: unknown, jurisdiction: string): Promise<{ compliant: boolean; violations?: string[] }>;
}

// ============================================================================
// TREASURY PORT (AgentKern-Treasury) - Agent Payments
// ============================================================================

/**
 * Port for Treasury operations - The "Bank" interface.
 * Per MANIFESTO.md: Agents can pay each other for services.
 */
export interface TreasuryPort {
  /**
   * Get balance for an agent.
   */
  getBalance(agentId: string): Promise<{ balance: number; currency: string; pending: number }>;

  /**
   * Transfer funds between agents.
   */
  transfer(
    from: string,
    to: string,
    amount: number,
    reference?: string
  ): Promise<{ transactionId: string; status: 'completed' | 'pending' | 'failed' }>;

  /**
   * Set spending limit for an agent.
   */
  setSpendingLimit(
    agentId: string,
    limit: number,
    period: 'transaction' | 'hourly' | 'daily' | 'weekly' | 'monthly'
  ): Promise<void>;

  /**
   * Get remaining budget for an agent.
   */
  getRemainingBudget(agentId: string): Promise<{ remaining: number; period: string }>;

  /**
   * Check if agent can spend amount (pre-flight check).
   */
  canSpend(agentId: string, amount: number): Promise<boolean>;
}
