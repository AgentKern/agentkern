/**
 * AgentKern Gateway - Nexus Service
 *
 * Business logic for protocol translation and agent discovery.
 * Now uses TypeORM-backed persistent storage instead of in-memory Map.
 */

import { Injectable, Logger } from '@nestjs/common';
import {
  RegisterAgentDto,
  RouteTaskDto,
  TranslateMessageDto,
  AgentCard,
  NexusMessage,
} from '../dto/nexus.dto';
import { NexusAgentRepository } from '../repositories/nexus-agent.repository';
import { NexusAgentEntity } from '../entities/nexus-agent.entity';
import { ProtocolTranslator } from './protocol-translator';

@Injectable()
export class NexusService {
  private readonly logger = new Logger(NexusService.name);

  constructor(private readonly agentRepository: NexusAgentRepository) {}

  /**
   * Convert entity to AgentCard DTO
   */
  private toAgentCard(entity: NexusAgentEntity): AgentCard {
    return {
      id: entity.id,
      name: entity.name,
      description: entity.description,
      url: entity.url,
      version: entity.version,
      capabilities: entity.capabilities,
      skills: entity.skills,
      protocols: entity.protocols,
      registeredAt: entity.registeredAt.toISOString(),
    };
  }

  /**
   * Register an agent (persisted to database).
   */
  async registerAgent(dto: RegisterAgentDto): Promise<AgentCard> {
    const entity = await this.agentRepository.register({
      id: dto.id,
      name: dto.name,
      description: dto.description,
      url: dto.url,
      version: dto.version,
      capabilities: dto.capabilities,
      skills: dto.skills,
      protocols: dto.protocols,
    });

    return this.toAgentCard(entity);
  }

  /**
   * List all agents (from database).
   */
  async listAgents(): Promise<AgentCard[]> {
    const entities = await this.agentRepository.findAll();
    return entities.map((e) => this.toAgentCard(e));
  }

  /**
   * Get agent by ID (from database).
   */
  async getAgent(id: string): Promise<AgentCard | null> {
    const entity = await this.agentRepository.findById(id);
    return entity ? this.toAgentCard(entity) : null;
  }

  /**
   * Unregister an agent (soft delete in database).
   */
  async unregisterAgent(id: string): Promise<boolean> {
    return this.agentRepository.unregister(id);
  }

  /**
   * Find agents by skill (JSONB search in database).
   */
  async findAgentsBySkill(skill: string): Promise<AgentCard[]> {
    const entities = await this.agentRepository.findBySkill(skill);
    return entities.map((e) => this.toAgentCard(e));
  }

  /**
   * Discover agent from URL (fetches /.well-known/agent.json).
   * This is genuinely async due to network I/O.
   */
  async discoverAgent(url: string): Promise<AgentCard> {
    const wellKnownUrl = `${url.replace(/\/$/, '')}/.well-known/agent.json`;

    this.logger.log(`Discovering agent from: ${wellKnownUrl}`);

    try {
      const response = await fetch(wellKnownUrl);

      if (!response.ok) {
        throw new Error(`Failed to fetch agent card: ${response.status}`);
      }

      const card = (await response.json()) as AgentCard;

      // Register the discovered agent with source tracking
      const entity = await this.agentRepository.register({
        id: card.id,
        name: card.name,
        description: card.description,
        url: card.url,
        version: card.version,
        capabilities: card.capabilities,
        skills: card.skills,
        protocols: card.protocols,
        discoveredFrom: url,
      });

      this.logger.log(`Agent discovered and registered: ${entity.id}`);
      return this.toAgentCard(entity);
    } catch (error) {
      this.logger.error(`Discovery failed for ${url}: ${error}`);
      throw error;
    }
  }

  /**
   * Route task to best matching agent (queries database).
   */
  async routeTask(
    dto: RouteTaskDto,
  ): Promise<(AgentCard & { matchScore: number }) | null> {
    const agents = await this.agentRepository.findAll();

    if (agents.length === 0) {
      return null;
    }

    // Score each agent
    const scored = agents.map((agent) => ({
      ...this.toAgentCard(agent),
      matchScore: this.calculateMatchScore(agent, dto),
    }));

    // Sort by score descending
    scored.sort((a, b) => b.matchScore - a.matchScore);

    // Return best match if score > 0
    const best = scored[0];
    return best.matchScore > 0 ? best : null;
  }

  /**
   * Calculate match score for agent vs task.
   */
  private calculateMatchScore(
    agent: NexusAgentEntity,
    task: RouteTaskDto,
  ): number {
    let score = 0;
    const requiredSkills = task.requiredSkills || [];

    if (requiredSkills.length === 0) {
      return 100; // No skills required = any agent matches
    }

    for (const required of requiredSkills) {
      const hasSkill = agent.skills.some(
        (s) =>
          s.id === required ||
          s.name.toLowerCase() === required.toLowerCase() ||
          s.tags?.includes(required),
      );
      if (hasSkill) {
        score += 100 / requiredSkills.length;
      }
    }

    return Math.round(score);
  }

  /**
   * Translate message between protocols using ProtocolTranslator.
   * Supports bidirectional translation: A2A <-> MCP <-> AgentKern
   */
  translateMessage(dto: TranslateMessageDto): any {
    const { sourceProtocol, targetProtocol, message } = dto;

    this.logger.log(`Translating ${sourceProtocol} -> ${targetProtocol}`);

    try {
      const translated = ProtocolTranslator.translate(
        sourceProtocol,
        targetProtocol,
        message,
      );

      this.logger.debug(
        `Translation complete: ${
          (translated as NexusMessage).method || 'unknown'
        } (${(translated as NexusMessage).id || 'unknown'})`,
      );

      return translated;
    } catch (error) {
      this.logger.error(`Translation failed: ${error}`);
      throw error;
    }
  }

  /**
   * Get service statistics (from database).
   */
  async getStats(): Promise<{
    registeredAgents: number;
    supportedProtocols: number;
  }> {
    const stats = await this.agentRepository.getStats();
    return {
      registeredAgents: stats.activeAgents,
      supportedProtocols: 6,
    };
  }

  /**
   * List supported protocols (synchronous operation).
   */
  listProtocols(): string[] {
    return ['a2a', 'mcp', 'agentkern', 'anp', 'nlip', 'aitp'];
  }
}
