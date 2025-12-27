/**
 * AgentKern Gateway - Synapse Controller
 * 
 * REST API for agent memory and intent tracking.
 * Endpoints:
 * - GET /synapse/:agentId/state - Get agent state
 * - PUT /synapse/:agentId/state - Update agent state
 * - POST /synapse/:agentId/intent - Start a new intent path
 * - POST /synapse/:agentId/step - Record an intent step
 * - GET /synapse/:agentId/drift - Check for intent drift
 */

import { Controller, Get, Post, Put, Body, Param, HttpCode, HttpStatus } from '@nestjs/common';
import { SynapseService } from '../services/synapse.service';
import { SetStateDto, StartIntentDto, RecordStepDto } from '../dto/synapse.dto';

@Controller('synapse')
export class SynapseController {
  constructor(private readonly synapseService: SynapseService) {}

  @Get(':agentId/state')
  async getState(@Param('agentId') agentId: string) {
    return this.synapseService.getState(agentId);
  }

  @Put(':agentId/state')
  async setState(@Param('agentId') agentId: string, @Body() dto: SetStateDto) {
    return this.synapseService.setState(agentId, dto.state);
  }

  @Post(':agentId/intent')
  @HttpCode(HttpStatus.CREATED)
  async startIntent(@Param('agentId') agentId: string, @Body() dto: StartIntentDto) {
    return this.synapseService.startIntent(agentId, dto.intent, dto.expectedSteps);
  }

  @Post(':agentId/step')
  async recordStep(@Param('agentId') agentId: string, @Body() dto: RecordStepDto) {
    return this.synapseService.recordStep(agentId, dto.action, dto.result);
  }

  @Get(':agentId/drift')
  async checkDrift(@Param('agentId') agentId: string) {
    return this.synapseService.checkDrift(agentId);
  }
}
