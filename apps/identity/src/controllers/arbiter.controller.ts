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
  Logger,
} from '@nestjs/common';
import { ApiTags, ApiOperation, ApiResponse, ApiQuery } from '@nestjs/swagger';
import {
  KillSwitchDto,
  KillSwitchResponseDto,
  KillSwitchStatusDto,
  AcquireLockDto,
  LockResponseDto,
  EscalationRequestDto,
  EscalationResponseDto,
  ApproveEscalationDto,
  AuditLogResponseDto,
  ChaosInjectDto,
  ChaosResultDto,
} from '../dto/arbiter.dto';
import { ArbiterService } from '../services/arbiter.service';

/**
 * Arbiter Controller - Governance & Coordination API
 *
 * Exposes the Arbiter pillar's capabilities via Rust N-API bridge:
 * - Kill switch (emergency agent termination)
 * - Distributed locking (Raft consensus)
 * - Human-in-the-loop escalation
 * - ISO 42001 audit logging
 * - Chaos testing injection
 */
@ApiTags('Arbiter')
@Controller('api/v1/arbiter')
export class ArbiterController {
  private readonly logger = new Logger(ArbiterController.name);

  // In-memory stores for features not yet in bridge
  private locks: Map<
    string,
    { resourceId: string; agentId: string; acquired: string; ttl: number }
  > = new Map();
  private escalations: Map<
    string,
    {
      id: string;
      agentId: string;
      reason: string;
      status: string;
      createdAt: string;
    }
  > = new Map();

  constructor(private readonly arbiterService: ArbiterService) {}

  // =========================================================================
  // Kill Switch Endpoints (Rust Bridge)
  // =========================================================================

  /**
   * Activate the kill switch (emergency agent termination)
   */
  @Post('killswitch')
  @HttpCode(HttpStatus.OK)
  @ApiOperation({ summary: 'Activate kill switch' })
  @ApiResponse({
    status: 200,
    description: 'Kill switch activated',
    type: KillSwitchResponseDto,
  })
  async activateKillSwitch(
    @Body() dto: KillSwitchDto,
  ): Promise<KillSwitchResponseDto> {
    this.logger.error(`🚨 KILL SWITCH ACTIVATED: ${dto.reason}`);

    const result = await this.arbiterService.activateKillSwitch(
      dto.reason,
      dto.agentId,
    );

    if ('error' in result) {
      this.logger.error(`Kill switch activation failed: ${result.error}`);
      return {
        success: false,
        killSwitchId: `ks_${Date.now()}`,
        affectedAgents: [],
        activatedAt: new Date().toISOString(),
      };
    }

    return {
      success: result.success,
      killSwitchId: result.id,
      affectedAgents: [result.target_id],
      activatedAt: result.timestamp,
    };
  }

  /**
   * Get kill switch status
   */
  @Get('killswitch/status')
  @ApiOperation({ summary: 'Get kill switch status' })
  @ApiResponse({
    status: 200,
    description: 'Kill switch status',
    type: KillSwitchStatusDto,
  })
  async getKillSwitchStatus(): Promise<KillSwitchStatusDto> {
    const result = await this.arbiterService.getKillSwitchStatus();

    return {
      active: result.active,
      reason: result.active ? 'Emergency shutdown active' : undefined,
      activatedAt: result.active ? new Date().toISOString() : undefined,
    };
  }

  /**
   * Deactivate the kill switch
   */
  @Delete('killswitch')
  @HttpCode(HttpStatus.OK)
  @ApiOperation({ summary: 'Deactivate kill switch' })
  @ApiResponse({ status: 200, description: 'Kill switch deactivated' })
  async deactivateKillSwitch(): Promise<{
    success: boolean;
    deactivatedAt: string;
  }> {
    this.logger.log('Kill switch deactivated');

    const result = await this.arbiterService.deactivateKillSwitch();

    return {
      success: !result.active,
      deactivatedAt: new Date().toISOString(),
    };
  }

  // =========================================================================
  // Lock Management Endpoints (Local - Bridge extension needed)
  // =========================================================================

  /**
   * Acquire a distributed lock
   * Note: Raft-based locking requires bridge extension.
   */
  @Post('locks')
  @HttpCode(HttpStatus.CREATED)
  @ApiOperation({ summary: 'Acquire a distributed lock' })
  @ApiResponse({
    status: 201,
    description: 'Lock acquired',
    type: LockResponseDto,
  })
  @ApiResponse({ status: 409, description: 'Lock already held' })
  async acquireLock(@Body() dto: AcquireLockDto): Promise<LockResponseDto> {
    this.logger.log(
      `Acquiring lock on ${dto.resourceId} for agent ${dto.agentId}`,
    );

    // TODO: Extend Rust Arbiter to support distributed locking via Raft
    if (this.locks.has(dto.resourceId)) {
      return {
        success: false,
        lockId: undefined,
        resourceId: dto.resourceId,
        error: 'Resource already locked',
      };
    }

    const lockId = `lock_${Date.now()}`;
    this.locks.set(dto.resourceId, {
      resourceId: dto.resourceId,
      agentId: dto.agentId,
      acquired: new Date().toISOString(),
      ttl: dto.ttlSeconds || 30,
    });

    return {
      success: true,
      lockId,
      resourceId: dto.resourceId,
      expiresAt: new Date(
        Date.now() + (dto.ttlSeconds || 30) * 1000,
      ).toISOString(),
    };
  }

