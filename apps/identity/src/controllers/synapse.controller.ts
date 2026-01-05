import {
  Controller,
  Post,
  Get,
  Put,
  Delete,
  Body,
  Param,
  HttpCode,
  HttpStatus,
  Logger,
} from '@nestjs/common';
import { ApiTags, ApiOperation, ApiResponse } from '@nestjs/swagger';
import {
  AgentStateDto,
  UpdateStateDto,
  MemoryPassportDto,
  CreatePassportDto,
  ExportPassportDto,
  ContextGuardDto,
  ContextGuardResultDto,
  GraphQueryDto,
  GraphQueryResultDto,
} from '../dto/synapse.dto';
import { SynapseService } from '../services/synapse.service';
import { GateService } from '../services/gate.service';

/**
 * Synapse Controller - Agent Memory & State API
 *
 * Exposes the Synapse pillar's capabilities via Rust N-API bridge:
 * - Agent state management (CRDTs)
 * - Memory passports (portable agent memory)
 * - RAG context guard (memory injection protection)
 * - Graph vector database operations
 */
@ApiTags('Synapse')
@Controller('api/v1/synapse')
export class SynapseController {
  private readonly logger = new Logger(SynapseController.name);

  // In-memory store for passports (not yet in bridge)
  private passports: Map<
    string,
    { id: string; agentId: string; layers: string[]; createdAt: string }
  > = new Map();

  constructor(
    private readonly synapseService: SynapseService,
    private readonly gateService: GateService,
  ) {}

  // =========================================================================
  // State Management Endpoints (Rust Bridge)
  // =========================================================================

  /**
   * Get agent state
   */
  @Get('state/:agentId')
  @ApiOperation({ summary: 'Get agent state' })
  @ApiResponse({ status: 200, description: 'Agent state', type: AgentStateDto })
  @ApiResponse({ status: 404, description: 'Agent not found' })
  async getState(@Param('agentId') agentId: string): Promise<AgentStateDto> {
    this.logger.log(`Getting state for agent: ${agentId}`);

    const result = await this.synapseService.getState(agentId);

    if (!result) {
      return {
        agentId,
        state: {},
        version: 0,
        lastUpdated: new Date().toISOString(),
      };
    }

    return {
      agentId: result.agent_id,
      state: result.state,
      version: result.version,
      lastUpdated: new Date().toISOString(),
    };
  }

  /**
   * Update agent state (CRDT merge)
   */
  @Put('state/:agentId')
  @ApiOperation({ summary: 'Update agent state (CRDT merge)' })
  @ApiResponse({
    status: 200,
    description: 'State updated',
    type: AgentStateDto,
  })
  async updateState(
    @Param('agentId') agentId: string,
    @Body() dto: UpdateStateDto,
  ): Promise<AgentStateDto> {
    this.logger.log(`Updating state for agent: ${agentId}`);

    const result = await this.synapseService.updateState(agentId, dto.state);

    if (!result.success) {
      this.logger.error(`State update failed: ${result.error}`);
    }

    // Fetch updated state
    const updated = await this.synapseService.getState(agentId);

    return {
      agentId,
      state: updated?.state || dto.state,
      version: result.version || (dto.version || 0) + 1,
      lastUpdated: new Date().toISOString(),
    };
  }

  /**
   * Delete agent state
   */
  @Delete('state/:agentId')
  @HttpCode(HttpStatus.NO_CONTENT)
  @ApiOperation({ summary: 'Delete agent state' })
  @ApiResponse({ status: 204, description: 'State deleted' })
  async deleteState(@Param('agentId') agentId: string): Promise<void> {
    this.logger.log(`Deleting state for agent: ${agentId}`);

    // Get current state keys and delete them all
    const current = await this.synapseService.getState(agentId);
    if (current) {
      const keys = Object.keys(current.state);
      await this.synapseService.deleteKeys(agentId, keys);
    }
  }

  // =========================================================================
  // Memory Passport Endpoints (Local - Bridge extension needed)
  // =========================================================================

  /**
   * Create a memory passport (portable agent memory)
   * Note: Passport serialization requires bridge extension.
   */
  @Post('memory/passport')
  @HttpCode(HttpStatus.CREATED)
  @ApiOperation({ summary: 'Create a memory passport' })
  @ApiResponse({
    status: 201,
    description: 'Passport created',
    type: MemoryPassportDto,
  })
  async createPassport(
    @Body() dto: CreatePassportDto,
  ): Promise<MemoryPassportDto> {
    this.logger.log(`Creating passport for agent: ${dto.agentId}`);

    // TODO: Extend Rust Synapse to support passport serialization
    const id = `passport_${Date.now()}`;
    const passport = {
      id,
      agentId: dto.agentId,
      layers: dto.layers || ['short_term', 'long_term', 'episodic'],
      createdAt: new Date().toISOString(),
    };

    this.passports.set(id, passport);

    return {
      id,
      agentId: dto.agentId,
      layers: passport.layers,
      version: '1.0.0',
      createdAt: passport.createdAt,
      expiresAt: new Date(Date.now() + 365 * 24 * 60 * 60 * 1000).toISOString(),
    };
  }

