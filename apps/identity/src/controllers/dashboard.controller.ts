/**
 * AgentKernIdentity - Dashboard Controller
 *
 * Enterprise dashboard API for monitoring and compliance.
 * Policy management is handled by Rust Gate package.
 * 🔒 Enterprise-only features - requires LICENSE_KEY
 */

import {
  Controller,
  Get,
  Post,
  Body,
  Query,
  HttpCode,
  HttpStatus,
  UseGuards,
} from '@nestjs/common';
import {
  ApiTags,
  ApiOperation,
  ApiResponse,
} from '@nestjs/swagger';
import { AuditLoggerService, AuditEvent, AuditEventType } from '../services/audit-logger.service';
import { LicenseGuard, EnterpriseOnly, RequireFeature } from '../guards/license.guard';
import {
  DashboardStatsResponseDto,
  VerificationTrendDto,
  TopAgentDto,
  ComplianceReportRequestDto,
  ComplianceReportResponseDto,
} from '../dto/dashboard.dto';

@ApiTags('Dashboard')
@Controller('api/v1/dashboard')
@UseGuards(LicenseGuard)
export class DashboardController {
  constructor(
    private readonly auditLogger: AuditLoggerService,
  ) {}

  // ============ Root Endpoint ============

  @Get()
  @ApiOperation({ summary: 'Dashboard root', description: 'Lists available dashboard endpoints.' })
  getRoot() {
    return {
      name: 'AgentKernIdentity Dashboard API',
      endpoints: {
        stats: 'GET /api/v1/dashboard/stats',
        trends: 'GET /api/v1/dashboard/trends',
        topAgents: 'GET /api/v1/dashboard/top-agents',
        compliance: 'POST /api/v1/dashboard/compliance/report',
        auditTrail: 'GET /api/v1/dashboard/compliance/audit-trail',
      },
      note: 'Policy management is available via the Rust Gate service.',
    };
  }

  // ============ Stats Endpoints ============

  @Get('stats')
  @EnterpriseOnly()
  @RequireFeature('dashboard.stats')
  @ApiOperation({
    summary: 'Get dashboard statistics',
    description: '🔒 Enterprise - Returns key metrics for the enterprise dashboard.',
  })
  @ApiResponse({ status: 200, description: 'Dashboard stats', type: DashboardStatsResponseDto })
  async getStats(): Promise<DashboardStatsResponseDto> {
    const events = await this.auditLogger.getRecentEvents(1000);
    const today = new Date().toISOString().split('T')[0];

    const todayEvents = events.filter((e: AuditEvent) => e.timestamp.startsWith(today));
    const verifications = todayEvents.filter((e: AuditEvent) =>
      e.type === AuditEventType.PROOF_VERIFICATION_SUCCESS ||
      e.type === AuditEventType.PROOF_VERIFICATION_FAILURE
    );

    const successCount = verifications.filter((e: AuditEvent) => e.success).length;
    const totalCount = verifications.length;

    const revocations = todayEvents.filter((e: AuditEvent) => e.type === AuditEventType.KEY_REVOKED).length;

    const uniqueAgents = new Set(events.map((e: AuditEvent) => e.agentId).filter(Boolean));
    const uniquePrincipals = new Set(events.map((e: AuditEvent) => e.principalId).filter(Boolean));

    return {
      verificationsToday: totalCount,
      successRate: totalCount > 0 ? Math.round((successCount / totalCount) * 100) : 100,
      activeAgents: uniqueAgents.size,
      activePrincipals: uniquePrincipals.size,
      revocationsToday: revocations,
      connectedPeers: 0,
      averageTrustScore: 750,
    };
  }

