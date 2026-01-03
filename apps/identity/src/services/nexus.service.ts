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
import * as fs from 'fs';
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

// Type definitions for bridge responses
interface BridgeErrorResponse {
  error: string;
}

interface BridgeAgentResponse extends Partial<AgentCard> {
  error?: string;
  matchScore?: number;
}

interface BridgeStatsResponse {
  registeredAgents: number;
  supportedProtocols: number;
}

interface BridgeAgentsResponse extends Array<Partial<AgentCard>> {
  error?: string;
}

@Injectable()
export class NexusService implements OnModuleInit {
  private readonly logger = new Logger(NexusService.name);
  private bridge!: NativeBridge;
  private bridgeLoaded = false;

  async onModuleInit(): Promise<void> {
    await Promise.resolve();

    const isProduction = process.env.NODE_ENV === 'production';
    const bridgePath = this.resolveBridgePath();

    try {
      // Verify bridge file exists
      if (!fs.existsSync(bridgePath)) {
        throw new Error(
          `Bridge file not found at: ${bridgePath}. Run: cd packages/foundation/bridge && pnpm build`,
        );
      }

      // Load bridge
      // eslint-disable-next-line @typescript-eslint/no-require-imports
      this.bridge = require(bridgePath) as NativeBridge;
      this.bridgeLoaded = true;
      this.logger.log('🌐 Nexus N-API Bridge loaded successfully');

      // Verify bridge is operational
      await this.verifyBridge();
    } catch (error: unknown) {
      const errorMessage =
        error instanceof Error ? error.message : String(error);

      if (isProduction) {
        this.logger.error(
          `🚨 CRITICAL: Failed to load N-API bridge in production: ${errorMessage}`,
        );
        throw new Error(
          `N-API bridge is required in production but failed to load: ${errorMessage}`,
        );
      } else {
        this.logger.error(
          `🚨 Failed to load Nexus N-API bridge: ${errorMessage}`,
        );
        this.logger.warn(
          '⚠️ DEPRECATED: NexusService operating in degraded mode. See EPISTEMIC_HEALTH.md',
        );
        this.logger.warn(
          '⚠️ To fix: cd packages/foundation/bridge && pnpm build',
        );
      }
    }
  }

  /**
   * Resolve bridge path with proper error handling
   */
  private resolveBridgePath(): string {
    const possiblePaths = [
      path.resolve(
        __dirname,
        '../../../../packages/foundation/bridge/index.node',
      ),
      path.resolve(__dirname, '../../../packages/foundation/bridge/index.node'),
      '/app/packages/foundation/bridge/index.node',
    ];

    for (const testPath of possiblePaths) {
      if (fs.existsSync(testPath)) {
        return testPath;
      }
    }

    throw new Error(
      `Bridge not found in any expected location: ${possiblePaths.join(', ')}`,
    );
  }

  /**
   * Verify bridge is operational
   */
  private async verifyBridge(): Promise<void> {
    try {
      // Bridge methods return Promises
      const testResult = await this.bridge.nexusGetStats();
      if (!testResult) {
        throw new Error('Bridge returned null for test call');
      }
      JSON.parse(testResult);
      this.logger.log('✅ Bridge verification successful');
    } catch (error: unknown) {
      const errorMessage =
        error instanceof Error ? error.message : String(error);
      throw new Error(`Bridge verification failed: ${errorMessage}`);
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
      return JSON.parse(result) as NexusMessage | BridgeErrorResponse;
    } catch (error: unknown) {
      const errorMessage =
        error instanceof Error ? error.message : String(error);
      this.logger.error(`Failed to receive message: ${errorMessage}`);
      return { error: errorMessage };
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
      const parsed = JSON.parse(result) as {
        error?: string;
        success?: boolean;
      };
      if (parsed.error) throw new Error(parsed.error);

      return card;
    } catch (error: unknown) {
      const errorMessage =
        error instanceof Error ? error.message : String(error);
      this.logger.error(
        `Failed to register agent ${dto.name}: ${errorMessage}`,
      );
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
      const agents = JSON.parse(result) as BridgeAgentsResponse | AgentCard[];
      return this.mapAgents(agents);
    } catch (error: unknown) {
      const errorMessage =
        error instanceof Error ? error.message : String(error);
      this.logger.error(`Failed to list agents: ${errorMessage}`);
      return [];
    }
  }

  /**
   * Find agents by skill (client-side filtering for now).
   */
  async findAgentsBySkill(skill: string): Promise<AgentCard[]> {
    const agents = await this.listAgents();
    return agents.filter((a) =>
      a.skills?.some((s) => s.name === skill || s.id === skill),
    );
  }

