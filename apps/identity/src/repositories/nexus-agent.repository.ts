/**
 * AgentKern Identity - Nexus Agent Repository
 *
 * PostgreSQL repository for Nexus Agent Registry.
 * Provides persistent storage for agent cards in the mesh.
 */

import { Injectable, Logger } from '@nestjs/common';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository, ILike } from 'typeorm';
import { NexusAgentEntity } from '../entities/nexus-agent.entity';
import { Skill, Capability } from '../dto/nexus.dto';

export interface RegisterAgentData {
  id?: string;
  name: string;
  description?: string;
  url: string;
  version?: string;
  capabilities?: Capability[];
  skills?: Skill[];
  protocols?: string[];
  discoveredFrom?: string;
}

@Injectable()
export class NexusAgentRepository {
  private readonly logger = new Logger(NexusAgentRepository.name);

  constructor(
    @InjectRepository(NexusAgentEntity)
    private readonly repository: Repository<NexusAgentEntity>,
  ) {}

  /**
   * Register a new agent or update existing
   */
  async register(data: RegisterAgentData): Promise<NexusAgentEntity> {
    const id = data.id || crypto.randomUUID();

    const existing = await this.repository.findOne({ where: { id } });

    if (existing) {
      // Update existing agent
      Object.assign(existing, {
        name: data.name,
        description: data.description || existing.description,
        url: data.url,
        version: data.version || existing.version,
        capabilities: data.capabilities || existing.capabilities,
        skills: data.skills || existing.skills,
        protocols: data.protocols || existing.protocols,
        active: true,
      });
      this.logger.log(`Agent updated: ${id} (${data.name})`);
      return this.repository.save(existing);
    }

    // Create new agent
    const agent = this.repository.create({
      id,
      name: data.name,
      description: data.description || '',
      url: data.url,
      version: data.version || '1.0.0',
      capabilities: data.capabilities || [],
      skills: data.skills || [],
      protocols: data.protocols || ['agentkern'],
      active: true,
      discoveredFrom: data.discoveredFrom,
    });

    this.logger.log(`Agent registered: ${id} (${data.name})`);
    return this.repository.save(agent);
  }

  /**
   * Find agent by ID
   */
  async findById(id: string): Promise<NexusAgentEntity | null> {
    return this.repository.findOne({
      where: { id, active: true },
    });
  }

  /**
   * List all active agents
   */
  async findAll(): Promise<NexusAgentEntity[]> {
    return this.repository.find({
      where: { active: true },
      order: { registeredAt: 'DESC' },
    });
  }

  /**
   * Find agents by skill
   * Uses JSONB containment query for PostgreSQL
   */
  async findBySkill(skill: string): Promise<NexusAgentEntity[]> {
    // Use raw query for JSONB skill matching
    return this.repository
      .createQueryBuilder('agent')
      .where('agent.active = true')
      .andWhere(
        `EXISTS (
          SELECT 1 FROM jsonb_array_elements(agent.skills) AS s 
          WHERE s->>'id' = :skill 
             OR LOWER(s->>'name') LIKE LOWER(:skillPattern)
             OR s->'tags' ? :skill
        )`,
        { skill, skillPattern: `%${skill}%` },
      )
      .getMany();
  }

  /**
   * Find agents by name pattern
   */
  async findByName(namePattern: string): Promise<NexusAgentEntity[]> {
    return this.repository.find({
      where: {
        name: ILike(`%${namePattern}%`),
        active: true,
      },
    });
  }

  /**
   * Unregister an agent (soft delete)
   */
  async unregister(id: string): Promise<boolean> {
    const result = await this.repository.update({ id }, { active: false });

    if (result.affected && result.affected > 0) {
      this.logger.log(`Agent unregistered: ${id}`);
      return true;
    }
    return false;
  }

  /**
   * Hard delete an agent (for cleanup)
   */
  async delete(id: string): Promise<boolean> {
    const result = await this.repository.delete({ id });
    return (result.affected ?? 0) > 0;
  }

  /**
   * Get registry statistics
   */
  async getStats(): Promise<{
    totalAgents: number;
    activeAgents: number;
    inactiveAgents: number;
  }> {
    const [active, total] = await Promise.all([
      this.repository.count({ where: { active: true } }),
      this.repository.count(),
    ]);

    return {
      totalAgents: total,
      activeAgents: active,
      inactiveAgents: total - active,
    };
  }

  /**
   * Check if agent exists
   */
  async exists(id: string): Promise<boolean> {
    const count = await this.repository.count({ where: { id } });
    return count > 0;
  }
}
