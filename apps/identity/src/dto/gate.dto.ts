import { ApiProperty, ApiPropertyOptional } from '@nestjs/swagger';
import { IsString, IsOptional, IsArray, IsBoolean, IsNumber, IsObject } from 'class-validator';

// ============================================================================
// Prompt Guard DTOs
// ============================================================================

export class GuardPromptDto {
  @ApiProperty({ description: 'The prompt to analyze for injection attacks' })
  @IsString()
  prompt: string;

  @ApiPropertyOptional({ description: 'Optional context for analysis' })
  @IsOptional()
  @IsString()
  context?: string;

  @ApiPropertyOptional({ description: 'Specific policies to apply', type: [String] })
  @IsOptional()
  @IsArray()
  @IsString({ each: true })
  policies?: string[];
}

export class GuardPromptResponseDto {
  @ApiProperty({ description: 'Whether the prompt is considered safe' })
  @IsBoolean()
  safe: boolean;

  @ApiProperty({ description: 'Threat level', enum: ['none', 'low', 'medium', 'high', 'critical'] })
  @IsString()
  threatLevel: 'none' | 'low' | 'medium' | 'high' | 'critical';

  @ApiPropertyOptional({ description: 'Type of attack detected' })
  @IsOptional()
  @IsString()
  threatType?: string;

  @ApiProperty({ description: 'Confidence score (0-100)' })
  @IsNumber()
  score: number;

  @ApiPropertyOptional({ description: 'Reason for classification' })
  @IsOptional()
  @IsString()
  reason?: string;
}

// ============================================================================
// Policy DTOs
// ============================================================================

export class PolicyRuleDto {
  @ApiProperty({ description: 'Rule ID' })
  @IsString()
  id: string;

  @ApiProperty({ description: 'Rule condition (DSL expression)' })
  @IsString()
  condition: string;

  @ApiProperty({ description: 'Action to take', enum: ['allow', 'deny', 'audit', 'escalate'] })
  @IsString()
  action: 'allow' | 'deny' | 'audit' | 'escalate';

  @ApiPropertyOptional({ description: 'Priority (higher = first)' })
  @IsOptional()
  @IsNumber()
  priority?: number;
}

export class PolicyDto {
  @ApiProperty({ description: 'Policy ID' })
  @IsString()
  id: string;

  @ApiProperty({ description: 'Policy name' })
  @IsString()
  name: string;

  @ApiPropertyOptional({ description: 'Policy description' })
  @IsOptional()
  @IsString()
  description?: string;

  @ApiProperty({ description: 'Whether policy is active' })
  @IsBoolean()
  active: boolean;

  @ApiProperty({ description: 'Policy rules', type: [PolicyRuleDto] })
  @IsArray()
  rules: PolicyRuleDto[];

  @ApiProperty({ description: 'Created timestamp' })
  @IsString()
  createdAt: string;

  @ApiPropertyOptional({ description: 'Last updated timestamp' })
  @IsOptional()
  @IsString()
  updatedAt?: string;
}

export class CreatePolicyDto {
  @ApiProperty({ description: 'Policy name' })
  @IsString()
  name: string;

  @ApiPropertyOptional({ description: 'Policy description' })
  @IsOptional()
  @IsString()
  description?: string;

  @ApiProperty({ description: 'Policy rules', type: [PolicyRuleDto] })
  @IsArray()
  rules: PolicyRuleDto[];
}

// ============================================================================
// Compliance DTOs
// ============================================================================

export class ComplianceCheckDto {
  @ApiProperty({ description: 'Data to check for compliance' })
  @IsObject()
  data: Record<string, unknown>;

  @ApiPropertyOptional({ description: 'Additional context for check' })
  @IsOptional()
  @IsObject()
  context?: Record<string, unknown>;
}

export class ComplianceIssueDto {
  @ApiProperty({ description: 'Issue code' })
  @IsString()
  code: string;

  @ApiProperty({ description: 'Issue severity', enum: ['info', 'warning', 'error', 'critical'] })
  @IsString()
  severity: 'info' | 'warning' | 'error' | 'critical';

  @ApiProperty({ description: 'Issue description' })
  @IsString()
  message: string;

  @ApiPropertyOptional({ description: 'Field or path where issue was found' })
  @IsOptional()
  @IsString()
  path?: string;
}

export class ComplianceResultDto {
  @ApiProperty({ description: 'Whether data is compliant' })
  @IsBoolean()
  compliant: boolean;

  @ApiProperty({ description: 'Compliance standard checked' })
  @IsString()
  standard: string;

  @ApiProperty({ description: 'Issues found', type: [ComplianceIssueDto] })
  @IsArray()
  issues: ComplianceIssueDto[];

  @ApiProperty({ description: 'Check timestamp' })
  @IsString()
  checkedAt: string;
}

// ============================================================================
// WASM Actor DTOs
// ============================================================================

export class WasmCapabilityDto {
  @ApiProperty({ description: 'Capability name' })
  @IsString()
  name: string;

  @ApiPropertyOptional({ description: 'Input schema (JSON Schema)' })
  @IsOptional()
  @IsObject()
  inputSchema?: Record<string, unknown>;

  @ApiPropertyOptional({ description: 'Output schema (JSON Schema)' })
  @IsOptional()
  @IsObject()
  outputSchema?: Record<string, unknown>;
}

export class WasmActorDto {
  @ApiProperty({ description: 'Actor name' })
  @IsString()
  name: string;

  @ApiProperty({ description: 'Actor version' })
  @IsString()
  version: string;

  @ApiProperty({ description: 'Declared capabilities', type: [WasmCapabilityDto] })
  @IsArray()
  capabilities: WasmCapabilityDto[];

  @ApiProperty({ description: 'Module size in bytes' })
  @IsNumber()
  sizeBytes: number;

  @ApiProperty({ description: 'Load timestamp' })
  @IsString()
  loadedAt: string;

  @ApiProperty({ description: 'Total invocation count' })
  @IsNumber()
  invocations: number;

  @ApiProperty({ description: 'Average latency in microseconds' })
  @IsNumber()
  avgLatencyUs: number;
}

export class RegisterWasmActorDto {
  @ApiProperty({ description: 'Actor name' })
  @IsString()
  name: string;

  @ApiProperty({ description: 'Actor version (semver)' })
  @IsString()
  version: string;

  @ApiProperty({ description: 'Base64-encoded WASM module' })
  @IsString()
  wasmBase64: string;

  @ApiProperty({ description: 'Declared capabilities', type: [WasmCapabilityDto] })
  @IsArray()
  capabilities: WasmCapabilityDto[];
}
