/**
 * AgentKernIdentity - Audit Logger Service
 *
 * Production-ready structured audit logging with TypeORM persistence.
 * Logs all proof verifications, key registrations, and security events.
 *
 * Follows mandate requirements:
 * - Full audit logging to PostgreSQL
 * - Compliance-ready structured logs
 * - Immutable audit trail (append-only)
 */

import { Injectable, Logger, OnModuleInit } from '@nestjs/common';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository, MoreThan, Between, In } from 'typeorm';
import {
  AuditEventEntity,
  AuditEventTypeEnum,
} from '../entities/audit-event.entity';

// Re-export for backwards compatibility
export { AuditEventTypeEnum as AuditEventType };

export interface AuditEvent {
  id: string;
  timestamp: string;
  type: AuditEventTypeEnum;
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
export class AuditLoggerService implements OnModuleInit {
  private readonly logger = new Logger('AuditLogger');

  constructor(
    @InjectRepository(AuditEventEntity)
    private readonly auditRepository: Repository<AuditEventEntity>,
  ) {}

  async onModuleInit(): Promise<void> {
    const count = await this.auditRepository.count();
    this.logger.log(
      `📋 Audit logger initialized with ${count} existing events`,
    );
  }

  /**
   * Log an audit event to the database
   */
  async log(event: Omit<AuditEvent, 'id' | 'timestamp'>): Promise<AuditEvent> {
    const entity = this.auditRepository.create({
      type: event.type,
      principalId: event.principalId,
      agentId: event.agentId,
      proofId: event.proofId,
      action: event.action,
      target: event.target,
      ipAddress: event.ipAddress,
      userAgent: event.userAgent,
      success: event.success,
      errorMessage: event.errorMessage,
      metadata: event.metadata,
    });

    const saved = await this.auditRepository.save(entity);

    const auditEvent: AuditEvent = {
      id: saved.id,
      timestamp: saved.timestamp.toISOString(),
      type: saved.type,
      principalId: saved.principalId,
      agentId: saved.agentId,
      proofId: saved.proofId,
      action: saved.action,
      target: saved.target,
      ipAddress: saved.ipAddress,
      userAgent: saved.userAgent,
      success: saved.success,
      errorMessage: saved.errorMessage,
      metadata: saved.metadata,
    };

    // Also log to console in structured JSON format
    const logLevel = event.success ? 'log' : 'warn';
    this.logger[logLevel](JSON.stringify(auditEvent));

    return auditEvent;
  }

  /**
   * Log a successful proof verification
   */
  async logVerificationSuccess(
    proofId: string,
    principalId: string,
    agentId: string,
    action: string,
    target: string,
    requestContext?: { ipAddress?: string; userAgent?: string },
  ): Promise<AuditEvent> {
    return this.log({
      type: AuditEventTypeEnum.PROOF_VERIFICATION_SUCCESS,
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
  async logVerificationFailure(
    proofId: string | undefined,
    errorMessage: string,
    requestContext?: { ipAddress?: string; userAgent?: string },
  ): Promise<AuditEvent> {
    return this.log({
      type: AuditEventTypeEnum.PROOF_VERIFICATION_FAILURE,
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
  async logSecurityEvent(
    type: AuditEventTypeEnum,
    message: string,
    metadata?: Record<string, unknown>,
    requestContext?: { ipAddress?: string; userAgent?: string },
  ): Promise<AuditEvent> {
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
  async getAuditTrailForPrincipal(
    principalId: string,
    limit = 100,
  ): Promise<AuditEvent[]> {
    const entities = await this.auditRepository.find({
      where: { principalId },
      order: { timestamp: 'DESC' },
      take: limit,
    });

    return entities.map((e) => this.entityToEvent(e));
  }

  /**
   * Get audit trail for a proof
   */
  async getAuditTrailForProof(proofId: string): Promise<AuditEvent[]> {
    const entities = await this.auditRepository.find({
      where: { proofId },
      order: { timestamp: 'DESC' },
    });

    return entities.map((e) => this.entityToEvent(e));
  }

  /**
   * Get all security events (for monitoring/alerting)
   */
  async getSecurityEvents(since?: Date, limit = 100): Promise<AuditEvent[]> {
    const securityTypes = [
      AuditEventTypeEnum.RATE_LIMIT_EXCEEDED,
      AuditEventTypeEnum.INVALID_INPUT,
      AuditEventTypeEnum.SUSPICIOUS_ACTIVITY,
      AuditEventTypeEnum.PROOF_INVALID_SIGNATURE,
      AuditEventTypeEnum.SECURITY_ALERT,
      AuditEventTypeEnum.SANDBOX_VIOLATION,
      AuditEventTypeEnum.KILL_SWITCH_ACTIVATED,
    ];

    const where: Record<string, unknown> = {
      type: In(securityTypes),
    };

    if (since) {
      where.timestamp = MoreThan(since);
    }

    const entities = await this.auditRepository.find({
      where: where as unknown as typeof AuditEventEntity extends {
        new (): infer E;
      }
        ? Partial<E>
        : never,
      order: { timestamp: 'DESC' },
      take: limit,
    });

    return entities.map((e) => this.entityToEvent(e));
  }

  /**
   * Export audit log for compliance (e.g., SOC 2, GDPR requests)
   */
  async exportAuditLog(filters?: {
    startDate?: Date;
    endDate?: Date;
    principalId?: string;
    types?: AuditEventTypeEnum[];
  }): Promise<AuditEvent[]> {
    const where: Record<string, unknown> = {};

    if (filters?.startDate && filters?.endDate) {
      where.timestamp = Between(filters.startDate, filters.endDate);
    } else if (filters?.startDate) {
      where.timestamp = MoreThan(filters.startDate);
    }

    if (filters?.principalId) {
      where.principalId = filters.principalId;
    }

    if (filters?.types) {
      where.type = In(filters.types);
    }

    const entities = await this.auditRepository.find({
      where: where as unknown as typeof AuditEventEntity extends {
        new (): infer E;
      }
        ? Partial<E>
        : never,
      order: { timestamp: 'DESC' },
    });

    return entities.map((e) => this.entityToEvent(e));
  }

  /**
   * Get recent events (for dashboard)
   */
  async getRecentEvents(limit = 100): Promise<AuditEvent[]> {
    const entities = await this.auditRepository.find({
      order: { timestamp: 'DESC' },
      take: limit,
    });

    return entities.map((e) => this.entityToEvent(e));
  }

  /**
   * Convert entity to interface
   */
  private entityToEvent(entity: AuditEventEntity): AuditEvent {
    return {
      id: entity.id,
      timestamp: entity.timestamp.toISOString(),
      type: entity.type,
      principalId: entity.principalId,
      agentId: entity.agentId,
      proofId: entity.proofId,
      action: entity.action,
      target: entity.target,
      ipAddress: entity.ipAddress,
      userAgent: entity.userAgent,
      success: entity.success,
      errorMessage: entity.errorMessage,
      metadata: entity.metadata,
    };
  }
}
