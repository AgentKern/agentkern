/**
 * AgentKernIdentity - System Config Entity
 *
 * Persistent storage for system-wide configuration values.
 * Used for critical state like kill switch, feature flags, etc.
 */

import {
  Entity,
  PrimaryColumn,
  Column,
  CreateDateColumn,
  UpdateDateColumn,
} from 'typeorm';

@Entity('system_config')
export class SystemConfigEntity {
  @PrimaryColumn()
  key: string;

  @Column('text')
  value: string;

  @Column({ nullable: true })
  description: string;

  @Column({ default: 'string' })
  valueType: 'string' | 'number' | 'boolean' | 'json';

  @CreateDateColumn()
  createdAt: Date;

  @UpdateDateColumn()
  updatedAt: Date;
}
