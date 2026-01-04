/**
 * AgentKern Custom Exceptions
 * 
 * Typed exceptions for better error handling, observability, and API responses.
 * Use these instead of generic `throw new Error()`.
 */
import { HttpException, HttpStatus } from '@nestjs/common';

// ============================================================================
// Base Exception
// ============================================================================

/**
 * Base exception for all AgentKern errors.
 * Provides structured error codes and metadata.
 */
export class AgentKernException extends HttpException {
  constructor(
    public readonly code: string,
    message: string,
    status: HttpStatus = HttpStatus.INTERNAL_SERVER_ERROR,
    public readonly metadata?: Record<string, unknown>,
  ) {
    super(
      {
        code,
        message,
        timestamp: new Date().toISOString(),
        ...metadata,
      },
      status,
    );
  }
}

// ============================================================================
// Authentication & Authorization
// ============================================================================

export class LiabilityProofMissingException extends AgentKernException {
  constructor(path?: string) {
    super(
      'AUTH_PROOF_MISSING',
      'X-AgentKernIdentity header is required',
      HttpStatus.UNAUTHORIZED,
      { path },
    );
  }
}

export class LiabilityProofInvalidException extends AgentKernException {
  constructor(reason: string) {
    super(
      'AUTH_PROOF_INVALID',
      `Invalid liability proof: ${reason}`,
      HttpStatus.UNAUTHORIZED,
      { reason },
    );
  }
}

export class LiabilityProofExpiredException extends AgentKernException {
  constructor(expiredAt: Date) {
    super(
      'AUTH_PROOF_EXPIRED',
      'Liability proof has expired',
      HttpStatus.UNAUTHORIZED,
      { expired_at: expiredAt.toISOString() },
    );
  }
}

export class InsufficientPermissionsException extends AgentKernException {
  constructor(requiredAction: string, agentId?: string) {
    super(
      'AUTH_INSUFFICIENT_PERMISSIONS',
      `Insufficient permissions for action: ${requiredAction}`,
      HttpStatus.FORBIDDEN,
      { required_action: requiredAction, agent_id: agentId },
    );
  }
}

// ============================================================================
// Bridge Errors
// ============================================================================

export class BridgeNotLoadedException extends AgentKernException {
  constructor(pillar: string) {
    super(
      'BRIDGE_NOT_LOADED',
      `Native bridge not loaded. ${pillar} operations unavailable.`,
      HttpStatus.SERVICE_UNAVAILABLE,
      { pillar },
    );
  }
}

export class BridgeCallFailedException extends AgentKernException {
  constructor(fn: string, reason: string) {
    super(
      'BRIDGE_CALL_FAILED',
      `Bridge call failed: ${fn}`,
      HttpStatus.INTERNAL_SERVER_ERROR,
      { function: fn, reason },
    );
  }
}

// ============================================================================
// Treasury Errors
// ============================================================================

export class InsufficientFundsException extends AgentKernException {
  constructor(agentId: string, required: string, available: string) {
    super(
      'TREASURY_INSUFFICIENT_FUNDS',
      `Insufficient funds for transfer`,
      HttpStatus.PAYMENT_REQUIRED,
      { agent_id: agentId, required, available },
    );
  }
}

export class BudgetExceededException extends AgentKernException {
  constructor(agentId: string, limit: string, attempted: string) {
    super(
      'TREASURY_BUDGET_EXCEEDED',
      `Transaction would exceed budget limit`,
      HttpStatus.FORBIDDEN,
      { agent_id: agentId, limit, attempted },
    );
  }
}

export class InvalidTransferException extends AgentKernException {
  constructor(reason: string) {
    super(
      'TREASURY_INVALID_TRANSFER',
      `Invalid transfer: ${reason}`,
      HttpStatus.BAD_REQUEST,
      { reason },
    );
  }
}

// ============================================================================
// Nexus Errors
// ============================================================================

export class AgentNotRegisteredException extends AgentKernException {
  constructor(agentId: string) {
    super(
      'NEXUS_AGENT_NOT_REGISTERED',
      `Agent not registered in Nexus`,
      HttpStatus.NOT_FOUND,
      { agent_id: agentId },
    );
  }
}

export class RoutingFailedException extends AgentKernException {
  constructor(from: string, to: string, reason: string) {
    super(
      'NEXUS_ROUTING_FAILED',
      `Could not route message from ${from} to ${to}`,
      HttpStatus.BAD_GATEWAY,
      { from, to, reason },
    );
  }
}

export class ProtocolMismatchException extends AgentKernException {
  constructor(expected: string, received: string) {
    super(
      'NEXUS_PROTOCOL_MISMATCH',
      `Protocol mismatch: expected ${expected}, received ${received}`,
      HttpStatus.BAD_REQUEST,
      { expected, received },
    );
  }
}

// ============================================================================
// Arbiter Errors
// ============================================================================

export class KillSwitchActiveException extends AgentKernException {
  constructor(reason: string, activatedBy: string) {
    super(
      'ARBITER_KILL_SWITCH_ACTIVE',
      `Kill switch is active: ${reason}`,
      HttpStatus.SERVICE_UNAVAILABLE,
      { reason, activated_by: activatedBy },
    );
  }
}

export class ChaosInjectionDisabledException extends AgentKernException {
  constructor() {
    super(
      'ARBITER_CHAOS_DISABLED',
      'Chaos injection is disabled in production',
      HttpStatus.FORBIDDEN,
      { environment: process.env.NODE_ENV },
    );
  }
}

// ============================================================================
// Gate Errors
// ============================================================================

export class PolicyViolationException extends AgentKernException {
  constructor(policyId: string, reason: string) {
    super(
      'GATE_POLICY_VIOLATION',
      `Action blocked by policy: ${policyId}`,
      HttpStatus.FORBIDDEN,
      { policy_id: policyId, reason },
    );
  }
}

export class PromptInjectionDetectedException extends AgentKernException {
  constructor(confidence: number) {
    super(
      'GATE_PROMPT_INJECTION',
      'Potential prompt injection detected',
      HttpStatus.FORBIDDEN,
      { confidence },
    );
  }
}

// ============================================================================
// Synapse Errors
// ============================================================================

export class MemoryQuotaExceededException extends AgentKernException {
  constructor(agentId: string, limit: number, used: number) {
    super(
      'SYNAPSE_QUOTA_EXCEEDED',
      'Memory storage quota exceeded',
      HttpStatus.PAYLOAD_TOO_LARGE,
      { agent_id: agentId, limit_bytes: limit, used_bytes: used },
    );
  }
}

// ============================================================================
// Validation Errors
// ============================================================================

export class ValidationException extends AgentKernException {
  constructor(field: string, issue: string) {
    super(
      'VALIDATION_FAILED',
      `Validation failed for ${field}: ${issue}`,
      HttpStatus.BAD_REQUEST,
      { field, issue },
    );
  }
}

export class InvalidAgentIdException extends AgentKernException {
  constructor(agentId: string) {
    super(
      'VALIDATION_INVALID_AGENT_ID',
      `Invalid agent ID format: ${agentId}`,
      HttpStatus.BAD_REQUEST,
      { agent_id: agentId, expected_format: 'did:key:z...' },
    );
  }
}
