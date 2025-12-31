/**
 * AgentKernIdentity - Agents Controller
 *
 * REST API for agent registration and management.
 * Used by playground and external clients to interact with the AgentSandboxService.
 */

import {
  Controller,
  Get,
  Post,
  Put,
  Delete,
  Body,
  Param,
  HttpCode,
  HttpStatus,
  NotFoundException,
  BadRequestException,
} from '@nestjs/common';
import {
  ApiTags,
  ApiOperation,
  ApiResponse,
  ApiParam,
} from '@nestjs/swagger';
import { AgentSandboxService } from '../services/agent-sandbox.service';
import { AgentStatus } from '../domain/agent.entity';

// ============================================================================
// DTOs
// ============================================================================

class RegisterAgentDto {
  name: string;
  version?: string;
  budget?: {
    maxTokens?: number;
    maxApiCalls?: number;
    maxCostUsd?: number;
    periodSeconds?: number;
  };
}

class VerifyActionDto {
  action: string;
  target: {
    service: string;
    endpoint: string;
    method: string;
  };
  estimatedTokens?: number;
  estimatedCost?: number;
}

class RecordResultDto {
  tokensUsed?: number;
  cost?: number;
  reason?: string;
}

// ============================================================================
// Controller
// ============================================================================

@ApiTags('Agents')
@Controller('api/v1/agents')
export class AgentsController {
  constructor(
    private readonly sandboxService: AgentSandboxService,
  ) {}

  // ============ Agent Registration ============

  @Post('register')
  @HttpCode(HttpStatus.CREATED)
  @ApiOperation({
    summary: 'Register a new agent',
    description: 'Creates a new agent identity with optional budget configuration.',
  })
  @ApiResponse({ status: 201, description: 'Agent registered successfully' })
  @ApiResponse({ status: 400, description: 'Invalid request' })
  async register(@Body() dto: RegisterAgentDto) {
    if (!dto.name) {
      throw new BadRequestException('Agent name is required');
    }

    const agentId = `agent-${Date.now().toString(36)}-${Math.random().toString(36).substring(2, 8)}`;
    const agent = await this.sandboxService.registerAgent(
      agentId,
      dto.name,
      dto.version || '1.0.0',
      dto.budget,
    );

    return {
      success: true,
      agent: {
        id: agent.id,
        name: agent.name,
        version: agent.version,
        status: agent.status,
        capabilities: ['read', 'write', 'execute'],
        trustScore: agent.reputation.score,
        budget: agent.budget,
        createdAt: agent.createdAt,
      },
    };
  }

  // ============ Agent Lookup ============

  @Get()
  @ApiOperation({
    summary: 'List all agents',
    description: 'Returns all registered agents.',
  })
  @ApiResponse({ status: 200, description: 'List of agents' })
  async listAgents() {
    const agents = this.sandboxService.getAllAgents();
    return {
      count: agents.length,
      agents: agents.map(a => ({
        id: a.id,
        name: a.name,
        status: a.status,
        trustScore: a.reputation.score,
        lastActiveAt: a.lastActiveAt,
      })),
    };
  }

  @Get(':agentId')
  @ApiOperation({
    summary: 'Get agent details',
    description: 'Returns detailed information about a specific agent.',
  })
  @ApiParam({ name: 'agentId', description: 'Agent ID' })
  @ApiResponse({ status: 200, description: 'Agent details' })
  @ApiResponse({ status: 404, description: 'Agent not found' })
  async getAgent(@Param('agentId') agentId: string) {
    const agent = this.sandboxService.getAgentStatus(agentId);
    if (!agent) {
      throw new NotFoundException(`Agent ${agentId} not found`);
    }

    return {
      id: agent.id,
      name: agent.name,
      version: agent.version,
      status: agent.status,
      capabilities: ['read', 'write', 'execute'],
      trustScore: agent.reputation.score,
      reputation: agent.reputation,
      budget: agent.budget,
      usage: agent.usage,
      createdAt: agent.createdAt,
      lastActiveAt: agent.lastActiveAt,
      terminatedAt: agent.terminatedAt,
      terminationReason: agent.terminationReason,
    };
  }

  // ============ Action Verification ============

