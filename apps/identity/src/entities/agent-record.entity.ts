import { Entity, Column, PrimaryColumn, CreateDateColumn, UpdateDateColumn } from 'typeorm';
import { AgentStatus, AgentBudget, AgentUsage, AgentReputation } from '../domain/agent.entity';

@Entity('agent_records')
export class AgentRecordEntity {
  @PrimaryColumn()
  id: string;

  @Column()
  name: string;

  @Column()
  version: string;

  @Column({
    type: 'enum',
    enum: AgentStatus,
    default: AgentStatus.ACTIVE,
  })
  status: AgentStatus;

  @Column({ type: 'jsonb' })
  budget: AgentBudget;

  @Column({ type: 'jsonb' })
  usage: AgentUsage;

  @Column({ type: 'jsonb' })
  reputation: AgentReputation;

  @CreateDateColumn()
  createdAt: Date;

  @UpdateDateColumn()
  lastActiveAt: Date;

  @Column({ nullable: true })
  terminatedAt: Date;

  @Column({ nullable: true })
  terminationReason: string;
}
