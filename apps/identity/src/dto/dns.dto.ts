/**
 * AgentProof - DNS DTOs with Validation
 * 
 * Data Transfer Objects for Intent DNS endpoints.
 */

import {
  IsString,
  IsNotEmpty,
  IsOptional,
  IsArray,
  ValidateNested,
  IsBoolean,
} from 'class-validator';
import { Type } from 'class-transformer';
import { ApiProperty, ApiPropertyOptional } from '@nestjs/swagger';

// ============ Request DTOs ============

export class ResolveQueryDto {
  @ApiProperty({ description: 'Agent identifier' })
  @IsString()
  @IsNotEmpty()
  agentId: string;

  @ApiProperty({ description: 'Principal identifier' })
  @IsString()
  @IsNotEmpty()
  principalId: string;
}

export class ResolveBatchRequestDto {
  @ApiProperty({ type: [ResolveQueryDto], description: 'Batch of queries' })
  @IsArray()
  @ValidateNested({ each: true })
  @Type(() => ResolveQueryDto)
  queries: ResolveQueryDto[];
}

export class RegisterTrustRequestDto {
  @ApiProperty({ description: 'Agent identifier' })
  @IsString()
  @IsNotEmpty()
  agentId: string;

  @ApiProperty({ description: 'Principal identifier' })
  @IsString()
  @IsNotEmpty()
  principalId: string;

  @ApiPropertyOptional({ description: 'Agent name' })
  @IsOptional()
  @IsString()
  agentName?: string;

  @ApiPropertyOptional({ description: 'Agent version' })
  @IsOptional()
  @IsString()
  agentVersion?: string;
}

export class RevokeTrustRequestDto {
  @ApiProperty({ description: 'Agent identifier' })
  @IsString()
  @IsNotEmpty()
  agentId: string;

  @ApiProperty({ description: 'Principal identifier' })
  @IsString()
  @IsNotEmpty()
  principalId: string;

  @ApiProperty({ description: 'Reason for revocation' })
  @IsString()
  @IsNotEmpty()
  reason: string;
}

// ============ Response DTOs ============

export class TrustResolutionResponseDto {
  @ApiProperty({ description: 'Protocol version' })
  version: string;

  @ApiProperty({ description: 'Agent identifier' })
  agentId: string;

  @ApiProperty({ description: 'Principal identifier' })
  principalId: string;

  @ApiProperty({ description: 'Whether agent is trusted' })
  trusted: boolean;

  @ApiProperty({ description: 'Trust score (0-1000)' })
  trustScore: number;

  @ApiProperty({ description: 'Trust expiry timestamp' })
  expiresAt: string;

  @ApiProperty({ description: 'Whether trust is revoked' })
  revoked: boolean;

  @ApiProperty({ description: 'When response was cached' })
  cachedAt: string;

  @ApiProperty({ description: 'Cache TTL in seconds' })
  ttl: number;
}

export class TrustRecordResponseDto {
  @ApiProperty({ description: 'Record ID' })
  id: string;

  @ApiProperty({ description: 'Agent identifier' })
  agentId: string;

  @ApiProperty({ description: 'Principal identifier' })
  principalId: string;

  @ApiProperty({ description: 'Trust score (0-1000)' })
  trustScore: number;

  @ApiProperty({ description: 'Whether agent is trusted' })
  trusted: boolean;

  @ApiProperty({ description: 'Whether trust is revoked' })
  revoked: boolean;

  @ApiProperty({ description: 'Registration timestamp' })
  registeredAt: string;

  @ApiProperty({ description: 'Last verification timestamp' })
  lastVerifiedAt: string;

  @ApiProperty({ description: 'Successful verification count' })
  verificationCount: number;

  @ApiProperty({ description: 'Failed verification count' })
  failureCount: number;
}
