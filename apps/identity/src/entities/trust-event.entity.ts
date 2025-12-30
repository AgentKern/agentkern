/**
 * AgentKernIdentity - Trust Event Entity
 *
 * Persistent storage for trust score change events.
 * Each event records a change to an agent's trust score with reason.
 */

import {
  Entity,
  PrimaryGeneratedColumn,
  Column,
  CreateDateColumn,
  Index,
  ManyToOne,
  JoinColumn,
} from 'typeorm';

/**
 * Trust event types that affect an agent's score
 */
export enum TrustEventType {
  TRANSACTION_SUCCESS = 'transaction_success',
  TRANSACTION_FAILURE = 'transaction_failure',
  POLICY_VIOLATION = 'policy_violation',
  PEER_ENDORSEMENT = 'peer_endorsement',
  PEER_REPORT = 'peer_report',
  CREDENTIAL_VERIFIED = 'credential_verified',
  REGISTRATION = 'registration',
}

@Entity('trust_events')
@Index(['agentId', 'timestamp'])
export class TrustEventEntity {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column()
  @Index()
  agentId: string;

  @Column({
    type: 'enum',
    enum: TrustEventType,
  })
  type: TrustEventType;

  /**
   * Change to trust score (positive or negative)
   */
  @Column({ type: 'int', default: 0 })
  delta: number;

  /**
   * Human-readable reason for the trust change
   */
  @Column('text')
  reason: string;

  /**
   * Related agent ID (for peer interactions)
   */
  @Column({ nullable: true })
  relatedAgentId?: string;

  /**
   * Response time in milliseconds (for transaction events)
   */
  @Column({ type: 'int', nullable: true })
  responseTimeMs?: number;

  @CreateDateColumn()
  timestamp: Date;
}

/**
 * Trust Score Entity
 *
 * Current trust score state for an agent.
 * Snapshot of calculated trust with factor weights.
 */
@Entity('trust_scores')
export class TrustScoreEntity {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column({ unique: true })
  @Index()
  agentId: string;

  /**
   * Current trust score (0-100)
   */
  @Column({ type: 'int', default: 50 })
  score: number;

  /**
   * Trust level classification
   */
  @Column({
    type: 'varchar',
    length: 20,
    default: 'medium',
  })
  level: 'untrusted' | 'low' | 'medium' | 'high' | 'verified';

  // Trust factors (stored as individual columns for query efficiency)

  @Column({ type: 'decimal', precision: 5, scale: 2, default: 100 })
  transactionSuccessRate: number;

  @Column({ type: 'int', default: 0 })
  averageResponseTimeMs: number;

  @Column({ type: 'decimal', precision: 5, scale: 2, default: 100 })
  policyComplianceRate: number;

  @Column({ type: 'int', default: 0 })
  peerEndorsementCount: number;

  @Column({ type: 'int', default: 0 })
  accountAgeDays: number;

  @Column({ type: 'int', default: 0 })
  verifiedCredentialCount: number;

  @Column({ type: 'int', default: 0 })
  totalTransactions: number;

  @Column({ type: 'int', default: 0 })
  failedTransactions: number;

  @CreateDateColumn()
  createdAt: Date;

  @Column({ type: 'timestamp', default: () => 'CURRENT_TIMESTAMP' })
  calculatedAt: Date;
}
