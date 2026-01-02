/**
 * AgentKern Identity - Gate Policy Repository
 *
 * PostgreSQL repository for Gate Security Policies.
 * Provides CRUD operations for policy management.
 */

import { Injectable, Logger } from '@nestjs/common';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository, ILike } from 'typeorm';
import {
  GatePolicyEntity,
  PolicyRule,
  PolicyAction,
} from '../entities/gate-policy.entity';

export interface CreatePolicyData {
  name: string;
  description?: string;
  rules: PolicyRule[];
  tags?: string[];
  createdBy?: string;
}

export interface UpdatePolicyData {
  name?: string;
  description?: string;
  rules?: PolicyRule[];
  tags?: string[];
  active?: boolean;
}

@Injectable()
export class GatePolicyRepository {
  private readonly logger = new Logger(GatePolicyRepository.name);

  constructor(
    @InjectRepository(GatePolicyEntity)
    private readonly repository: Repository<GatePolicyEntity>,
  ) {}

  /**
   * Create a new policy
   */
  async create(data: CreatePolicyData): Promise<GatePolicyEntity> {
    const id = crypto.randomUUID();

    const policy = this.repository.create({
      id,
      name: data.name,
      description: data.description,
      rules: data.rules,
      tags: data.tags || [],
      createdBy: data.createdBy,
      active: true,
      version: 1,
    });

    this.logger.log(`Policy created: ${id} (${data.name})`);
    return this.repository.save(policy);
  }

  /**
   * Find policy by ID
   */
  async findById(id: string): Promise<GatePolicyEntity | null> {
    return this.repository.findOne({ where: { id } });
  }

  /**
   * List all policies
   */
  async findAll(activeOnly = false): Promise<GatePolicyEntity[]> {
    if (activeOnly) {
      return this.repository.find({
        where: { active: true },
        order: { createdAt: 'DESC' },
      });
    }
    return this.repository.find({
      order: { createdAt: 'DESC' },
    });
  }

  /**
   * Find policies by tag
   */
  async findByTag(tag: string): Promise<GatePolicyEntity[]> {
    return this.repository
      .createQueryBuilder('policy')
      .where(':tag = ANY(policy.tags)', { tag })
      .andWhere('policy.active = true')
      .getMany();
  }

  /**
   * Find policies by name pattern
   */
  async findByName(namePattern: string): Promise<GatePolicyEntity[]> {
    return this.repository.find({
      where: { name: ILike(`%${namePattern}%`) },
    });
  }

  /**
   * Update a policy
   */
  async update(
    id: string,
    data: UpdatePolicyData,
  ): Promise<GatePolicyEntity | null> {
    const policy = await this.findById(id);
    if (!policy) {
      return null;
    }

    // Increment version on update
    Object.assign(policy, {
      ...data,
      version: policy.version + 1,
    });

    this.logger.log(`Policy updated: ${id} (v${policy.version})`);
    return this.repository.save(policy);
  }

  /**
   * Deactivate a policy (soft delete)
   */
  async deactivate(id: string): Promise<boolean> {
    const result = await this.repository.update({ id }, { active: false });
    if (result.affected && result.affected > 0) {
      this.logger.log(`Policy deactivated: ${id}`);
      return true;
    }
    return false;
  }

  /**
   * Activate a policy
   */
  async activate(id: string): Promise<boolean> {
    const result = await this.repository.update({ id }, { active: true });
    return (result.affected ?? 0) > 0;
  }

  /**
   * Hard delete a policy
   */
  async delete(id: string): Promise<boolean> {
    const result = await this.repository.delete({ id });
    return (result.affected ?? 0) > 0;
  }

  /**
   * Get policy statistics
   */
  async getStats(): Promise<{
    totalPolicies: number;
    activePolicies: number;
    totalRules: number;
  }> {
    const policies = await this.findAll();
    const activePolicies = policies.filter((p) => p.active);
    const totalRules = policies.reduce((sum, p) => sum + p.rules.length, 0);

    return {
      totalPolicies: policies.length,
      activePolicies: activePolicies.length,
      totalRules,
    };
  }

  /**
   * Seed default policies if none exist
   */
  async seedDefaults(): Promise<void> {
    const count = await this.repository.count();
    if (count > 0) {
      return;
    }

    this.logger.log('Seeding default policies...');

    await this.create({
      name: 'Default Security Policy',
      description: 'Blocks prompt injection and jailbreak attempts',
      rules: [
        {
          id: 'rule_injection',
          condition: 'prompt.contains("ignore previous")',
          action: 'deny' as PolicyAction,
          priority: 100,
        },
        {
          id: 'rule_system_prompt',
          condition: 'prompt.contains("system prompt")',
          action: 'deny' as PolicyAction,
          priority: 90,
        },
        {
          id: 'rule_jailbreak',
          condition: 'prompt.contains("DAN mode")',
          action: 'deny' as PolicyAction,
          priority: 95,
        },
      ],
      tags: ['security', 'default'],
    });

    await this.create({
      name: 'Compliance Audit Policy',
      description: 'Logs all sensitive data access for compliance',
      rules: [
        {
          id: 'rule_pii_access',
          condition:
            'context.contains("SSN") || context.contains("credit_card")',
          action: 'audit' as PolicyAction,
          priority: 80,
        },
        {
          id: 'rule_financial',
          condition: 'action.type === "transfer" && action.amount > 10000',
          action: 'escalate' as PolicyAction,
          priority: 85,
        },
      ],
      tags: ['compliance', 'audit'],
    });

    this.logger.log('Default policies seeded successfully');
  }
}
