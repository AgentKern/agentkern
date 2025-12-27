/**
 * AgentKern Gateway - Arbiter DTOs
 */

export class CoordinateDto {
  agentId!: string;
  resource!: string;
  operation!: 'read' | 'write' | 'exclusive';
  expectedDurationMs!: number;
  priority!: number;
}

export class AcquireLockDto {
  agentId!: string;
  resource!: string;
  priority?: number;
}

export class ReleaseLockDto {
  agentId!: string;
  resource!: string;
}
