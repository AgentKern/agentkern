/**
 * AgentKern Identity - API DTOs with Validation
 * 
 * Data Transfer Objects with comprehensive validation.
 * Follows mandate: input validation, no shortcuts.
 */

import {
  IsString,
  IsNotEmpty,
  IsOptional,
  IsNumber,
  IsArray,
  IsEnum,
  IsObject,
  ValidateNested,
  Min,
  Max,
  IsUUID,
  IsISO8601,
  Matches,
} from 'class-validator';
import { Type } from 'class-transformer';
import { ApiProperty, ApiPropertyOptional } from '@nestjs/swagger';

// ============ Enums ============

export enum HttpMethod {
  GET = 'GET',
  POST = 'POST',
  PUT = 'PUT',
  DELETE = 'DELETE',
  PATCH = 'PATCH',
}

// ============ Nested DTOs ============

export class PrincipalDto {
  @ApiProperty({ description: 'Unique identifier for the principal (user)' })
  @IsString()
  @IsNotEmpty()
  id: string;

  @ApiProperty({ description: 'WebAuthn credential ID (Passkey)' })
  @IsString()
  @IsNotEmpty()
  credentialId: string;

  @ApiPropertyOptional({ description: 'Device attestation hash' })
  @IsOptional()
  @IsString()
  deviceAttestation?: string;
}

export class AgentDto {
  @ApiProperty({ description: 'Unique identifier for the AI agent' })
  @IsString()
  @IsNotEmpty()
  id: string;

  @ApiProperty({ description: 'Agent name' })
  @IsString()
  @IsNotEmpty()
  name: string;

  @ApiProperty({ description: 'Agent version' })
  @IsString()
  @Matches(/^\d+\.\d+\.\d+$/, { message: 'Version must be semver format (e.g., 1.0.0)' })
  version: string;
}

export class IntentTargetDto {
  @ApiProperty({ description: 'Target service URL or identifier' })
  @IsString()
  @IsNotEmpty()
  service: string;

  @ApiProperty({ description: 'Target endpoint path' })
  @IsString()
  @IsNotEmpty()
  endpoint: string;

  @ApiProperty({ enum: HttpMethod, description: 'HTTP method' })
  @IsEnum(HttpMethod)
  method: HttpMethod;
}

export class IntentDto {
  @ApiProperty({ description: 'Action type (e.g., transfer, delete, access)' })
  @IsString()
  @IsNotEmpty()
  action: string;

  @ApiProperty({ type: IntentTargetDto, description: 'Target of the action' })
  @ValidateNested()
  @Type(() => IntentTargetDto)
  target: IntentTargetDto;

  @ApiPropertyOptional({ description: 'Action parameters' })
  @IsOptional()
  @IsObject()
  parameters?: Record<string, unknown>;
}

export class ConstraintsDto {
  @ApiPropertyOptional({ description: 'Maximum amount allowed' })
  @IsOptional()
  @IsNumber()
  @Min(0)
  maxAmount?: number;

  @ApiPropertyOptional({ description: 'Allowed recipient identifiers' })
  @IsOptional()
  @IsArray()
  @IsString({ each: true })
  allowedRecipients?: string[];

  @ApiPropertyOptional({ description: 'Allowed geographic regions (ISO country codes)' })
  @IsOptional()
  @IsArray()
  @IsString({ each: true })
  geoFence?: string[];

  @ApiPropertyOptional({ description: 'Valid hours range (UTC)' })
  @IsOptional()
  @IsObject()
  validHours?: { start: number; end: number };

  @ApiPropertyOptional({ description: 'Amount threshold requiring confirmation' })
  @IsOptional()
  @IsNumber()
  @Min(0)
  requireConfirmationAbove?: number;

  @ApiPropertyOptional({ description: 'Whether proof can only be used once' })
  @IsOptional()
  singleUse?: boolean;
}

// ============ Request DTOs ============

export class VerifyProofRequestDto {
  @ApiProperty({ description: 'X-AgentKern Identity header value' })
  @IsString()
  @IsNotEmpty()
  proof: string;
}

export class CreateProofRequestDto {
  @ApiProperty({ type: PrincipalDto, description: 'The authorizing principal' })
  @ValidateNested()
  @Type(() => PrincipalDto)
  principal: PrincipalDto;

  @ApiProperty({ type: AgentDto, description: 'The AI agent being authorized' })
  @ValidateNested()
  @Type(() => AgentDto)
  agent: AgentDto;

  @ApiProperty({ type: IntentDto, description: 'The intent being authorized' })
  @ValidateNested()
  @Type(() => IntentDto)
  intent: IntentDto;

  @ApiPropertyOptional({ type: ConstraintsDto, description: 'Authorization constraints' })
  @IsOptional()
  @ValidateNested()
  @Type(() => ConstraintsDto)
  constraints?: ConstraintsDto;

  @ApiPropertyOptional({ description: 'Proof validity in seconds (default: 300)' })
  @IsOptional()
  @IsNumber()
  @Min(60)
  @Max(86400) // Max 24 hours
  expiresInSeconds?: number;
}

export class RegisterKeyRequestDto {
  @ApiProperty({ description: 'Principal identifier' })
  @IsString()
  @IsNotEmpty()
  principalId: string;

  @ApiProperty({ description: 'WebAuthn credential ID' })
  @IsString()
  @IsNotEmpty()
  credentialId: string;

  @ApiProperty({ description: 'Public key in PEM format' })
  @IsString()
  @IsNotEmpty()
  publicKey: string;

  @ApiPropertyOptional({ description: 'Signing algorithm (default: ES256)' })
  @IsOptional()
  @IsString()
  algorithm?: string;
}

// ============ Response DTOs ============

export class VerifyProofResponseDto {
  @ApiProperty({ description: 'Whether the proof is valid' })
  valid: boolean;

  @ApiPropertyOptional({ description: 'Proof ID' })
  proofId?: string;

  @ApiPropertyOptional({ description: 'Principal who authorized' })
  principalId?: string;

  @ApiPropertyOptional({ description: 'Agent that was authorized' })
  agentId?: string;

  @ApiPropertyOptional({ description: 'Authorized intent' })
  intent?: { action: string; target: string };

  @ApiPropertyOptional({ description: 'Who accepted liability' })
  liabilityAcceptedBy?: string;

  @ApiPropertyOptional({ description: 'Validation errors' })
  errors?: string[];
}

export class CreateProofResponseDto {
  @ApiProperty({ description: 'The X-AgentKern Identity header value' })
  header: string;

  @ApiProperty({ description: 'Proof ID for tracking' })
  proofId: string;

  @ApiProperty({ description: 'When the proof expires' })
  expiresAt: string;
}

export class AuditEventResponseDto {
  @ApiProperty({ description: 'Event ID' })
  id: string;

  @ApiProperty({ description: 'Event timestamp' })
  timestamp: string;

  @ApiProperty({ description: 'Event type' })
  type: string;

  @ApiProperty({ description: 'Whether the event was successful' })
  success: boolean;

  @ApiPropertyOptional({ description: 'Principal ID' })
  principalId?: string;

  @ApiPropertyOptional({ description: 'Agent ID' })
  agentId?: string;

  @ApiPropertyOptional({ description: 'Proof ID' })
  proofId?: string;

  @ApiPropertyOptional({ description: 'Error message if failed' })
  errorMessage?: string;
}