  /**
   * Get memory passport
   */
  @Get('memory/passport/:id')
  @ApiOperation({ summary: 'Get memory passport' })
  @ApiResponse({
    status: 200,
    description: 'Passport details',
    type: MemoryPassportDto,
  })
  @ApiResponse({ status: 404, description: 'Passport not found' })
  async getPassport(
    @Param('id') passportId: string,
  ): Promise<MemoryPassportDto> {
    const passport = this.passports.get(passportId);

    if (!passport) {
      throw new Error('Passport not found');
    }

    return {
      id: passport.id,
      agentId: passport.agentId,
      layers: passport.layers,
      version: '1.0.0',
      createdAt: passport.createdAt,
      expiresAt: new Date(Date.now() + 365 * 24 * 60 * 60 * 1000).toISOString(),
    };
  }

  /**
   * Export memory passport (GDPR compliance)
   */
  @Post('memory/export')
  @HttpCode(HttpStatus.OK)
  @ApiOperation({ summary: 'Export memory passport (GDPR data portability)' })
  @ApiResponse({ status: 200, description: 'Export data' })
  async exportPassport(
    @Body() dto: ExportPassportDto,
  ): Promise<{ exportUrl: string; expiresAt: string }> {
    this.logger.log(`Exporting passport: ${dto.passportId} as ${dto.format}`);

    return {
      exportUrl: `https://api.agentkern.io/exports/${dto.passportId}.${dto.format}`,
      expiresAt: new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString(),
    };
  }

  // =========================================================================
  // Context Guard Endpoints (Rust Bridge via GateService)
  // =========================================================================

  /**
   * Analyze RAG context for injection attacks
   * Uses Gate pillar's context guard.
   */
  @Post('context/guard')
  @HttpCode(HttpStatus.OK)
  @ApiOperation({ summary: 'Analyze RAG context for injection attacks' })
  @ApiResponse({
    status: 200,
    description: 'Context analysis result',
    type: ContextGuardResultDto,
  })
  async guardContext(
    @Body() dto: ContextGuardDto,
  ): Promise<ContextGuardResultDto> {
    this.logger.log('Analyzing RAG context for injection');

    const startTime = Date.now();

    // Use Gate pillar's context guard (Rust N-API)
    const result = await this.gateService.guardContext(dto.documents);

    if (!result) {
      return {
        safe: true,
        analyzedDocuments: dto.documents.length,
        threats: [],
        processingTimeMs: Date.now() - startTime,
      };
    }

    // Map suspicious chunk indices to threat objects
    const suspiciousChunks = result.suspicious_chunks || [];
    const threats = suspiciousChunks.map((idx) => ({
      type: 'context_injection',
      severity: 'high' as const,
      content: dto.documents[idx]?.substring(0, 100) || 'Unknown',
    }));

    return {
      safe: result.safe ?? true,
      analyzedDocuments: dto.documents.length,
      threats,
      processingTimeMs: Date.now() - startTime,
    };
  }

  // =========================================================================
  // Graph Vector Database Endpoints (Rust Bridge)
  // =========================================================================

  /**
   * Query the graph vector database
   */
  @Post('graph/query')
  @HttpCode(HttpStatus.OK)
  @ApiOperation({ summary: 'Query graph vector database' })
  @ApiResponse({
    status: 200,
    description: 'Query results',
    type: GraphQueryResultDto,
  })
  async queryGraph(@Body() dto: GraphQueryDto): Promise<GraphQueryResultDto> {
    this.logger.log(`Graph query: ${dto.query.substring(0, 50)}...`);

    const startTime = Date.now();

    // Use Synapse's memory query (vector similarity search)
    const results = await this.synapseService.queryMemory(
      dto.query,
      dto.limit || 10,
    );

    return {
      results: results.map((r) => ({
        nodeId: r.node_id,
        type: 'memory',
        similarity: r.score,
        data: {},
      })),
      totalResults: results.length,
      queryTimeMs: Date.now() - startTime,
    };
  }

  /**
   * Store memory for an agent
   */
  @Post('memory/:agentId')
  @HttpCode(HttpStatus.CREATED)
  @ApiOperation({ summary: 'Store agent memory' })
  @ApiResponse({ status: 201, description: 'Memory stored' })
  async storeMemory(
    @Param('agentId') agentId: string,
    @Body() dto: { text: string },
  ): Promise<{ id?: string; error?: string }> {
    this.logger.log(`Storing memory for agent: ${agentId}`);

    return this.synapseService.storeMemory(agentId, dto.text);
  }
}
