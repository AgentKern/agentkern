/**
 * VeriMantle Gateway - Nexus Service
 * 
 * Business logic for protocol translation and agent discovery.
 * In production, this would call the Rust Nexus crate via FFI or HTTP.
 */

import { Injectable, Logger } from '@nestjs/common';
import { 
  RegisterAgentDto, 
  RouteTaskDto, 
  TranslateMessageDto,
  AgentCard,
  NexusMessage,
  Protocol,
} from '../dto/nexus.dto';

@Injectable()
export class NexusService {
  private readonly logger = new Logger(NexusService.name);
  
  // In-memory registry (would be Rust FFI in production)
  private agents = new Map<string, AgentCard>();

  /**
   * Register an agent.
   */
  async registerAgent(dto: RegisterAgentDto): Promise<AgentCard> {
    const card: AgentCard = {
      id: dto.id || crypto.randomUUID(),
      name: dto.name,
      description: dto.description || '',
      url: dto.url,
      version: dto.version || '1.0.0',
      capabilities: dto.capabilities || [],
      skills: dto.skills || [],
      protocols: dto.protocols || ['verimantle'],
      registeredAt: new Date().toISOString(),
    };

    this.agents.set(card.id, card);
    this.logger.log(`Agent registered: ${card.id} (${card.name})`);
    
    return card;
  }

  /**
   * List all agents.
   */
  async listAgents(): Promise<AgentCard[]> {
    return Array.from(this.agents.values());
  }

  /**
   * Get agent by ID.
   */
  async getAgent(id: string): Promise<AgentCard | null> {
    return this.agents.get(id) || null;
  }

  /**
   * Unregister an agent.
   */
  async unregisterAgent(id: string): Promise<boolean> {
    const existed = this.agents.has(id);
    this.agents.delete(id);
    if (existed) {
      this.logger.log(`Agent unregistered: ${id}`);
    }
    return existed;
  }

  /**
   * Find agents by skill.
   */
  async findAgentsBySkill(skill: string): Promise<AgentCard[]> {
    return Array.from(this.agents.values()).filter(agent =>
      agent.skills.some(s => 
        s.id === skill || 
        s.name.toLowerCase().includes(skill.toLowerCase()) ||
        s.tags?.includes(skill)
      )
    );
  }

  /**
   * Discover agent from URL (fetches /.well-known/agent.json).
   */
  async discoverAgent(url: string): Promise<AgentCard> {
    const wellKnownUrl = `${url.replace(/\/$/, '')}/.well-known/agent.json`;
    
    this.logger.log(`Discovering agent from: ${wellKnownUrl}`);

    try {
      const response = await fetch(wellKnownUrl);
      
      if (!response.ok) {
        throw new Error(`Failed to fetch agent card: ${response.status}`);
      }

      const card: AgentCard = await response.json();
      
      // Register the discovered agent
      this.agents.set(card.id, {
        ...card,
        registeredAt: new Date().toISOString(),
      });

      this.logger.log(`Agent discovered and registered: ${card.id}`);
      return card;
    } catch (error) {
      this.logger.error(`Discovery failed for ${url}: ${error}`);
      throw error;
    }
  }

  /**
   * Route task to best matching agent.
   */
  async routeTask(dto: RouteTaskDto): Promise<AgentCard & { matchScore: number } | null> {
    const agents = Array.from(this.agents.values());
    
    if (agents.length === 0) {
      return null;
    }

    // Score each agent
    const scored = agents.map(agent => ({
      ...agent,
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
  private calculateMatchScore(agent: AgentCard, task: RouteTaskDto): number {
    let score = 0;
    const requiredSkills = task.requiredSkills || [];

    if (requiredSkills.length === 0) {
      return 100; // No skills required = any agent matches
    }

    for (const required of requiredSkills) {
      const hasSkill = agent.skills.some(s => 
        s.id === required || 
        s.name.toLowerCase() === required.toLowerCase() ||
        s.tags?.includes(required)
      );
      if (hasSkill) {
        score += 100 / requiredSkills.length;
      }
    }

    return Math.round(score);
  }

  /**
   * Translate message between protocols.
   */
  async translateMessage(dto: TranslateMessageDto): Promise<NexusMessage> {
    const { sourceProtocol, targetProtocol, message } = dto;

    this.logger.log(`Translating ${sourceProtocol} -> ${targetProtocol}`);

    // Parse based on source protocol
    const unified = this.parseToUnified(sourceProtocol, message);

    // Serialize to target protocol
    const translated = this.serializeFromUnified(targetProtocol, unified);

    return translated;
  }

  /**
   * Parse protocol-specific message to unified format.
   */
  private parseToUnified(protocol: Protocol, message: any): NexusMessage {
    const base: NexusMessage = {
      id: message.id || crypto.randomUUID(),
      method: '',
      params: {},
      sourceProtocol: protocol,
      timestamp: new Date().toISOString(),
    };

    switch (protocol) {
      case 'a2a':
        // A2A uses JSON-RPC 2.0
        return {
          ...base,
          method: message.method || '',
          params: message.params || {},
        };

      case 'mcp':
        // MCP also uses JSON-RPC 2.0
        return {
          ...base,
          method: message.method || '',
          params: message.params || {},
        };

      case 'verimantle':
      default:
        return {
          ...base,
          method: message.method || '',
          params: message.params || message,
        };
    }
  }

  /**
   * Serialize unified message to protocol-specific format.
   */
  private serializeFromUnified(protocol: Protocol, msg: NexusMessage): NexusMessage {
    switch (protocol) {
      case 'a2a':
        return {
          ...msg,
          // A2A specific fields
          jsonrpc: '2.0',
          targetProtocol: 'a2a',
        } as any;

      case 'mcp':
        return {
          ...msg,
          // MCP specific fields
          jsonrpc: '2.0',
          targetProtocol: 'mcp',
        } as any;

      case 'verimantle':
      default:
        return {
          ...msg,
          targetProtocol: 'verimantle',
        };
    }
  }

  /**
   * Get service statistics.
   */
  async getStats() {
    return {
      registeredAgents: this.agents.size,
      supportedProtocols: 6,
    };
  }
}