  @Post(':agentId/verify')
  @HttpCode(HttpStatus.OK)
  @ApiOperation({
    summary: 'Verify an action',
    description: 'Check if an agent is allowed to perform an action.',
  })
  @ApiParam({ name: 'agentId', description: 'Agent ID' })
  @ApiResponse({ status: 200, description: 'Verification result' })
  async verifyAction(
    @Param('agentId') agentId: string,
    @Body() dto: VerifyActionDto,
  ) {
    const result = await this.sandboxService.checkAction({
      agentId,
      action: dto.action,
      target: dto.target,
      estimatedTokens: dto.estimatedTokens,
      estimatedCost: dto.estimatedCost,
    });

    return {
      allowed: result.allowed,
      status: result.agentStatus,
      reason: result.reason,
      remainingBudget: result.remainingBudget,
    };
  }

  // ============ Action Recording ============

  @Post(':agentId/success')
  @HttpCode(HttpStatus.OK)
  @ApiOperation({
    summary: 'Record successful action',
    description: 'Record that an agent completed an action successfully.',
  })
  async recordSuccess(
    @Param('agentId') agentId: string,
    @Body() dto: RecordResultDto,
  ) {
    await this.sandboxService.recordSuccess(agentId, dto.tokensUsed, dto.cost);
    return { success: true };
  }

  @Post(':agentId/failure')
  @HttpCode(HttpStatus.OK)
  @ApiOperation({
    summary: 'Record failed action',
    description: 'Record that an agent action failed.',
  })
  async recordFailure(
    @Param('agentId') agentId: string,
    @Body() dto: RecordResultDto,
  ) {
    await this.sandboxService.recordFailure(agentId, dto.reason || 'Unknown failure');
    return { success: true };
  }

  @Post(':agentId/violation')
  @HttpCode(HttpStatus.OK)
  @ApiOperation({
    summary: 'Record security violation',
    description: 'Record that an agent committed a security violation.',
  })
  async recordViolation(
    @Param('agentId') agentId: string,
    @Body() dto: RecordResultDto,
  ) {
    await this.sandboxService.recordViolation(agentId, dto.reason || 'Unknown violation');
    return { success: true };
  }

  // ============ Agent Lifecycle ============

  @Put(':agentId/suspend')
  @ApiOperation({
    summary: 'Suspend an agent',
    description: 'Temporarily suspend an agent.',
  })
  async suspendAgent(
    @Param('agentId') agentId: string,
    @Body() dto: { reason: string },
  ) {
    const result = await this.sandboxService.suspendAgent(agentId, dto.reason || 'Manual suspension');
    if (!result) {
      throw new NotFoundException(`Agent ${agentId} not found`);
    }
    return { success: true, status: AgentStatus.SUSPENDED };
  }

  @Put(':agentId/reactivate')
  @ApiOperation({
    summary: 'Reactivate a suspended agent',
    description: 'Reactivate a previously suspended agent.',
  })
  async reactivateAgent(@Param('agentId') agentId: string) {
    const result = await this.sandboxService.reactivateAgent(agentId);
    if (!result) {
      throw new BadRequestException(`Cannot reactivate agent ${agentId} (not found or terminated)`);
    }
    return { success: true, status: AgentStatus.ACTIVE };
  }

  @Delete(':agentId')
  @ApiOperation({
    summary: 'Terminate an agent',
    description: 'Permanently terminate an agent. This cannot be undone.',
  })
  async terminateAgent(
    @Param('agentId') agentId: string,
    @Body() dto: { reason?: string },
  ) {
    const result = await this.sandboxService.terminateAgent(agentId, dto.reason || 'Manual termination');
    if (!result) {
      throw new NotFoundException(`Agent ${agentId} not found`);
    }
    return { success: true, status: AgentStatus.TERMINATED };
  }

  // ============ System Status ============

  @Get('system/status')
  @ApiOperation({
    summary: 'Get system status',
    description: 'Returns kill switch status and in-flight request counts.',
  })
  async getSystemStatus() {
    const inFlightStats = this.sandboxService.getInFlightStats();
    return {
      killSwitchActive: this.sandboxService.isKillSwitchActive(),
      inFlightRequests: inFlightStats.total,
      inFlightByAgent: inFlightStats.byAgent,
    };
  }
}
