/**
 * AgentKern Gateway - Nexus Controller
 *
 * HTTP API for protocol translation and agent discovery.
 * Provides REST endpoints for the Nexus inter-agent protocol gateway.
 */

import {
  Controller,
  Post,
  Get,
  Delete,
  Body,
  Param,
  Query,
  HttpCode,
  HttpStatus,
  NotFoundException,
  BadRequestException,
} from '@nestjs/common';
import { ApiTags, ApiOperation } from '@nestjs/swagger';
import { NexusService } from '../services/nexus.service';
import {
  RegisterAgentDto,
  RouteTaskDto,
  TranslateMessageDto,
  DiscoverAgentDto,
} from '../dto/nexus.dto';
import type { AgentCard } from '../dto/nexus.dto';

/**
 * Nexus Controller - Protocol Gateway API
 */
@ApiTags('Nexus')
@Controller('api/v1/nexus')
export class NexusController {
  constructor(private readonly nexusService: NexusService) {}

  @Post('agents')
  @HttpCode(HttpStatus.CREATED)
  @ApiOperation({
    summary: 'Register an agent',
    description:
      'Register an agent with the Nexus registry (persisted to database)',
  })
  async registerAgent(@Body() dto: RegisterAgentDto): Promise<AgentCard> {
    return this.nexusService.registerAgent(dto);
  }

  @Get('agents')
  @ApiOperation({
    summary: 'List agents',
    description: 'List all registered agents or filter by skill',
  })
  async listAgents(@Query('skill') skill?: string): Promise<AgentCard[]> {
    if (skill) {
      return this.nexusService.findAgentsBySkill(skill);
    }
    return this.nexusService.listAgents();
  }

  @Get('agents/:id')
  @ApiOperation({
    summary: 'Get agent',
    description: 'Get a specific agent by ID',
  })
  async getAgent(@Param('id') id: string): Promise<AgentCard> {
    const agent = await this.nexusService.getAgent(id);
    if (!agent) {
      throw new NotFoundException(`Agent ${id} not found`);
    }
    return agent;
  }

  @Delete('agents/:id')
  @ApiOperation({
    summary: 'Unregister agent',
    description: 'Remove an agent from the registry (soft delete)',
  })
  async unregisterAgent(@Param('id') id: string): Promise<{
    success: boolean;
    agentId: string;
  }> {
    const result = await this.nexusService.unregisterAgent(id);
    if (!result) {
      throw new NotFoundException(`Agent ${id} not found`);
    }
    return { success: true, agentId: id };
  }

  @Post('discover')
  @HttpCode(HttpStatus.OK)
  @ApiOperation({
    summary: 'Discover agent',
    description: 'Discover an agent from its /.well-known/agent.json',
  })
  async discoverAgent(@Body() dto: DiscoverAgentDto): Promise<AgentCard> {
    return this.nexusService.discoverAgent(dto.url);
  }

  @Post('route')
  @HttpCode(HttpStatus.OK)
  @ApiOperation({
    summary: 'Route task',
    description: 'Route a task to the best matching agent',
  })
  async routeTask(@Body() dto: RouteTaskDto): Promise<{
    selectedAgent: AgentCard & { matchScore: number };
    taskId?: string;
    matchScore: number;
  }> {
    const agent = await this.nexusService.routeTask(dto);
    if (!agent) {
      throw new BadRequestException('No matching agent found for task');
    }
    return {
      selectedAgent: agent,
      taskId: dto.taskId,
      matchScore: agent.matchScore,
    };
  }

  @Post('translate')
  @HttpCode(HttpStatus.OK)
  @ApiOperation({
    summary: 'Translate message',
    description: 'Translate message between protocols (A2A, MCP, AgentKern)',
  })
  translateMessage(@Body() dto: TranslateMessageDto) {
    return this.nexusService.translateMessage(dto);
  }

  @Get('protocols')
  @ApiOperation({
    summary: 'List protocols',
    description: 'List all supported agent protocols',
  })
  listProtocols() {
    return {
      protocols: [
        {
          name: 'a2a',
          fullName: 'Google Agent-to-Agent Protocol',
          version: '0.3',
          status: 'stable',
        },
        {
          name: 'mcp',
          fullName: 'Anthropic Model Context Protocol',
          version: '2025-06-18',
          status: 'stable',
        },
        {
          name: 'agentkern',
          fullName: 'AgentKern Native Protocol',
          version: '1.0',
          status: 'stable',
        },
        {
          name: 'anp',
          fullName: 'W3C Agent Network Protocol',
          version: '0.1',
          status: 'beta',
        },
        {
          name: 'nlip',
          fullName: 'ECMA Natural Language Interaction Protocol',
          version: 'draft',
          status: 'beta',
        },
        {
          name: 'aitp',
          fullName: 'NEAR Agent Interaction and Transaction Protocol',
          version: 'rfc',
          status: 'beta',
        },
      ],
    };
  }

  @Get('health')
  @ApiOperation({
    summary: 'Health check',
    description: 'Check Nexus service health',
  })
  async health(): Promise<{
    status: string;
    registeredAgents: number;
    supportedProtocols: number;
  }> {
    const stats = await this.nexusService.getStats();
    return { status: 'healthy', ...stats };
  }
}

/**
 * Well-Known Controller for A2A Agent Discovery
 * Per A2A Spec: Agents publish capabilities at /.well-known/agent.json
 */
@ApiTags('Well-Known')
@Controller('.well-known')
export class WellKnownController {
  @Get('agent.json')
  @ApiOperation({
    summary: 'Agent card',
    description: 'A2A agent card for discovery',
  })
  getAgentCard() {
    return {
      id: 'agentkern-identity',
      name: 'AgentKern Identity',
      description:
        'Universal Agent Protocol Gateway - The Agentic Operating System',
      url: process.env.IDENTITY_URL || 'http://localhost:3001',
      version: '1.0.0',
      provider: { organization: 'AgentKern', url: 'https://agentkern.io' },
      capabilities: [
        'agent-registration',
        'protocol-translation',
        'task-routing',
        'agent-discovery',
      ],
      protocols: ['a2a', 'mcp', 'agentkern', 'anp', 'nlip', 'aitp'],
      authentication: {
        schemes: ['bearer', 'did-auth'],
      },
    };
  }
}
