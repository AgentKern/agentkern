/**
 * AgentKern Gateway - Nexus Controller
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
   * Supports: a2a, mcp, agentkern
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

/**
 * Well-Known Controller for A2A Agent Discovery
 * 
 * Per A2A Spec: Agents publish capabilities at /.well-known/agent.json
 */
@Controller('.well-known')
export class WellKnownController {
  constructor(private readonly nexusService: NexusService) {}

  /**
   * A2A Agent Card endpoint.
   * Returns this gateway's agent card for discovery.
   */
  @Get('agent.json')
  async getAgentCard() {
    return {
      id: 'agentkern-gateway',
      name: 'AgentKern Gateway',
      description: 'Universal Agent Protocol Gateway - The Agentic Operating System',
      url: process.env.GATEWAY_URL || 'http://localhost:3000',
      version: '1.0.0',
      provider: {
        organization: 'AgentKern',
        url: 'https://agentkern.io',
      },
      capabilities: [
        {
          name: 'protocol-translation',
          inputModes: ['text', 'code'],
          outputModes: ['text', 'code'],
        },
        {
          name: 'agent-discovery',
          inputModes: ['text'],
          outputModes: ['text'],
        },
        {
          name: 'task-routing',
          inputModes: ['text'],
          outputModes: ['text'],
        },
      ],
      skills: [
        {
          id: 'translate',
          name: 'Protocol Translation',
          description: 'Translate messages between A2A, MCP, and AgentKern protocols',
          tags: ['translation', 'a2a', 'mcp'],
        },
        {
          id: 'route',
          name: 'Task Routing',
          description: 'Route tasks to the best matching agent',
          tags: ['routing', 'orchestration'],
        },
        {
          id: 'discover',
          name: 'Agent Discovery',
          description: 'Discover and register agents from URLs',
          tags: ['discovery', 'registry'],
        },
      ],
      defaultInputModes: ['text'],
      defaultOutputModes: ['text'],
      authentication: {
        schemes: ['bearer', 'apiKey'],
      },
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

