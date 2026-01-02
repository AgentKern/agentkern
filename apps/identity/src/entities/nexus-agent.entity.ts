/**
 * AgentKern Identity - Nexus Agent Entity
 *
 * TypeORM entity for persistent storage of registered agents in the Nexus mesh.
 * Replaces the in-memory Map for production deployments.
 */

import {
  Entity,
  Column,
  PrimaryColumn,
  CreateDateColumn,
  UpdateDateColumn,
} from 'typeorm';
import { Skill, Capability } from '../dto/nexus.dto';

@Entity('nexus_agents')
export class NexusAgentEntity {
  @PrimaryColumn('uuid')
  id: string;

  @Column()
  name: string;

  @Column({ type: 'text', default: '' })
  description: string;

  @Column()
  url: string;

  @Column({ default: '1.0.0' })
  version: string;

  @Column({ type: 'jsonb', default: [] })
  capabilities: Capability[];

  @Column({ type: 'jsonb', default: [] })
  skills: Skill[];

  @Column({ type: 'simple-array', default: '' })
  protocols: string[];

  @CreateDateColumn()
  registeredAt: Date;

  @UpdateDateColumn()
  updatedAt: Date;

  @Column({ default: true })
  active: boolean;

  @Column({ nullable: true })
  discoveredFrom?: string;
}
