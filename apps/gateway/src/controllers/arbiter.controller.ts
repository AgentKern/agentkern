/**
 * AgentKern Gateway - Arbiter Controller
 * 
 * REST API for coordination and traffic control.
 * Endpoints:
 * - POST /arbiter/coordinate - Request coordination for a resource
 * - POST /arbiter/lock - Acquire a lock
 * - DELETE /arbiter/lock - Release a lock
 * - GET /arbiter/lock/:resource - Get lock status
 */

import { Controller, Get, Post, Delete, Body, Param, Query, HttpCode, HttpStatus } from '@nestjs/common';
import { ArbiterService } from '../services/arbiter.service';
import { CoordinateDto, AcquireLockDto, ReleaseLockDto } from '../dto/arbiter.dto';

@Controller('arbiter')
export class ArbiterController {
  constructor(private readonly arbiterService: ArbiterService) {}

  @Post('coordinate')
  @HttpCode(HttpStatus.OK)
  async coordinate(@Body() dto: CoordinateDto) {
    return this.arbiterService.requestCoordination(dto);
  }

  @Post('lock')
  @HttpCode(HttpStatus.CREATED)
  async acquireLock(@Body() dto: AcquireLockDto) {
    return this.arbiterService.acquireLock(dto.agentId, dto.resource, dto.priority);
  }

  @Delete('lock')
  async releaseLock(@Body() dto: ReleaseLockDto) {
    return this.arbiterService.releaseLock(dto.agentId, dto.resource);
  }

  @Get('lock/:resource')
  async getLockStatus(@Param('resource') resource: string) {
    return this.arbiterService.getLockStatus(resource);
  }

  @Get('queue')
  async getQueuePosition(
    @Query('agentId') agentId: string,
    @Query('resource') resource: string,
  ) {
    const position = await this.arbiterService.getQueuePosition(agentId, resource);
    return { agentId, resource, position };
  }
}
