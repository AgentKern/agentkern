/**
 * AgentKern Identity - Nexus Service
 *
 * Wrapper for N-API bridge to Rust Nexus logic.
 * Handles protocol translation and agent registration via the Gateway.
 *
 * Per DECISION_RECORD_BRIDGE.md: N-API for hot path (0ms latency)
 */

import { Injectable, Logger, OnModuleInit } from '@nestjs/common';
import * as path from 'path';
import {
  AgentCard,
  TranslateMessageDto,
  RouteTaskDto,
  NexusMessage,
  Skill,
  Capability,
  RegisterAgentDto,
} from '../dto/nexus.dto';
import { v4 as uuidv4 } from 'uuid';

// Bridge interface (loaded from native module)
interface NativeBridge {
  nexusReceive(payload: string): Promise<string>;
  nexusRegisterAgent(cardJson: string): Promise<string>;
  nexusListAgents(): Promise<string>;
  nexusGetAgent(id: string): Promise<string>;
  nexusUnregisterAgent(id: string): Promise<boolean>;
  nexusDiscoverAgent(url: string): Promise<string>;
  nexusRouteTask(taskJson: string): Promise<string>;
  nexusGetStats(): Promise<string>;
  nexusCreateA2aTask(id: string, description: string): string;
  nexusSend(msgJson: string, targetProtocol: string): Promise<string>;
}

@Injectable()
export class NexusService implements OnModuleInit {
  private readonly logger = new Logger(NexusService.name);
  private bridge!: NativeBridge;
  private bridgeLoaded = false;

  async onModuleInit(): Promise<void> {
    await Promise.resolve(); // Ensure async context

    try {
      const bridgePath = path.resolve(
        __dirname,
        '../../../../packages/foundation/bridge/index.node',
      );

      // eslint-disable-next-line @typescript-eslint/no-require-imports
      this.bridge = require(bridgePath) as NativeBridge;
      this.bridgeLoaded = true;
      this.logger.log('🌐 Nexus N-API Bridge loaded successfully');
    } catch (error) {
      this.logger.error(`🚨 Failed to load Nexus N-API bridge: ${error}`);
      this.logger.warn('NexusService will operate in degraded mode');
    }
  }

  isOperational(): boolean {
    return this.bridgeLoaded;
  }

  /**
   * Receive and translate a raw message payload.
   * Auto-detects protocol (A2A, MCP, etc.) in Rust.
   */
  async receive(payload: string): Promise<NexusMessage | { error: string }> {
    if (!this.bridgeLoaded) return { error: 'Bridge not loaded' };

    try {
      const result = await this.bridge.nexusReceive(payload);
      return JSON.parse(result);
    } catch (error) {
      this.logger.error(`Failed to receive message: ${error}`);
      return { error: String(error) };
    }
  }

  /**
   * Register a new agent with the Nexus Gateway.
   */
  async registerAgent(dto: RegisterAgentDto): Promise<AgentCard> {
    if (!this.bridgeLoaded) throw new Error('Bridge not loaded');

    try {
      // Ensure ID exists
      const id = dto.id || uuidv4();
      
      // Convert DTO to Card (filling defaults)
      const card: AgentCard = {
        ...dto,
        id,
        description: dto.description || '',
        skills: dto.skills || [],
        capabilities: dto.capabilities || [],
        protocols: dto.protocols || ['a2a', 'mcp'],
        version: dto.version || '1.0.0',
        registeredAt: new Date().toISOString(),
      };

      const json = JSON.stringify(card);
      const result = await this.bridge.nexusRegisterAgent(json);
      const parsed = JSON.parse(result);
      if (parsed.error) throw new Error(parsed.error);
      
      return card;
    } catch (error) {
      this.logger.error(`Failed to register agent ${dto.name}: ${error}`);
      throw error;
    }
  }

  /**
   * List all agents.
   */
  async listAgents(): Promise<AgentCard[]> {
    if (!this.bridgeLoaded) return [];

    try {
      const result = await this.bridge.nexusListAgents();
      const agents = JSON.parse(result);
      return this.mapAgents(agents);
    } catch (error) {
      this.logger.error(`Failed to list agents: ${error}`);
      return [];
    }
  }

  /**
   * Find agents by skill (client-side filtering for now).
   */
  async findAgentsBySkill(skill: string): Promise<AgentCard[]> {
    const agents = await this.listAgents();
    return agents.filter((a) => a.skills?.some(s => s.name === skill || s.id === skill));
  }

