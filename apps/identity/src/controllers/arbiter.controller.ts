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
  AuditLogQueryDto,
  AuditLogResponseDto,
  ChaosInjectDto,
  ChaosResultDto,
} from '../dto/arbiter.dto';

/**
 * Arbiter Controller - Governance & Coordination API
 * 
 * Exposes the Arbiter pillar's capabilities:
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

  // In-memory stores (would use Rust bridge in production)
  private killSwitchActive = false;
  private killSwitchReason = '';
  private locks: Map<string, { resourceId: string; agentId: string; acquired: string; ttl: number }> = new Map();
  private escalations: Map<string, { id: string; agentId: string; reason: string; status: string; createdAt: string }> = new Map();
  private auditLog: Array<{ id: string; action: string; agentId: string; outcome: string; timestamp: string }> = [];

  // =========================================================================
  // Kill Switch Endpoints
  // =========================================================================

  /**
   * Activate the kill switch (emergency agent termination)
   */
  @Post('killswitch')
  @HttpCode(HttpStatus.OK)
  @ApiOperation({ summary: 'Activate kill switch' })
  @ApiResponse({ status: 200, description: 'Kill switch activated', type: KillSwitchResponseDto })
  async activateKillSwitch(@Body() dto: KillSwitchDto): Promise<KillSwitchResponseDto> {
    this.logger.error(`🚨 KILL SWITCH ACTIVATED: ${dto.reason}`);
    
    this.killSwitchActive = true;
    this.killSwitchReason = dto.reason;
    
    // Log to audit
    this.auditLog.push({
      id: `audit_${Date.now()}`,
      action: 'KILL_SWITCH_ACTIVATED',
      agentId: dto.agentId || 'GLOBAL',
      outcome: 'success',
      timestamp: new Date().toISOString(),
    });
    
    return {
      success: true,
      killSwitchId: `ks_${Date.now()}`,
      affectedAgents: dto.agentId ? [dto.agentId] : ['ALL'],
      activatedAt: new Date().toISOString(),
    };
  }

  /**
   * Get kill switch status
   */
  @Get('killswitch/status')
  @ApiOperation({ summary: 'Get kill switch status' })
  @ApiResponse({ status: 200, description: 'Kill switch status', type: KillSwitchStatusDto })
  async getKillSwitchStatus(): Promise<KillSwitchStatusDto> {
    return {
      active: this.killSwitchActive,
      reason: this.killSwitchReason || undefined,
      activatedAt: this.killSwitchActive ? new Date().toISOString() : undefined,
    };
  }

  /**
   * Deactivate the kill switch
   */
  @Delete('killswitch')
  @HttpCode(HttpStatus.OK)
  @ApiOperation({ summary: 'Deactivate kill switch' })
  @ApiResponse({ status: 200, description: 'Kill switch deactivated' })
  async deactivateKillSwitch(): Promise<{ success: boolean; deactivatedAt: string }> {
    this.logger.log('Kill switch deactivated');
    
    this.killSwitchActive = false;
    this.killSwitchReason = '';
    
    this.auditLog.push({
      id: `audit_${Date.now()}`,
      action: 'KILL_SWITCH_DEACTIVATED',
      agentId: 'GLOBAL',
      outcome: 'success',
      timestamp: new Date().toISOString(),
    });
    
    return {
      success: true,
      deactivatedAt: new Date().toISOString(),
    };
  }

  // =========================================================================
  // Lock Management Endpoints
  // =========================================================================

  /**
   * Acquire a distributed lock
   */
  @Post('locks')
  @HttpCode(HttpStatus.CREATED)
  @ApiOperation({ summary: 'Acquire a distributed lock' })
  @ApiResponse({ status: 201, description: 'Lock acquired', type: LockResponseDto })
  @ApiResponse({ status: 409, description: 'Lock already held' })
  async acquireLock(@Body() dto: AcquireLockDto): Promise<LockResponseDto> {
    this.logger.log(`Acquiring lock on ${dto.resourceId} for agent ${dto.agentId}`);
    
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
      expiresAt: new Date(Date.now() + (dto.ttlSeconds || 30) * 1000).toISOString(),
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
  async releaseLock(@Param('resourceId') resourceId: string): Promise<{ success: boolean; releasedAt: string }> {
    this.logger.log(`Releasing lock on ${resourceId}`);
    
    this.locks.delete(resourceId);
    
    return {
      success: true,
      releasedAt: new Date().toISOString(),
    };
  }

  // =========================================================================
  // Escalation Endpoints
  // =========================================================================

  /**
   * Create an escalation request (human-in-the-loop)
   */
  @Post('escalation/request')
  @HttpCode(HttpStatus.CREATED)
  @ApiOperation({ summary: 'Create escalation request' })
  @ApiResponse({ status: 201, description: 'Escalation created', type: EscalationResponseDto })
  async createEscalation(@Body() dto: EscalationRequestDto): Promise<EscalationResponseDto> {
    this.logger.log(`Escalation requested for agent ${dto.agentId}: ${dto.reason}`);
    
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
  @ApiResponse({ status: 200, description: 'Escalation resolved', type: EscalationResponseDto })
  async resolveEscalation(
    @Param('id') id: string,
    @Body() dto: ApproveEscalationDto,
  ): Promise<EscalationResponseDto> {
    const escalation = this.escalations.get(id);
    
    if (!escalation) {
      throw new Error('Escalation not found');
    }
    
    escalation.status = dto.approved ? 'approved' : 'rejected';
    this.escalations.set(id, escalation);
    
    this.logger.log(`Escalation ${id} ${escalation.status} by ${dto.approvedBy}`);
    
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
  // Audit Log Endpoints
  // =========================================================================

  /**
   * Query audit log (ISO 42001 compliance)
   */
  @Get('audit')
  @ApiOperation({ summary: 'Query audit log' })
  @ApiQuery({ name: 'agentId', required: false })
  @ApiQuery({ name: 'action', required: false })
  @ApiQuery({ name: 'limit', required: false, type: Number })
  @ApiResponse({ status: 200, description: 'Audit log entries', type: AuditLogResponseDto })
  async queryAuditLog(
    @Query('agentId') agentId?: string,
    @Query('action') action?: string,
    @Query('limit') limit?: number,
  ): Promise<AuditLogResponseDto> {
    let entries = [...this.auditLog];
    
    if (agentId) {
      entries = entries.filter(e => e.agentId === agentId);
    }
    if (action) {
      entries = entries.filter(e => e.action === action);
    }
    
    entries = entries.slice(0, limit || 100);
    
    return {
      entries,
      totalCount: entries.length,
      hasMore: this.auditLog.length > (limit || 100),
    };
  }

  // =========================================================================
  // Chaos Testing Endpoints
  // =========================================================================

  /**
   * Inject chaos for testing (fault injection)
   */
  @Post('chaos/inject')
  @HttpCode(HttpStatus.OK)
  @ApiOperation({ summary: 'Inject chaos event for testing' })
  @ApiResponse({ status: 200, description: 'Chaos injected', type: ChaosResultDto })
  async injectChaos(@Body() dto: ChaosInjectDto): Promise<ChaosResultDto> {
    this.logger.warn(`⚡ CHAOS INJECTED: ${dto.type} targeting ${dto.target}`);
    
    this.auditLog.push({
      id: `audit_${Date.now()}`,
      action: `CHAOS_${dto.type.toUpperCase()}`,
      agentId: dto.target,
      outcome: 'injected',
      timestamp: new Date().toISOString(),
    });
    
    return {
      chaosId: `chaos_${Date.now()}`,
      type: dto.type,
      target: dto.target,
      injectedAt: new Date().toISOString(),
      duration: dto.durationSeconds || 60,
      recoverable: dto.type !== 'terminate',
    };
  }
}
