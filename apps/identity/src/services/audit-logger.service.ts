/**
 * AgentKern Identity - Audit Logger Service
 * 
 * Structured audit logging for compliance and security.
 * Logs all proof verifications, key registrations, and security events.
 * 
 * Follows mandate requirements:
 * - Full audit logging
 * - Compliance-ready structured logs
 * - Immutable audit trail
 */

import { Injectable, Logger } from '@nestjs/common';

export enum AuditEventType {
  PROOF_VERIFICATION_SUCCESS = 'proof.verification.success',
  PROOF_VERIFICATION_FAILURE = 'proof.verification.failure',
  PROOF_EXPIRED = 'proof.expired',
  PROOF_INVALID_SIGNATURE = 'proof.invalid_signature',
  KEY_REGISTERED = 'key.registered',
  KEY_REVOKED = 'key.revoked',
  RATE_LIMIT_EXCEEDED = 'security.rate_limit_exceeded',
  INVALID_INPUT = 'security.invalid_input',
  SUSPICIOUS_ACTIVITY = 'security.suspicious_activity',
  SECURITY_ALERT = 'security.alert',
  SANDBOX_VIOLATION = 'security.sandbox_violation',
  KILL_SWITCH_ACTIVATED = 'security.kill_switch_activated',
  PQC_DOWNGRADE_ATTEMPT = 'security.pqc_downgrade',
  CRYPTO_ROTATION = 'security.crypto_rotation',
  COMPLIANCE_ATTACHMENT_ADDED = 'compliance.attachment_added',
  COMPLIANCE_REPORT_GENERATED = 'compliance.report_generated',
  AI_RISK_ASSESSMENT = 'ai.risk_assessment',
  BIAS_AUDIT_COMPLETED = 'ai.bias_audit_completed',
}

export interface AuditEvent {
  id: string;
  timestamp: string;
  type: AuditEventType;
  principalId?: string;
  agentId?: string;
  proofId?: string;
  action?: string;
  target?: string;
  ipAddress?: string;
  userAgent?: string;
  success: boolean;
  errorMessage?: string;
  metadata?: Record<string, unknown>;
}

@Injectable()
export class AuditLoggerService {
  private readonly logger = new Logger('AuditLogger');
  
  // In production, this would write to a persistent, immutable store
  // (e.g., append-only database, blockchain, or secure log aggregator)
  private auditLog: AuditEvent[] = [];

  /**
   * Log an audit event
   */
  log(event: Omit<AuditEvent, 'id' | 'timestamp'>): AuditEvent {
    const auditEvent: AuditEvent = {
      id: crypto.randomUUID(),
      timestamp: new Date().toISOString(),
      ...event,
    };

    // Store in memory (replace with persistent storage in production)
    this.auditLog.push(auditEvent);

    // Also log to console in structured JSON format
    const logLevel = event.success ? 'log' : 'warn';
    this.logger[logLevel](JSON.stringify(auditEvent));

    return auditEvent;
  }

  /**
   * Log a successful proof verification
   */
  logVerificationSuccess(
    proofId: string,
    principalId: string,
    agentId: string,
    action: string,
    target: string,
    requestContext?: { ipAddress?: string; userAgent?: string },
  ): AuditEvent {
    return this.log({
      type: AuditEventType.PROOF_VERIFICATION_SUCCESS,
      proofId,
      principalId,
      agentId,
      action,
      target,
      success: true,
      ipAddress: requestContext?.ipAddress,
      userAgent: requestContext?.userAgent,
    });
  }

  /**
   * Log a failed proof verification
   */
  logVerificationFailure(
    proofId: string | undefined,
    errorMessage: string,
    requestContext?: { ipAddress?: string; userAgent?: string },
  ): AuditEvent {
    return this.log({
      type: AuditEventType.PROOF_VERIFICATION_FAILURE,
      proofId,
      success: false,
      errorMessage,
      ipAddress: requestContext?.ipAddress,
      userAgent: requestContext?.userAgent,
    });
  }

  /**
   * Log a security event
   */
  logSecurityEvent(
    type: AuditEventType,
    message: string,
    metadata?: Record<string, unknown>,
    requestContext?: { ipAddress?: string; userAgent?: string },
  ): AuditEvent {
    return this.log({
      type,
      success: false,
      errorMessage: message,
      metadata,
      ipAddress: requestContext?.ipAddress,
      userAgent: requestContext?.userAgent,
    });
  }

  /**
   * Get audit trail for a principal
   */
  getAuditTrailForPrincipal(principalId: string, limit = 100): AuditEvent[] {
    return this.auditLog
      .filter((event) => event.principalId === principalId)
      .slice(-limit)
      .reverse();
  }

  /**
   * Get audit trail for a proof
   */
  getAuditTrailForProof(proofId: string): AuditEvent[] {
    return this.auditLog.filter((event) => event.proofId === proofId);
  }

  /**
   * Get all security events (for monitoring/alerting)
   */
  getSecurityEvents(since?: Date, limit = 100): AuditEvent[] {
    const securityTypes = [
      AuditEventType.RATE_LIMIT_EXCEEDED,
      AuditEventType.INVALID_INPUT,
      AuditEventType.SUSPICIOUS_ACTIVITY,
      AuditEventType.PROOF_INVALID_SIGNATURE,
    ];

    return this.auditLog
      .filter((event) => {
        if (!securityTypes.includes(event.type)) return false;
        if (since && new Date(event.timestamp) < since) return false;
        return true;
      })
      .slice(-limit)
      .reverse();
  }

  /**
   * Export audit log for compliance (e.g., SOC 2, GDPR requests)
   */
  exportAuditLog(
    filters?: {
      startDate?: Date;
      endDate?: Date;
      principalId?: string;
      types?: AuditEventType[];
    },
  ): AuditEvent[] {
    return this.auditLog.filter((event) => {
      const eventDate = new Date(event.timestamp);
      
      if (filters?.startDate && eventDate < filters.startDate) return false;
      if (filters?.endDate && eventDate > filters.endDate) return false;
      if (filters?.principalId && event.principalId !== filters.principalId) return false;
      if (filters?.types && !filters.types.includes(event.type)) return false;
      
      return true;
    });
  }

  /**
   * Get recent events (for dashboard)
   */
  getRecentEvents(limit = 100): AuditEvent[] {
    return this.auditLog.slice(-limit).reverse();
  }
}

