/**
 * AgentKern Identity - Dashboard Controller
 * 
 * Enterprise dashboard API for monitoring, policy management, and compliance.
 * 🔒 Enterprise-only features - requires LICENSE_KEY
 */

import {
  Controller,
  Get,
  Post,
  Put,
  Delete,
  Body,
  Param,
  Query,
  HttpCode,
  HttpStatus,
  NotFoundException,
  UseGuards,
} from '@nestjs/common';
import {
  ApiTags,
  ApiOperation,
  ApiResponse,
} from '@nestjs/swagger';
import { PolicyService } from '../services/policy.service';
import { AuditLoggerService, AuditEventType } from '../services/audit-logger.service';
import { LicenseGuard, EnterpriseOnly, RequireFeature } from '../guards/license.guard';
import {
  CreatePolicyRequestDto,
  PolicyResponseDto,
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
    private readonly policyService: PolicyService,
    private readonly auditLogger: AuditLoggerService,
  ) {}

  // ============ Root Endpoint ============

  @Get()
  @ApiOperation({ summary: 'Dashboard root', description: 'Lists available dashboard endpoints.' })
  getRoot() {
    return {
      name: 'AgentKern Identity Dashboard API',
      endpoints: {
        stats: 'GET /api/v1/dashboard/stats',
        trends: 'GET /api/v1/dashboard/trends',
        topAgents: 'GET /api/v1/dashboard/top-agents',
        policies: 'GET /api/v1/dashboard/policies',
        compliance: 'POST /api/v1/dashboard/compliance/report',
        auditTrail: 'GET /api/v1/dashboard/compliance/audit-trail',
      },
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
  getStats(): DashboardStatsResponseDto {
    const events = this.auditLogger.getRecentEvents(1000);
    const today = new Date().toISOString().split('T')[0];
    
    const todayEvents = events.filter((e: any) => e.timestamp.startsWith(today));
    const verifications = todayEvents.filter((e: any) => 
      e.type === AuditEventType.PROOF_VERIFICATION_SUCCESS || 
      e.type === AuditEventType.PROOF_VERIFICATION_FAILURE
    );
    
    const successCount = verifications.filter((e: any) => e.success).length;
    const totalCount = verifications.length;
    
    const revocations = todayEvents.filter((e: any) => e.type === AuditEventType.KEY_REVOKED).length;
    
    const uniqueAgents = new Set(events.map((e: any) => e.agentId).filter(Boolean));
    const uniquePrincipals = new Set(events.map((e: any) => e.principalId).filter(Boolean));

    return {
      verificationsToday: totalCount,
      successRate: totalCount > 0 ? Math.round((successCount / totalCount) * 100) : 100,
      activeAgents: uniqueAgents.size,
      activePrincipals: uniquePrincipals.size,
      revocationsToday: revocations,
      connectedPeers: 0, // Would come from MeshNodeService
      averageTrustScore: 750, // Would calculate from DNS records
    };
  }

  @Get('trends')
  @ApiOperation({
    summary: 'Get verification trends',
    description: 'Returns verification success/failure trends over time.',
  })
  @ApiResponse({ status: 200, description: 'Verification trends', type: [VerificationTrendDto] })
  getTrends(@Query('days') days: number = 7): VerificationTrendDto[] {
    const trends: VerificationTrendDto[] = [];
    const events = this.auditLogger.getRecentEvents(10000);

    for (let i = days - 1; i >= 0; i--) {
      const date = new Date();
      date.setDate(date.getDate() - i);
      const dateStr = date.toISOString().split('T')[0];

      const dayEvents = events.filter(e => 
        e.timestamp.startsWith(dateStr) &&
        (e.type === AuditEventType.PROOF_VERIFICATION_SUCCESS || 
         e.type === AuditEventType.PROOF_VERIFICATION_FAILURE)
      );

      trends.push({
        date: dateStr,
        success: dayEvents.filter(e => e.success).length,
        failure: dayEvents.filter(e => !e.success).length,
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
  getTopAgents(@Query('limit') limit: number = 10): TopAgentDto[] {
    const events = this.auditLogger.getRecentEvents(10000);
    
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
        trustScore: 750, // Would come from DNS records
      }))
      .sort((a, b) => b.verificationCount - a.verificationCount)
      .slice(0, limit);
  }

  // ============ Policy Endpoints ============

  @Get('policies')
  @ApiOperation({
    summary: 'Get all policies',
    description: 'Returns all configured policies.',
  })
  @ApiResponse({ status: 200, description: 'List of policies', type: [PolicyResponseDto] })
  getPolicies(): PolicyResponseDto[] {
    return this.policyService.getAllPolicies().map(p => ({
      id: p.id,
      name: p.name,
      description: p.description,
      rules: p.rules,
      active: p.active,
      createdAt: p.createdAt,
      updatedAt: p.updatedAt,
    }));
  }

  @Get('policies/:id')
  @ApiOperation({
    summary: 'Get policy by ID',
    description: 'Returns a specific policy.',
  })
  @ApiResponse({ status: 200, description: 'Policy details', type: PolicyResponseDto })
  @ApiResponse({ status: 404, description: 'Policy not found' })
  getPolicy(@Param('id') id: string): PolicyResponseDto {
    const policy = this.policyService.getPolicy(id);
    if (!policy) {
      throw new NotFoundException('Policy not found');
    }
    return {
      id: policy.id,
      name: policy.name,
      description: policy.description,
      rules: policy.rules,
      active: policy.active,
      createdAt: policy.createdAt,
      updatedAt: policy.updatedAt,
    };
  }

  @Post('policies')
  @HttpCode(HttpStatus.CREATED)
  @ApiOperation({
    summary: 'Create a policy',
    description: 'Creates a new policy.',
  })
  @ApiResponse({ status: 201, description: 'Policy created', type: PolicyResponseDto })
  createPolicy(@Body() dto: CreatePolicyRequestDto): PolicyResponseDto {
    const policy = this.policyService.createPolicy(
      dto.name,
      dto.description,
      dto.rules,
      dto.targetAgents,
      dto.targetPrincipals,
    );
    return {
      id: policy.id,
      name: policy.name,
      description: policy.description,
      rules: policy.rules,
      active: policy.active,
      createdAt: policy.createdAt,
      updatedAt: policy.updatedAt,
    };
  }

  @Put('policies/:id/activate')
  @ApiOperation({
    summary: 'Activate a policy',
    description: 'Activates a policy.',
  })
  @ApiResponse({ status: 200, description: 'Policy activated' })
  activatePolicy(@Param('id') id: string): { success: boolean } {
    const policy = this.policyService.setActive(id, true);
    if (!policy) {
      throw new NotFoundException('Policy not found');
    }
    return { success: true };
  }

  @Put('policies/:id/deactivate')
  @ApiOperation({
    summary: 'Deactivate a policy',
    description: 'Deactivates a policy.',
  })
  @ApiResponse({ status: 200, description: 'Policy deactivated' })
  deactivatePolicy(@Param('id') id: string): { success: boolean } {
    const policy = this.policyService.setActive(id, false);
    if (!policy) {
      throw new NotFoundException('Policy not found');
    }
    return { success: true };
  }

  @Delete('policies/:id')
  @ApiOperation({
    summary: 'Delete a policy',
    description: 'Deletes a policy.',
  })
  @ApiResponse({ status: 200, description: 'Policy deleted' })
  deletePolicy(@Param('id') id: string): { success: boolean } {
    const deleted = this.policyService.deletePolicy(id);
    if (!deleted) {
      throw new NotFoundException('Policy not found');
    }
    return { success: true };
  }

  // ============ Compliance Endpoints ============

  @Post('compliance/report')
  @HttpCode(HttpStatus.OK)
  @ApiOperation({
    summary: 'Generate compliance report',
    description: 'Generates a compliance report for a date range.',
  })
  @ApiResponse({ status: 200, description: 'Compliance report', type: ComplianceReportResponseDto })
  generateComplianceReport(@Body() dto: ComplianceReportRequestDto): ComplianceReportResponseDto {
    const events = this.auditLogger.getRecentEvents(100000);
    
    const startDate = new Date(dto.startDate);
    const endDate = new Date(dto.endDate);
    
    const filteredEvents = events.filter(e => {
      const eventDate = new Date(e.timestamp);
      if (eventDate < startDate || eventDate > endDate) return false;
      if (dto.agentId && e.agentId !== dto.agentId) return false;
      if (dto.principalId && e.principalId !== dto.principalId) return false;
      return true;
    });

    const verifications = filteredEvents.filter(e =>
      e.type === AuditEventType.PROOF_VERIFICATION_SUCCESS ||
      e.type === AuditEventType.PROOF_VERIFICATION_FAILURE
    );

    const successful = verifications.filter(e => e.success).length;
    const failed = verifications.filter(e => !e.success).length;
    const revocations = filteredEvents.filter(e => e.type === AuditEventType.KEY_REVOKED).length;
    const violations = filteredEvents.filter(e => e.type === AuditEventType.SECURITY_ALERT).length;

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
  getAuditTrail(
    @Query('limit') limit: number = 100,
    @Query('type') type?: string,
  ) {
    const events = this.auditLogger.getRecentEvents(limit);
    
    if (type) {
      return events.filter(e => e.type === type);
    }
    
    return events;
  }
}
