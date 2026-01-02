/**
 * AgentKern Identity - Gate Policy Entity
 *
 * TypeORM entity for persistent storage of security policies.
 * Policies define rules for prompt filtering, access control, and compliance.
 */

import {
  Entity,
  Column,
  PrimaryColumn,
  CreateDateColumn,
  UpdateDateColumn,
} from 'typeorm';

/**
 * Policy rule actions
 */
export type PolicyAction = 'allow' | 'deny' | 'audit' | 'escalate';

/**
 * Policy rule definition
 */
export interface PolicyRule {
  id: string;
  condition: string;
  action: PolicyAction;
  priority?: number;
  metadata?: Record<string, unknown>;
}

@Entity('gate_policies')
export class GatePolicyEntity {
  @PrimaryColumn('uuid')
  id: string;

  @Column()
  name: string;

  @Column({ type: 'text', nullable: true })
  description?: string;

  @Column({ default: true })
  active: boolean;

  @Column({ type: 'jsonb', default: [] })
  rules: PolicyRule[];

  @Column({ type: 'simple-array', default: '' })
  tags: string[];

  @Column({ nullable: true })
  createdBy?: string;

  @CreateDateColumn()
  createdAt: Date;

  @UpdateDateColumn()
  updatedAt: Date;

  @Column({ default: 0 })
  version: number;
}