  @Get('trends')
  @ApiOperation({
    summary: 'Get verification trends',
    description: 'Returns verification success/failure trends over time.',
  })
  @ApiResponse({ status: 200, description: 'Verification trends', type: [VerificationTrendDto] })
  async getTrends(@Query('days') days: number = 7): Promise<VerificationTrendDto[]> {
    const trends: VerificationTrendDto[] = [];
    const events = await this.auditLogger.getRecentEvents(10000);

    for (let i = days - 1; i >= 0; i--) {
      const date = new Date();
      date.setDate(date.getDate() - i);
      const dateStr = date.toISOString().split('T')[0];

      const dayEvents = events.filter((e: AuditEvent) =>
        e.timestamp.startsWith(dateStr) &&
        (e.type === AuditEventType.PROOF_VERIFICATION_SUCCESS ||
         e.type === AuditEventType.PROOF_VERIFICATION_FAILURE)
      );

      trends.push({
        date: dateStr,
        success: dayEvents.filter((e: AuditEvent) => e.success).length,
        failure: dayEvents.filter((e: AuditEvent) => !e.success).length,
      });
    }

    return trends;
  }

  @Get('top-agents')
  @ApiOperation({
    summary: 'Get top agents',
    description: 'Returns the most active agents by verification count.',
  })
  @ApiResponse({ status: 200, description: 'Top agents', type: [TopAgentDto] })
  async getTopAgents(@Query('limit') limit: number = 10): Promise<TopAgentDto[]> {
    const events = await this.auditLogger.getRecentEvents(10000);

    const agentCounts = new Map<string, { count: number; name: string }>();

    for (const event of events) {
      if (!event.agentId) continue;

      const current = agentCounts.get(event.agentId) || { count: 0, name: event.agentId };
      agentCounts.set(event.agentId, { ...current, count: current.count + 1 });
    }

    return Array.from(agentCounts.entries())
      .map(([agentId, data]) => ({
        agentId,
        agentName: data.name,
        verificationCount: data.count,
        trustScore: 750,
      }))
      .sort((a, b) => b.verificationCount - a.verificationCount)
      .slice(0, limit);
  }

  // ============ Compliance Endpoints ============

  @Post('compliance/report')
  @HttpCode(HttpStatus.OK)
  @ApiOperation({
    summary: 'Generate compliance report',
    description: 'Generates a compliance report for a date range.',
  })
  @ApiResponse({ status: 200, description: 'Compliance report', type: ComplianceReportResponseDto })
  async generateComplianceReport(@Body() dto: ComplianceReportRequestDto): Promise<ComplianceReportResponseDto> {
    const startDate = new Date(dto.startDate);
    const endDate = new Date(dto.endDate);

    // Use the exportAuditLog method which supports date filtering
    const filteredEvents = await this.auditLogger.exportAuditLog({
      startDate,
      endDate,
    });

    // Further filter by agent/principal if specified
    const events = filteredEvents.filter((e: AuditEvent) => {
      if (dto.agentId && e.agentId !== dto.agentId) return false;
      if (dto.principalId && e.principalId !== dto.principalId) return false;
      return true;
    });

    const verifications = events.filter((e: AuditEvent) =>
      e.type === AuditEventType.PROOF_VERIFICATION_SUCCESS ||
      e.type === AuditEventType.PROOF_VERIFICATION_FAILURE
    );

    const successful = verifications.filter((e: AuditEvent) => e.success).length;
    const failed = verifications.filter((e: AuditEvent) => !e.success).length;
    const revocations = events.filter((e: AuditEvent) => e.type === AuditEventType.KEY_REVOKED).length;
    const violations = events.filter((e: AuditEvent) => e.type === AuditEventType.SECURITY_ALERT).length;

    return {
      id: crypto.randomUUID(),
      period: { start: dto.startDate, end: dto.endDate },
      totalVerifications: verifications.length,
      successfulVerifications: successful,
      failedVerifications: failed,
      revocations,
      policyViolations: violations,
      generatedAt: new Date().toISOString(),
    };
  }

  @Get('compliance/audit-trail')
  @ApiOperation({
    summary: 'Get audit trail',
    description: 'Returns the complete audit trail for compliance.',
  })
  @ApiResponse({ status: 200, description: 'Audit trail' })
  async getAuditTrail(
    @Query('limit') limit: number = 100,
    @Query('type') type?: string,
  ): Promise<AuditEvent[]> {
    const events = await this.auditLogger.getRecentEvents(limit);

    if (type) {
      return events.filter((e: AuditEvent) => e.type === type);
    }

    return events;
  }
}
