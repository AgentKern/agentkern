/**
 * AgentKern Identity - Policy Entity (TypeORM)
 * 
 * PostgreSQL entity for enterprise policies.
 */

import {
  Entity,
  PrimaryGeneratedColumn,
  Column,
  CreateDateColumn,
  UpdateDateColumn,
} from 'typeorm';

export enum PolicyActionEnum {
  ALLOW = 'ALLOW',
  DENY = 'DENY',
  REQUIRE_CONFIRMATION = 'REQUIRE_CONFIRMATION',
  RATE_LIMIT = 'RATE_LIMIT',
}

export interface PolicyRule {
  name: string;
  condition: string;
  action: PolicyActionEnum;
  rateLimit?: number;
}

@Entity('policies')
export class PolicyEntity {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column()
  name: string;

  @Column({ type: 'text' })
  description: string;

  @Column({ type: 'jsonb' })
  rules: PolicyRule[];

  @Column({ type: 'simple-array', nullable: true })
  targetAgents: string[];

  @Column({ type: 'simple-array', nullable: true })
  targetPrincipals: string[];

  @Column({ type: 'boolean', default: true })
  active: boolean;

  @CreateDateColumn()
  createdAt: Date;

  @UpdateDateColumn()
  updatedAt: Date;
}