  /**
   * Get agent by ID.
   */
  async getAgent(id: string): Promise<AgentCard | null> {
    if (!this.bridgeLoaded) return null;

    try {
      const result = await this.bridge.nexusGetAgent(id);
      if (result === 'null') return null;
      const agent = JSON.parse(result) as BridgeAgentResponse;
      return this.mapAgent(agent);
    } catch (error: unknown) {
      const errorMessage =
        error instanceof Error ? error.message : String(error);
      this.logger.error(`Failed to get agent ${id}: ${errorMessage}`);
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
    } catch (error: unknown) {
      const errorMessage =
        error instanceof Error ? error.message : String(error);
      this.logger.error(`Failed to unregister agent ${id}: ${errorMessage}`);
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
      const parsed = JSON.parse(result) as BridgeAgentResponse;
      if (parsed.error) throw new Error(parsed.error);
      return this.mapAgent(parsed);
    } catch (error: unknown) {
      const errorMessage =
        error instanceof Error ? error.message : String(error);
      this.logger.error(`Failed to discover agent at ${url}: ${errorMessage}`);
      throw error instanceof Error ? error : new Error(errorMessage);
    }
  }

  /**
   * Route task to best matching agent.
   */
  async routeTask(
    dto: RouteTaskDto,
  ): Promise<(AgentCard & { matchScore: number }) | null> {
    if (!this.bridgeLoaded) return null;

    try {
      const taskJson = JSON.stringify(dto);
      const result = await this.bridge.nexusRouteTask(taskJson);
      const parsed = JSON.parse(result) as BridgeAgentResponse;
      if (parsed.error) throw new Error(parsed.error);

      const agent = this.mapAgent(parsed);
      return { ...agent, matchScore: parsed.matchScore ?? 0 };
    } catch (error: unknown) {
      const errorMessage =
        error instanceof Error ? error.message : String(error);
      this.logger.error(`Failed to route task: ${errorMessage}`);
      return null;
    }
  }

  /**
   * Translate message between protocols.
   */
  async translateMessage(
    dto: TranslateMessageDto,
  ): Promise<NexusMessage | { error: string; payload?: string }> {
    if (!this.bridgeLoaded) return { error: 'Bridge not loaded' };

    try {
      // 1. Receive (Foreign -> Native)
      const payload =
        typeof dto.message === 'string'
          ? dto.message
          : JSON.stringify(dto.message);
      const nativeMsg = await this.receive(payload);

      if ('error' in nativeMsg) throw new Error(nativeMsg.error);

      // 2. Send (Native -> Target)
      const nativeJson = JSON.stringify(nativeMsg);
      const result = await this.bridge.nexusSend(
        nativeJson,
        dto.targetProtocol,
      );

      try {
        return JSON.parse(result) as NexusMessage;
      } catch {
        return { error: 'Failed to parse result', payload: result };
      }
    } catch (error: unknown) {
      const errorMessage =
        error instanceof Error ? error.message : String(error);
      this.logger.error(`Failed to translate message: ${errorMessage}`);
      return { error: errorMessage };
    }
  }

  /**
   * Get Nexus stats.
   */
  async getStats(): Promise<{
    registeredAgents: number;
    supportedProtocols: number;
  }> {
    if (!this.bridgeLoaded) {
      return { registeredAgents: 0, supportedProtocols: 0 };
    }

    try {
      const result = await this.bridge.nexusGetStats();
      return JSON.parse(result) as BridgeStatsResponse;
    } catch (error: unknown) {
      const errorMessage =
        error instanceof Error ? error.message : String(error);
      this.logger.error(`Failed to get stats: ${errorMessage}`);
      return { registeredAgents: 0, supportedProtocols: 0 };
    }
  }

  /**
   * Helper: Map bridge agent to DTO AgentCard
   * Handles loose typing from bridge (e.g. capabilities might be strings or objects)
   */
  private mapAgent(raw: BridgeAgentResponse | Partial<AgentCard>): AgentCard {
    const rawSkills = raw.skills;
    const skills: Skill[] = Array.isArray(rawSkills)
      ? rawSkills.map((s: string | Skill) =>
          typeof s === 'string' ? { id: s, name: s } : s,
        )
      : [];

    const rawCapabilities = raw.capabilities;
    const capabilities: Capability[] = Array.isArray(rawCapabilities)
      ? rawCapabilities.map((c: string | Capability) =>
          typeof c === 'string' ? { name: c } : c,
        )
      : [];

    return {
      id: raw.id ?? '',
      name: raw.name ?? '',
      description: raw.description ?? '',
      skills,
      capabilities,
      protocols: raw.protocols ?? ['a2a', 'mcp'], // Default if missing
      version: raw.version ?? '1.0.0',
      registeredAt: raw.registeredAt ?? new Date().toISOString(),
    };
  }

  private mapAgents(
    list: BridgeAgentsResponse | AgentCard[] | Partial<AgentCard>[],
  ): AgentCard[] {
    if (Array.isArray(list)) {
      return list.map((a) => this.mapAgent(a));
    }
    return [];
  }

  createA2ATask(id: string, description: string): string {
    if (!this.bridgeLoaded)
      return JSON.stringify({ error: 'Bridge not loaded' });
    try {
      return this.bridge.nexusCreateA2aTask(id, description);
    } catch (error: unknown) {
      const errorMessage =
        error instanceof Error ? error.message : String(error);
      return JSON.stringify({ error: errorMessage });
    }
  }
}
