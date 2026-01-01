import { ApiProperty, ApiPropertyOptional } from '@nestjs/swagger';
import { IsString, IsNumber, IsOptional, IsBoolean, IsEnum, IsArray, Min } from 'class-validator';

// ============================================================================
// Kill Switch DTOs
// ============================================================================

export class KillSwitchDto {
  @ApiProperty({ description: 'Reason for kill switch activation' })
  @IsString()
  reason: string;

  @ApiPropertyOptional({ description: 'Specific agent ID (omit for global)' })
  @IsOptional()
  @IsString()
  agentId?: string;

  @ApiPropertyOptional({ description: 'Termination type', enum: ['graceful', 'immediate'] })
  @IsOptional()
  @IsEnum(['graceful', 'immediate'])
  type?: 'graceful' | 'immediate';
}

export class KillSwitchResponseDto {
  @ApiProperty({ description: 'Whether activation succeeded' })
  @IsBoolean()
  success: boolean;

  @ApiProperty({ description: 'Kill switch event ID' })
  @IsString()
  killSwitchId: string;

  @ApiProperty({ description: 'Affected agent IDs', type: [String] })
  @IsArray()
  affectedAgents: string[];

  @ApiProperty({ description: 'Activation timestamp' })
  @IsString()
  activatedAt: string;
}

export class KillSwitchStatusDto {
  @ApiProperty({ description: 'Whether kill switch is active' })
  @IsBoolean()
  active: boolean;

  @ApiPropertyOptional({ description: 'Reason for activation' })
  @IsOptional()
  @IsString()
  reason?: string;

  @ApiPropertyOptional({ description: 'Activation timestamp' })
  @IsOptional()
  @IsString()
  activatedAt?: string;
}

// ============================================================================
// Lock DTOs
// ============================================================================

export class AcquireLockDto {
  @ApiProperty({ description: 'Resource ID to lock' })
  @IsString()
  resourceId: string;

  @ApiProperty({ description: 'Agent requesting lock' })
  @IsString()
  agentId: string;

  @ApiPropertyOptional({ description: 'Lock type', enum: ['exclusive', 'shared'] })
  @IsOptional()
  @IsEnum(['exclusive', 'shared'])
  lockType?: 'exclusive' | 'shared';

  @ApiPropertyOptional({ description: 'Lock TTL in seconds' })
  @IsOptional()
  @IsNumber()
  @Min(1)
  ttlSeconds?: number;
}

export class LockResponseDto {
  @ApiProperty({ description: 'Whether lock was acquired' })
  @IsBoolean()
  success: boolean;

  @ApiPropertyOptional({ description: 'Lock ID if acquired' })
  @IsOptional()
  @IsString()
  lockId?: string;

  @ApiProperty({ description: 'Resource ID' })
  @IsString()
  resourceId: string;

  @ApiPropertyOptional({ description: 'Lock expiration timestamp' })
  @IsOptional()
  @IsString()
  expiresAt?: string;

  @ApiPropertyOptional({ description: 'Error message if failed' })
  @IsOptional()
  @IsString()
  error?: string;
}

// ============================================================================
// Escalation DTOs
// ============================================================================

export class EscalationRequestDto {
  @ApiProperty({ description: 'Agent requesting escalation' })
  @IsString()
  agentId: string;

  @ApiProperty({ description: 'Reason for escalation' })
  @IsString()
  reason: string;

  @ApiPropertyOptional({ description: 'Escalation level', enum: ['low', 'medium', 'high', 'critical'] })
  @IsOptional()
  @IsEnum(['low', 'medium', 'high', 'critical'])
  level?: 'low' | 'medium' | 'high' | 'critical';

  @ApiPropertyOptional({ description: 'Context data' })
  @IsOptional()
  context?: Record<string, unknown>;
}

export class EscalationResponseDto {
  @ApiProperty({ description: 'Escalation ID' })
  @IsString()
  id: string;

  @ApiProperty({ description: 'Agent ID' })
  @IsString()
  agentId: string;

  @ApiProperty({ description: 'Escalation reason' })
  @IsString()
  reason: string;

