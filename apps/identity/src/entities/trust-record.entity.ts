/**
 * AgentKernIdentity - Trust Record Entity (TypeORM)
 *
 * PostgreSQL entity for trust records.
 */

import {
  Entity,
  PrimaryGeneratedColumn,
  Column,
  CreateDateColumn,
  UpdateDateColumn,
  Index,
} from 'typeorm';

@Entity('trust_records')
@Index(['agentId', 'principalId'], { unique: true })
export class TrustRecordEntity {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column()
  @Index()
  agentId: string;

  @Column()
  @Index()
  principalId: string;

  @Column({ type: 'int', default: 500 })
  trustScore: number;

  @Column({ type: 'boolean', default: true })
  trusted: boolean;

  @Column({ type: 'boolean', default: false })
  revoked: boolean;

  @Column({ type: 'int', default: 0 })
  verificationCount: number;

  @Column({ type: 'int', default: 0 })
  failureCount: number;

  @Column({ type: 'timestamp', nullable: true })
  lastVerifiedAt: Date;

  @Column({ type: 'timestamp', nullable: true })
  expiresAt: Date;

  @Column({ type: 'jsonb', nullable: true })
  metadata: {
    agentName?: string;
    agentVersion?: string;
    principalDevice?: string;
  };

  @CreateDateColumn()
  registeredAt: Date;

  @UpdateDateColumn()
  updatedAt: Date;
}
