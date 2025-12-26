/**
 * VeriMantle Gateway - Nexus Controller
 * 
 * REST API for protocol translation and agent discovery.
 * 
 * Endpoints:
 * - POST /nexus/agents - Register an agent
 * - GET /nexus/agents - List all agents
 * - GET /nexus/agents/:id - Get agent by ID
 * - DELETE /nexus/agents/:id - Unregister agent
 * - POST /nexus/discover - Discover agent from URL
 * - POST /nexus/route - Route task to best agent
 * - POST /nexus/translate - Translate message between protocols
 * - GET /nexus/protocols - List supported protocols
 */

import { 
  Controller, 
  Get, 
  Post, 
  Delete, 
  Body, 
  Param, 
  Query,
  HttpCode, 
  HttpStatus,
  NotFoundException,
  BadRequestException,
} from '@nestjs/common';
import { NexusService } from '../services/nexus.service';
import { 
  RegisterAgentDto, 
  DiscoverAgentDto, 
  RouteTaskDto,
  TranslateMessageDto,
} from '../dto/nexus.dto';

@Controller('nexus')
export class NexusController {
  constructor(private readonly nexusService: NexusService) {}

  /**
   * Register an agent with the Nexus registry.
   * Returns the registered agent card.
   */
  @Post('agents')
  @HttpCode(HttpStatus.CREATED)
  async registerAgent(@Body() dto: RegisterAgentDto) {
    return this.nexusService.registerAgent(dto);
  }

  /**
   * List all registered agents.
   */
  @Get('agents')
  async listAgents(@Query('skill') skill?: string) {
    if (skill) {
      return this.nexusService.findAgentsBySkill(skill);
    }
    return this.nexusService.listAgents();
  }

  /**
   * Get a specific agent by ID.
   */
  @Get('agents/:id')
  async getAgent(@Param('id') id: string) {
    const agent = await this.nexusService.getAgent(id);
    if (!agent) {
      throw new NotFoundException(`Agent ${id} not found`);
    }
    return agent;
  }

  /**
   * Unregister an agent.
   */
  @Delete('agents/:id')
  async unregisterAgent(@Param('id') id: string) {
    const result = await this.nexusService.unregisterAgent(id);
    if (!result) {
      throw new NotFoundException(`Agent ${id} not found`);
    }
    return { success: true, agentId: id };
  }

  /**
   * Discover an agent from its base URL.
   * Fetches /.well-known/agent.json per A2A spec.
   */
  @Post('discover')
  @HttpCode(HttpStatus.OK)
  async discoverAgent(@Body() dto: DiscoverAgentDto) {
    return this.nexusService.discoverAgent(dto.url);
  }

  /**
   * Route a task to the best matching agent.
   */
  @Post('route')
  @HttpCode(HttpStatus.OK)
  async routeTask(@Body() dto: RouteTaskDto) {
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

  /**
   * Translate a message between protocols.
   * Supports: a2a, mcp, verimantle
   */
  @Post('translate')
  @HttpCode(HttpStatus.OK)
  async translateMessage(@Body() dto: TranslateMessageDto) {
    return this.nexusService.translateMessage(dto);
  }

  /**
   * List supported protocols.
   */
  @Get('protocols')
  async listProtocols() {
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
          name: 'verimantle',
          fullName: 'VeriMantle Native Protocol',
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

  /**
   * Health check for Nexus service.
   */
  @Get('health')
  async health() {
    const stats = await this.nexusService.getStats();
    return {
      status: 'healthy',
      ...stats,
    };
  }
}
