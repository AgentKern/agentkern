/**
 * AgentKernIdentity - Dashboard DTOs
 * 
 * Data Transfer Objects for enterprise dashboard.
 */

import { IsString, IsNotEmpty, IsOptional, IsEnum, IsNumber, IsBoolean, IsArray, ValidateNested, Min, Max } from 'class-validator';
import { Type } from 'class-transformer';
import { ApiProperty, ApiPropertyOptional } from '@nestjs/swagger';

// ============ Policy DTOs ============

export enum PolicyAction {
  ALLOW = 'ALLOW',
  DENY = 'DENY',
  REQUIRE_CONFIRMATION = 'REQUIRE_CONFIRMATION',
  RATE_LIMIT = 'RATE_LIMIT',
}

export class PolicyRuleDto {
  @ApiProperty({ description: 'Rule name' })
  @IsString()
  @IsNotEmpty()
  name: string;

  @ApiProperty({ description: 'Rule condition (JSON expression)' })
  @IsString()
  @IsNotEmpty()
  condition: string;

  @ApiProperty({ enum: PolicyAction, description: 'Action when rule matches' })
  @IsEnum(PolicyAction)
  action: PolicyAction;

  @ApiPropertyOptional({ description: 'Rate limit if action is RATE_LIMIT' })
  @IsOptional()
  @IsNumber()
  rateLimit?: number;
}

export class CreatePolicyRequestDto {
  @ApiProperty({ description: 'Policy name' })
  @IsString()
  @IsNotEmpty()
  name: string;

  @ApiProperty({ description: 'Policy description' })
  @IsString()
  description: string;

  @ApiProperty({ type: [PolicyRuleDto], description: 'Policy rules' })
  @IsArray()
  @ValidateNested({ each: true })
  @Type(() => PolicyRuleDto)
  rules: PolicyRuleDto[];

  @ApiPropertyOptional({ description: 'Target agents (empty = all)' })
  @IsOptional()
  @IsArray()
  targetAgents?: string[];

  @ApiPropertyOptional({ description: 'Target principals (empty = all)' })
  @IsOptional()
  @IsArray()
  targetPrincipals?: string[];
}

export class PolicyResponseDto {
  @ApiProperty({ description: 'Policy ID' })
  id: string;

  @ApiProperty({ description: 'Policy name' })
  name: string;

  @ApiProperty({ description: 'Policy description' })
  description: string;

  @ApiProperty({ type: [PolicyRuleDto], description: 'Policy rules' })
  rules: PolicyRuleDto[];

  @ApiProperty({ description: 'Whether policy is active' })
  active: boolean;

  @ApiProperty({ description: 'Creation timestamp' })
  createdAt: string;

  @ApiProperty({ description: 'Last update timestamp' })
  updatedAt: string;
}

// ============ Dashboard DTOs ============

export class DashboardStatsResponseDto {
  @ApiProperty({ description: 'Total verifications today' })
  verificationsToday: number;

  @ApiProperty({ description: 'Success rate percentage' })
  successRate: number;

  @ApiProperty({ description: 'Active agents' })
  activeAgents: number;

  @ApiProperty({ description: 'Active principals' })
  activePrincipals: number;

  @ApiProperty({ description: 'Revocations today' })
  revocationsToday: number;

  @ApiProperty({ description: 'Mesh connected peers' })
  connectedPeers: number;

  @ApiProperty({ description: 'Average trust score' })
  averageTrustScore: number;
}

export class VerificationTrendDto {
  @ApiProperty({ description: 'Date' })
  date: string;

  @ApiProperty({ description: 'Success count' })
  success: number;

  @ApiProperty({ description: 'Failure count' })
  failure: number;
}

export class TopAgentDto {
  @ApiProperty({ description: 'Agent ID' })
  agentId: string;

  @ApiProperty({ description: 'Agent name' })
  agentName: string;

  @ApiProperty({ description: 'Verification count' })
  verificationCount: number;

  @ApiProperty({ description: 'Trust score' })
  trustScore: number;
}

// ============ Compliance DTOs ============

export class ComplianceReportRequestDto {
  @ApiProperty({ description: 'Start date' })
  @IsString()
  @IsNotEmpty()
  startDate: string;

  @ApiProperty({ description: 'End date' })
  @IsString()
  @IsNotEmpty()
  endDate: string;

  @ApiPropertyOptional({ description: 'Filter by agent' })
  @IsOptional()
  @IsString()
  agentId?: string;

  @ApiPropertyOptional({ description: 'Filter by principal' })
  @IsOptional()
  @IsString()
  principalId?: string;
}

export class ComplianceReportResponseDto {
  @ApiProperty({ description: 'Report ID' })
  id: string;

  @ApiProperty({ description: 'Report period' })
  period: { start: string; end: string };

  @ApiProperty({ description: 'Total verifications' })
  totalVerifications: number;

  @ApiProperty({ description: 'Successful verifications' })
  successfulVerifications: number;

  @ApiProperty({ description: 'Failed verifications' })
  failedVerifications: number;

  @ApiProperty({ description: 'Revocations' })
  revocations: number;

  @ApiProperty({ description: 'Policy violations' })
  policyViolations: number;

  @ApiProperty({ description: 'Generated timestamp' })
  generatedAt: string;
}
