import { ApiProperty, ApiPropertyOptional } from '@nestjs/swagger';
import {
  IsString,
  IsNumber,
  IsOptional,
  IsArray,
  IsObject,
  IsEnum,
} from 'class-validator';

// ============================================================================
// State DTOs
// ============================================================================

export class AgentStateDto {
  @ApiProperty({ description: 'Agent ID' })
  @IsString()
  agentId: string;

  @ApiProperty({ description: 'Agent state (key-value pairs)' })
  @IsObject()
  state: Record<string, unknown>;

  @ApiProperty({ description: 'State version (for CRDT)' })
  @IsNumber()
  version: number;

  @ApiProperty({ description: 'Last update timestamp' })
  @IsString()
  lastUpdated: string;
}

export class UpdateStateDto {
  @ApiProperty({ description: 'State updates (merged with existing)' })
  @IsObject()
  state: Record<string, unknown>;

  @ApiPropertyOptional({ description: 'Expected version (optimistic locking)' })
  @IsOptional()
  @IsNumber()
  version?: number;
}

// ============================================================================
// Memory Passport DTOs
// ============================================================================

export class MemoryPassportDto {
  @ApiProperty({ description: 'Passport ID' })
  @IsString()
  id: string;

  @ApiProperty({ description: 'Agent ID' })
  @IsString()
  agentId: string;

  @ApiProperty({ description: 'Memory layers included', type: [String] })
  @IsArray()
  layers: string[];

  @ApiProperty({ description: 'Passport format version' })
  @IsString()
  version: string;

  @ApiProperty({ description: 'Creation timestamp' })
  @IsString()
  createdAt: string;

  @ApiPropertyOptional({ description: 'Expiration timestamp' })
  @IsOptional()
  @IsString()
  expiresAt?: string;
}

export class CreatePassportDto {
  @ApiProperty({ description: 'Agent ID' })
  @IsString()
  agentId: string;

  @ApiPropertyOptional({
    description: 'Memory layers to include',
    type: [String],
  })
  @IsOptional()
  @IsArray()
  @IsString({ each: true })
  layers?: string[];

  @ApiPropertyOptional({ description: 'Encryption key for passport' })
  @IsOptional()
  @IsString()
  encryptionKey?: string;
}

export class ExportPassportDto {
  @ApiProperty({ description: 'Passport ID to export' })
  @IsString()
  passportId: string;

  @ApiProperty({
    description: 'Export format',
    enum: ['json', 'cbor', 'protobuf'],
  })
  @IsEnum(['json', 'cbor', 'protobuf'])
  format: 'json' | 'cbor' | 'protobuf';

  @ApiPropertyOptional({ description: 'Include encryption' })
  @IsOptional()
  encrypted?: boolean;
}

// ============================================================================
// Context Guard DTOs
// ============================================================================

export class ContextGuardDto {
  @ApiProperty({ description: 'Documents to analyze', type: [String] })
  @IsArray()
  @IsString({ each: true })
  documents: string[];

  @ApiPropertyOptional({ description: 'Original query for context' })
  @IsOptional()
  @IsString()
  query?: string;

  @ApiPropertyOptional({ description: 'Sensitivity threshold (0-1)' })
  @IsOptional()
  @IsNumber()
  threshold?: number;
}

export class ContextThreatDto {
  @ApiProperty({ description: 'Threat type' })
  @IsString()
  type: string;

  @ApiProperty({
    description: 'Threat severity',
    enum: ['low', 'medium', 'high', 'critical'],
  })
  @IsString()
  severity: string;

  @ApiProperty({ description: 'Snippet of threatening content' })
  @IsString()
  content: string;
}

export class ContextGuardResultDto {
  @ApiProperty({ description: 'Whether context is safe' })
  safe: boolean;

  @ApiProperty({ description: 'Number of documents analyzed' })
  @IsNumber()
  analyzedDocuments: number;

  @ApiProperty({ description: 'Detected threats', type: [ContextThreatDto] })
  @IsArray()
  threats: ContextThreatDto[];

  @ApiProperty({ description: 'Processing time in milliseconds' })
  @IsNumber()
  processingTimeMs: number;
}

// ============================================================================
// Graph Query DTOs
// ============================================================================

export class GraphQueryDto {
  @ApiProperty({ description: 'Natural language or vector query' })
  @IsString()
  query: string;

  @ApiPropertyOptional({ description: 'Node types to search', type: [String] })
  @IsOptional()
  @IsArray()
  @IsString({ each: true })
  nodeTypes?: string[];

  @ApiPropertyOptional({ description: 'Maximum results to return' })
  @IsOptional()
  @IsNumber()
  limit?: number;

  @ApiPropertyOptional({ description: 'Minimum similarity threshold (0-1)' })
  @IsOptional()
  @IsNumber()
  minSimilarity?: number;
}

export class GraphNodeResultDto {
  @ApiProperty({ description: 'Node ID' })
  @IsString()
  nodeId: string;

  @ApiProperty({ description: 'Node type' })
  @IsString()
  type: string;

  @ApiProperty({ description: 'Similarity score (0-1)' })
  @IsNumber()
  similarity: number;

  @ApiProperty({ description: 'Node data' })
  @IsObject()
  data: Record<string, unknown>;
}

export class GraphQueryResultDto {
  @ApiProperty({ description: 'Query results', type: [GraphNodeResultDto] })
  @IsArray()
  results: GraphNodeResultDto[];

  @ApiProperty({ description: 'Total matching results' })
  @IsNumber()
  totalResults: number;

  @ApiProperty({ description: 'Query execution time in milliseconds' })
  @IsNumber()
  queryTimeMs: number;
}
