/**
 * AgentProof - Audit Event Entity (TypeORM)
 * 
 * PostgreSQL entity for audit logs.
 */

import {
  Entity,
  PrimaryGeneratedColumn,
  Column,
  CreateDateColumn,
  Index,
} from 'typeorm';

export enum AuditEventTypeEnum {
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

@Entity('audit_events')
@Index(['principalId'])
@Index(['agentId'])
@Index(['type'])
@Index(['timestamp'])
export class AuditEventEntity {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @CreateDateColumn()
  timestamp: Date;

  @Column({ type: 'enum', enum: AuditEventTypeEnum })
  type: AuditEventTypeEnum;

  @Column({ nullable: true })
  principalId: string;

  @Column({ nullable: true })
  agentId: string;

  @Column({ nullable: true })
  proofId: string;

  @Column({ nullable: true })
  action: string;

  @Column({ nullable: true })
  target: string;

  @Column({ nullable: true })
  ipAddress: string;

  @Column({ nullable: true })
  userAgent: string;

  @Column({ type: 'boolean' })
  success: boolean;

  @Column({ nullable: true })
  errorMessage: string;

  @Column({ type: 'jsonb', nullable: true })
  metadata: Record<string, unknown>;
}
