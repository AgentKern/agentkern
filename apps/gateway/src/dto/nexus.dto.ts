/**
 * VeriMantle Gateway - Nexus DTOs
 * 
 * Data Transfer Objects for Nexus API.
 */

import { IsString, IsOptional, IsArray, IsUrl, IsObject, IsIn } from 'class-validator';

// Supported protocols
export type Protocol = 'a2a' | 'mcp' | 'verimantle' | 'anp' | 'nlip' | 'aitp';

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
  params: any;
  sourceProtocol: Protocol;
  targetProtocol?: Protocol;
  sourceAgent?: string;
  targetAgent?: string;
  correlationId?: string;
  timestamp: string;
  metadata?: Record<string, any>;
}

/**
 * Register agent DTO.
 */
export class RegisterAgentDto {
  @IsOptional()
  @IsString()
  id?: string;

  @IsString()
  name: string;

  @IsOptional()
  @IsString()
  description?: string;

  @IsUrl()
  url: string;

  @IsOptional()
  @IsString()
  version?: string;

  @IsOptional()
  @IsArray()
  capabilities?: Capability[];

  @IsOptional()
  @IsArray()
  skills?: Skill[];

  @IsOptional()
  @IsArray()
  protocols?: string[];
}

/**
 * Discover agent DTO.
 */
export class DiscoverAgentDto {
  @IsUrl()
  url: string;
}

/**
 * Route task DTO.
 */
export class RouteTaskDto {
  @IsOptional()
  @IsString()
  taskId?: string;

  @IsString()
  taskType: string;

  @IsOptional()
  @IsArray()
  requiredSkills?: string[];

  @IsOptional()
  @IsObject()
  params?: any;

  @IsOptional()
  priority?: number;
}

/**
 * Translate message DTO.
 */
export class TranslateMessageDto {
  @IsIn(['a2a', 'mcp', 'verimantle', 'anp', 'nlip', 'aitp'])
  sourceProtocol: Protocol;

  @IsIn(['a2a', 'mcp', 'verimantle', 'anp', 'nlip', 'aitp'])
  targetProtocol: Protocol;

  @IsObject()
  message: any;
}
