/**
 * AgentKernIdentity - Nexus Controller
 * 
 * REST API for protocol translation and agent discovery.
 * Merged from apps/gateway for consolidated architecture.
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
import { ApiTags, ApiOperation } from '@nestjs/swagger';
import { NexusService } from '../services/nexus.service';
import { 
  RegisterAgentDto, 
  DiscoverAgentDto, 
  RouteTaskDto,
  TranslateMessageDto,
} from '../dto/nexus.dto';

@ApiTags('Nexus')
@Controller('nexus')
export class NexusController {
  constructor(private readonly nexusService: NexusService) {}

  @Post('agents')
  @HttpCode(HttpStatus.CREATED)
  @ApiOperation({ summary: 'Register an agent', description: 'Register an agent with the Nexus registry' })
  async registerAgent(@Body() dto: RegisterAgentDto) {
    return this.nexusService.registerAgent(dto);
  }

  @Get('agents')
  @ApiOperation({ summary: 'List agents', description: 'List all registered agents or filter by skill' })
  async listAgents(@Query('skill') skill?: string) {
    if (skill) {
      return this.nexusService.findAgentsBySkill(skill);
    }
    return this.nexusService.listAgents();
  }

  @Get('agents/:id')
  @ApiOperation({ summary: 'Get agent', description: 'Get a specific agent by ID' })
  async getAgent(@Param('id') id: string) {
    const agent = await this.nexusService.getAgent(id);
    if (!agent) {
      throw new NotFoundException(`Agent ${id} not found`);
    }
    return agent;
  }

  @Delete('agents/:id')
  @ApiOperation({ summary: 'Unregister agent', description: 'Remove an agent from the registry' })
  async unregisterAgent(@Param('id') id: string) {
    const result = await this.nexusService.unregisterAgent(id);
    if (!result) {
      throw new NotFoundException(`Agent ${id} not found`);
    }
    return { success: true, agentId: id };
  }

  @Post('discover')
  @HttpCode(HttpStatus.OK)
  @ApiOperation({ summary: 'Discover agent', description: 'Discover an agent from its /.well-known/agent.json' })
  async discoverAgent(@Body() dto: DiscoverAgentDto) {
    return this.nexusService.discoverAgent(dto.url);
  }

  @Post('route')
  @HttpCode(HttpStatus.OK)
  @ApiOperation({ summary: 'Route task', description: 'Route a task to the best matching agent' })
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

  @Post('translate')
  @HttpCode(HttpStatus.OK)
  @ApiOperation({ summary: 'Translate message', description: 'Translate message between protocols (A2A, MCP, AgentKern)' })
  async translateMessage(@Body() dto: TranslateMessageDto) {
    return this.nexusService.translateMessage(dto);
  }

  @Get('protocols')
  @ApiOperation({ summary: 'List protocols', description: 'List all supported agent protocols' })
  async listProtocols() {
    return {
      protocols: [
        { name: 'a2a', fullName: 'Google Agent-to-Agent Protocol', version: '0.3', status: 'stable' },
        { name: 'mcp', fullName: 'Anthropic Model Context Protocol', version: '2025-06-18', status: 'stable' },
        { name: 'agentkern', fullName: 'AgentKern Native Protocol', version: '1.0', status: 'stable' },
        { name: 'anp', fullName: 'W3C Agent Network Protocol', version: '0.1', status: 'beta' },
        { name: 'nlip', fullName: 'ECMA Natural Language Interaction Protocol', version: 'draft', status: 'beta' },
        { name: 'aitp', fullName: 'NEAR Agent Interaction and Transaction Protocol', version: 'rfc', status: 'beta' },
      ],
    };
  }

  @Get('health')
  @ApiOperation({ summary: 'Health check', description: 'Check Nexus service health' })
  async health() {
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
  @ApiOperation({ summary: 'Agent card', description: 'A2A agent card for discovery' })
  async getAgentCard() {
    return {
      id: 'agentkern-identity',
      name: 'AgentKern Identity',
      description: 'Universal Agent Protocol Gateway - The Agentic Operating System',
      url: process.env.IDENTITY_URL || 'http://localhost:3001',
      version: '1.0.0',
      provider: { organization: 'AgentKern', url: 'https://agentkern.io' },
      capabilities: [
        { name: 'protocol-translation', inputModes: ['text', 'code'], outputModes: ['text', 'code'] },
        { name: 'agent-discovery', inputModes: ['text'], outputModes: ['text'] },
        { name: 'task-routing', inputModes: ['text'], outputModes: ['text'] },
      ],
      skills: [
        { id: 'translate', name: 'Protocol Translation', description: 'Translate between A2A, MCP, AgentKern', tags: ['translation', 'a2a', 'mcp'] },
        { id: 'route', name: 'Task Routing', description: 'Route tasks to best agent', tags: ['routing'] },
        { id: 'discover', name: 'Agent Discovery', description: 'Discover agents from URLs', tags: ['discovery'] },
      ],
      protocols: [
        { name: 'a2a', version: '0.3' },
        { name: 'mcp', version: '2025-06-18' },
        { name: 'agentkern', version: '1.0' },
      ],
      extensions: {
        agentkern: {
          pillars: ['identity', 'gate', 'synapse', 'arbiter', 'nexus', 'treasury'],
          loopPrevention: true,
          explainability: true,
        },
      },
    };
  }
}
