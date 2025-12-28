/**
 * AgentKernIdentity - Mesh DTOs
 * 
 * Data Transfer Objects for Mesh endpoints.
 */

import { IsString, IsNotEmpty, IsOptional, IsEnum, IsArray, IsNumber } from 'class-validator';
import { ApiProperty, ApiPropertyOptional } from '@nestjs/swagger';
import { MeshNodeType } from '../domain/mesh.entity';

export class ConnectPeerRequestDto {
  @ApiProperty({ description: 'Peer WebSocket endpoint' })
  @IsString()
  @IsNotEmpty()
  endpoint: string;
}

export class MeshNodeResponseDto {
  @ApiProperty({ description: 'Node ID' })
  id: string;

  @ApiProperty({ description: 'Node public key' })
  publicKey: string;

  @ApiProperty({ enum: MeshNodeType, description: 'Node type' })
  type: MeshNodeType;

  @ApiProperty({ description: 'Node endpoints' })
  endpoints: string[];

  @ApiProperty({ description: 'Node capabilities' })
  capabilities: string[];

  @ApiProperty({ description: 'Trust score' })
  trustScore: number;
}

export class MeshStatsResponseDto {
  @ApiProperty({ description: 'Local node ID' })
  nodeId: string;

  @ApiProperty({ enum: MeshNodeType, description: 'Node type' })
  nodeType: MeshNodeType;

  @ApiProperty({ description: 'Number of connected peers' })
  connectedPeers: number;

  @ApiProperty({ description: 'Total processed messages' })
  processedMessages: number;

  @ApiProperty({ description: 'Node uptime in seconds' })
  uptime: number;
}

export class BroadcastTrustUpdateRequestDto {
  @ApiProperty({ description: 'Agent ID' })
  @IsString()
  @IsNotEmpty()
  agentId: string;

  @ApiProperty({ description: 'Principal ID' })
  @IsString()
  @IsNotEmpty()
  principalId: string;

  @ApiProperty({ description: 'Trust score' })
  @IsNumber()
  trustScore: number;

  @ApiProperty({ description: 'Event type' })
  @IsString()
  @IsNotEmpty()
  event: string;

  @ApiPropertyOptional({ description: 'Previous score' })
  @IsOptional()
  previousScore?: number;
}

export class BroadcastRevocationRequestDto {
  @ApiProperty({ description: 'Agent ID' })
  @IsString()
  @IsNotEmpty()
  agentId: string;

  @ApiProperty({ description: 'Principal ID' })
  @IsString()
  @IsNotEmpty()
  principalId: string;

  @ApiProperty({ description: 'Revocation reason' })
  @IsString()
  @IsNotEmpty()
  reason: string;
}