  /**
   * Get agent by ID.
   */
  async getAgent(id: string): Promise<AgentCard | null> {
    if (!this.bridgeLoaded) return null;

    try {
      const result = await this.bridge.nexusGetAgent(id);
      if (result === 'null') return null;
      const agent = JSON.parse(result);
      return this.mapAgent(agent);
    } catch (error) {
      this.logger.error(`Failed to get agent ${id}: ${error}`);
      return null;
    }
  }

  /**
   * Unregister agent.
   */
  async unregisterAgent(id: string): Promise<boolean> {
    if (!this.bridgeLoaded) return false;

    try {
      return await this.bridge.nexusUnregisterAgent(id);
    } catch (error) {
      this.logger.error(`Failed to unregister agent ${id}: ${error}`);
      return false;
    }
  }

  /**
   * Discover agent from URL.
   */
  async discoverAgent(url: string): Promise<AgentCard> {
    if (!this.bridgeLoaded) throw new Error('Bridge not loaded');

    try {
      const result = await this.bridge.nexusDiscoverAgent(url);
      const parsed = JSON.parse(result);
      if (parsed.error) throw new Error(parsed.error);
      return this.mapAgent(parsed);
    } catch (error) {
      this.logger.error(`Failed to discover agent at ${url}: ${error}`);
      throw error;
    }
  }

  /**
   * Route task to best matching agent.
   */
  async routeTask(dto: RouteTaskDto): Promise<(AgentCard & { matchScore: number }) | null> {
    if (!this.bridgeLoaded) return null;

    try {
      const taskJson = JSON.stringify(dto);
      const result = await this.bridge.nexusRouteTask(taskJson);
      const parsed = JSON.parse(result);
      if (parsed.error) throw new Error(parsed.error);
      
      const agent = this.mapAgent(parsed);
      return { ...agent, matchScore: parsed.matchScore || 0 };
    } catch (error) {
      this.logger.error(`Failed to route task: ${error}`);
      return null;
    }
  }

  /**
   * Translate message between protocols.
   */
  async translateMessage(dto: TranslateMessageDto): Promise<any> {
    if (!this.bridgeLoaded) return { error: 'Bridge not loaded' };

    try {
      // 1. Receive (Foreign -> Native)
      const payload = typeof dto.message === 'string' ? dto.message : JSON.stringify(dto.message);
      const nativeMsg = await this.receive(payload);

      if ('error' in nativeMsg) throw new Error(nativeMsg.error);

      // 2. Send (Native -> Target)
      const nativeJson = JSON.stringify(nativeMsg);
      const result = await this.bridge.nexusSend(nativeJson, dto.targetProtocol);
      
      try {
        return JSON.parse(result);
      } catch {
        return { payload: result };
      }
    } catch (error) {
      this.logger.error(`Failed to translate message: ${error}`);
      return { error: String(error) };
    }
  }

  /**
   * Get Nexus stats.
   */
  async getStats(): Promise<{ registeredAgents: number; supportedProtocols: number }> {
    if (!this.bridgeLoaded) {
      return { registeredAgents: 0, supportedProtocols: 0 };
    }

    try {
      const result = await this.bridge.nexusGetStats();
      return JSON.parse(result);
    } catch (error) {
      this.logger.error(`Failed to get stats: ${error}`);
      return { registeredAgents: 0, supportedProtocols: 0 };
    }
  }

  /**
   * Helper: Map bridge agent to DTO AgentCard
   * Handles loose typing from bridge (e.g. capabilities might be strings or objects)
   */
  private mapAgent(raw: any): AgentCard {
    const skills: Skill[] = Array.isArray(raw.skills) 
      ? raw.skills.map((s: any) => typeof s === 'string' ? { id: s, name: s } : s)
      : [];

    const capabilities: Capability[] = Array.isArray(raw.capabilities)
      ? raw.capabilities.map((c: any) => typeof c === 'string' ? { name: c } : c)
      : [];

    return {
      ...raw,
      skills,
      capabilities,
      protocols: raw.protocols || ['a2a', 'mcp'], // Default if missing
    };
  }
  
  private mapAgents(list: any[]): AgentCard[] {
      return list.map(a => this.mapAgent(a));
  }

  createA2ATask(id: string, description: string): string {
    if (!this.bridgeLoaded) return JSON.stringify({ error: 'Bridge not loaded' });
    try {
      return this.bridge.nexusCreateA2aTask(id, description);
    } catch (error) {
      return JSON.stringify({ error: String(error) });
    }
  }
}