  /**
   * Release a lock
   */
  @Delete('locks/:resourceId')
  @HttpCode(HttpStatus.OK)
  @ApiOperation({ summary: 'Release a distributed lock' })
  @ApiResponse({ status: 200, description: 'Lock released' })
  @ApiResponse({ status: 404, description: 'Lock not found' })
  async releaseLock(
    @Param('resourceId') resourceId: string,
  ): Promise<{ success: boolean; releasedAt: string }> {
    this.logger.log(`Releasing lock on ${resourceId}`);

    this.locks.delete(resourceId);

    return {
      success: true,
      releasedAt: new Date().toISOString(),
    };
  }

  // =========================================================================
  // Escalation Endpoints (Local - Bridge extension needed)
  // =========================================================================

  /**
   * Create an escalation request (human-in-the-loop)
   * Note: Escalation workflow requires bridge extension.
   */
  @Post('escalation/request')
  @HttpCode(HttpStatus.CREATED)
  @ApiOperation({ summary: 'Create escalation request' })
  @ApiResponse({
    status: 201,
    description: 'Escalation created',
    type: EscalationResponseDto,
  })
  async createEscalation(
    @Body() dto: EscalationRequestDto,
  ): Promise<EscalationResponseDto> {
    this.logger.log(
      `Escalation requested for agent ${dto.agentId}: ${dto.reason}`,
    );

    // TODO: Extend Rust Arbiter to support escalation workflow
    const id = `esc_${Date.now()}`;
    this.escalations.set(id, {
      id,
      agentId: dto.agentId,
      reason: dto.reason,
      status: 'pending',
      createdAt: new Date().toISOString(),
    });

    return {
      id,
      agentId: dto.agentId,
      reason: dto.reason,
      status: 'pending',
      createdAt: new Date().toISOString(),
    };
  }

  /**
   * Approve or reject an escalation
   */
  @Post('escalation/:id/approve')
  @HttpCode(HttpStatus.OK)
  @ApiOperation({ summary: 'Approve or reject escalation' })
  @ApiResponse({
    status: 200,
    description: 'Escalation resolved',
    type: EscalationResponseDto,
  })
  async resolveEscalation(
    @Param('id') escalationId: string,
    @Body() dto: ApproveEscalationDto,
  ): Promise<EscalationResponseDto> {
    const escalation = this.escalations.get(escalationId);

    if (!escalation) {
      throw new Error('Escalation not found');
    }

    escalation.status = dto.approved ? 'approved' : 'rejected';
    this.escalations.set(escalationId, escalation);

    this.logger.log(
      `Escalation ${escalationId} ${escalation.status} by ${dto.approvedBy}`,
    );

    return {
      id: escalation.id,
      agentId: escalation.agentId,
      reason: escalation.reason,
      status: escalation.status as 'pending' | 'approved' | 'rejected',
      createdAt: escalation.createdAt,
      resolvedAt: new Date().toISOString(),
      resolvedBy: dto.approvedBy,
    };
  }

  // =========================================================================
  // Audit Log Endpoints (Rust Bridge)
  // =========================================================================

  /**
   * Query audit log (ISO 42001 compliance)
   */
  @Get('audit')
  @ApiOperation({ summary: 'Query audit log' })
  @ApiQuery({ name: 'agentId', required: false })
  @ApiQuery({ name: 'action', required: false })
  @ApiQuery({ name: 'limit', required: false, type: Number })
  @ApiResponse({
    status: 200,
    description: 'Audit log entries',
    type: AuditLogResponseDto,
  })
  async queryAuditLog(
    @Query('agentId') _agentId?: string,
    @Query('action') _action?: string,
    @Query('limit') limit?: number,
  ): Promise<AuditLogResponseDto> {
    const stats = await this.arbiterService.getAuditStatistics(limit);

    if (!stats) {
      return {
        entries: [],
        totalCount: 0,
        hasMore: false,
      };
    }

    // Bridge returns statistics, not individual entries
    // For full audit trail, we'd need to extend the bridge
    return {
      entries: [],
      totalCount: stats.total_records,
      hasMore: stats.total_records > (limit || 100),
      statistics: {
        approved: stats.approved_count,
        denied: stats.denied_count,
        inReview: stats.review_count,
        logged: stats.logged_count,
        highRisk: stats.high_risk_count,
        avgRiskScore: stats.avg_risk_score,
      },
    };
  }

  // =========================================================================
  // Chaos Testing Endpoints (Rust Bridge)
  // =========================================================================

  /**
   * Inject chaos for testing (fault injection)
   * Note: Returns chaos stats; actual injection requires bridge extension.
   */
  @Post('chaos/inject')
  @HttpCode(HttpStatus.OK)
  @ApiOperation({ summary: 'Inject chaos event for testing' })
  @ApiResponse({
    status: 200,
    description: 'Chaos injected',
    type: ChaosResultDto,
  })
  async injectChaos(@Body() dto: ChaosInjectDto): Promise<ChaosResultDto> {
    this.logger.warn(`⚡ CHAOS INJECTED: ${dto.type} targeting ${dto.target}`);

    // Get chaos stats from bridge
    const stats = this.arbiterService.getChaosStats();

    return {
      chaosId: `chaos_${Date.now()}`,
      type: dto.type,
      target: dto.target,
      injectedAt: new Date().toISOString(),
      duration: dto.durationSeconds || 60,
      recoverable: dto.type !== 'terminate',
      stats: {
        totalOps: stats.total_ops,
        latencyInjections: stats.latency_injections,
        errorInjections: stats.error_injections,
      },
    };
  }
}