  @ApiProperty({ description: 'Escalation status', enum: ['pending', 'approved', 'rejected'] })
  @IsString()
  status: 'pending' | 'approved' | 'rejected';

  @ApiProperty({ description: 'Creation timestamp' })
  @IsString()
  createdAt: string;

  @ApiPropertyOptional({ description: 'Resolution timestamp' })
  @IsOptional()
  @IsString()
  resolvedAt?: string;

  @ApiPropertyOptional({ description: 'Resolved by (user ID)' })
  @IsOptional()
  @IsString()
  resolvedBy?: string;
}

export class ApproveEscalationDto {
  @ApiProperty({ description: 'Whether to approve' })
  @IsBoolean()
  approved: boolean;

  @ApiProperty({ description: 'Approver user ID' })
  @IsString()
  approvedBy: string;

  @ApiPropertyOptional({ description: 'Approval note' })
  @IsOptional()
  @IsString()
  note?: string;
}

// ============================================================================
// Audit Log DTOs
// ============================================================================

export class AuditEntryDto {
  @ApiProperty({ description: 'Entry ID' })
  @IsString()
  id: string;

  @ApiProperty({ description: 'Action performed' })
  @IsString()
  action: string;

  @ApiProperty({ description: 'Agent ID' })
  @IsString()
  agentId: string;

  @ApiProperty({ description: 'Outcome' })
  @IsString()
  outcome: string;

  @ApiProperty({ description: 'Timestamp' })
  @IsString()
  timestamp: string;
}

export class AuditLogQueryDto {
  @ApiPropertyOptional({ description: 'Filter by agent ID' })
  @IsOptional()
  @IsString()
  agentId?: string;

  @ApiPropertyOptional({ description: 'Filter by action' })
  @IsOptional()
  @IsString()
  action?: string;

  @ApiPropertyOptional({ description: 'Start timestamp' })
  @IsOptional()
  @IsString()
  startTime?: string;

  @ApiPropertyOptional({ description: 'End timestamp' })
  @IsOptional()
  @IsString()
  endTime?: string;

  @ApiPropertyOptional({ description: 'Max entries to return' })
  @IsOptional()
  @IsNumber()
  limit?: number;
}

export class AuditLogResponseDto {
  @ApiProperty({ description: 'Audit entries', type: [AuditEntryDto] })
  @IsArray()
  entries: AuditEntryDto[];

  @ApiProperty({ description: 'Total count' })
  @IsNumber()
  totalCount: number;

  @ApiProperty({ description: 'Has more entries' })
  @IsBoolean()
  hasMore: boolean;
}

// ============================================================================
// Chaos Testing DTOs
// ============================================================================

export class ChaosInjectDto {
  @ApiProperty({ description: 'Chaos type', enum: ['latency', 'error', 'timeout', 'terminate', 'resource_exhaustion'] })
  @IsEnum(['latency', 'error', 'timeout', 'terminate', 'resource_exhaustion'])
  type: 'latency' | 'error' | 'timeout' | 'terminate' | 'resource_exhaustion';

  @ApiProperty({ description: 'Target agent or service' })
  @IsString()
  target: string;

  @ApiPropertyOptional({ description: 'Duration in seconds' })
  @IsOptional()
  @IsNumber()
  durationSeconds?: number;

  @ApiPropertyOptional({ description: 'Probability (0-1) for probabilistic faults' })
  @IsOptional()
  @IsNumber()
  probability?: number;
}

export class ChaosResultDto {
  @ApiProperty({ description: 'Chaos event ID' })
  @IsString()
  chaosId: string;

  @ApiProperty({ description: 'Chaos type' })
  @IsString()
  type: string;

  @ApiProperty({ description: 'Target' })
  @IsString()
  target: string;

  @ApiProperty({ description: 'Injection timestamp' })
  @IsString()
  injectedAt: string;

  @ApiProperty({ description: 'Duration in seconds' })
  @IsNumber()
  duration: number;

  @ApiProperty({ description: 'Whether fault is recoverable' })
  @IsBoolean()
  recoverable: boolean;
}
