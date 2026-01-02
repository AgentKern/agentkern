/**
 * AgentKern Gateway - Nexus DTOs
 *
 * Data Transfer Objects for Nexus API.
 * Note: Uses plain TypeScript interfaces. For runtime validation,
 * add class-validator to package.json and uncomment decorators.
 */

// Supported protocols
export type Protocol = 'a2a' | 'mcp' | 'agentkern' | 'anp' | 'nlip' | 'aitp';

/**
 * Skill definition.
 */
export interface Skill {
  id: string;
  name: string;
  description?: string;
  tags?: string[];
  inputSchema?: object;
  outputSchema?: object;
}

/**
 * Capability definition.
 */
export interface Capability {
  name: string;
  inputModes?: string[];
  outputModes?: string[];
  rateLimit?: number;
}

/**
 * Agent Card (A2A compatible).
 */
export interface AgentCard {
  id: string;
  name: string;
  description: string;
  url: string;
  version: string;
  capabilities: Capability[];
  skills: Skill[];
  protocols: string[];
  registeredAt?: string;
}

/**
 * Unified Nexus Message format.
 */
export interface NexusMessage {
  id: string;
  method: string;
  params: unknown;
  sourceProtocol: Protocol;
  targetProtocol?: Protocol;
  sourceAgent?: string;
  targetAgent?: string;
  correlationId?: string;
  timestamp: string;
  metadata?: Record<string, unknown>;
}

/**
 * Register agent DTO.
 */
export class RegisterAgentDto {
  id?: string;
  name!: string;
  description?: string;
  url!: string;
  version?: string;
  capabilities?: Capability[];
  skills?: Skill[];
  protocols?: string[];
}

/**
 * Discover agent DTO.
 */
export class DiscoverAgentDto {
  url!: string;
}

/**
 * Route task DTO.
 */
export class RouteTaskDto {
  taskId?: string;
  taskType!: string;
  requiredSkills?: string[];
  params?: unknown;
  priority?: number;
}

/**
 * Translate message DTO.
 */
export class TranslateMessageDto {
  sourceProtocol!: Protocol;
  targetProtocol!: Protocol;
  message!: unknown;
}
