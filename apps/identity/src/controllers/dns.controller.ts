/**
 * AgentKernIdentity - DNS Controller
 * 
 * REST API endpoints for Intent DNS resolution.
 */

import {
  Controller,
  Get,
  Post,
  Body,
  Query,
  Param,
  HttpCode,
  HttpStatus,
  NotFoundException,
} from '@nestjs/common';
import {
  ApiTags,
  ApiOperation,
  ApiResponse,
  ApiQuery,
} from '@nestjs/swagger';
import { DnsResolutionService } from '../services/dns-resolution.service';
import {
  ResolveQueryDto,
  ResolveBatchRequestDto,
  RegisterTrustRequestDto,
  RevokeTrustRequestDto,
  TrustResolutionResponseDto,
  TrustRecordResponseDto,
} from '../dto/dns.dto';

@ApiTags('Intent DNS')
@Controller('api/v1/dns')
export class DnsController {
  constructor(private readonly dnsService: DnsResolutionService) {}

  @Get('resolve')
  @ApiOperation({
    summary: 'Resolve trust for an agent-principal pair',
    description: 'Returns cached trust resolution. Creates new record if not exists.',
  })
  @ApiQuery({ name: 'agentId', description: 'Agent identifier' })
  @ApiQuery({ name: 'principalId', description: 'Principal identifier' })
  @ApiResponse({ status: 200, description: 'Trust resolution', type: TrustResolutionResponseDto })
  async resolve(
    @Query('agentId') agentId: string,
    @Query('principalId') principalId: string,
  ): Promise<TrustResolutionResponseDto> {
    return await this.dnsService.resolve({ agentId, principalId });
  }

  @Post('resolve/batch')
  @HttpCode(HttpStatus.OK)
  @ApiOperation({
    summary: 'Batch resolve multiple agent-principal pairs',
    description: 'Efficiently resolves multiple trust queries in a single request.',
  })
  @ApiResponse({ status: 200, description: 'Array of trust resolutions', type: [TrustResolutionResponseDto] })
  async resolveBatch(
    @Body() dto: ResolveBatchRequestDto,
  ): Promise<TrustResolutionResponseDto[]> {
    return await this.dnsService.resolveBatch(dto.queries);
  }

  @Post('register')
  @HttpCode(HttpStatus.CREATED)
  @ApiOperation({
    summary: 'Register a new trust relationship',
    description: 'Creates a new trust record for an agent-principal pair.',
  })
  @ApiResponse({ status: 201, description: 'Trust record created', type: TrustRecordResponseDto })
  async registerTrust(
    @Body() dto: RegisterTrustRequestDto,
  ): Promise<TrustRecordResponseDto> {
    const record = await this.dnsService.registerTrust(
      dto.agentId,
      dto.principalId,
      {
        agentName: dto.agentName,
        agentVersion: dto.agentVersion,
      },
    );
    
    return {
      id: record.id,
      agentId: record.agentId,
      principalId: record.principalId,
      trustScore: record.trustScore,
      trusted: record.trusted,
      revoked: record.revoked,
      registeredAt: typeof record.registeredAt === 'string' ? record.registeredAt : record.registeredAt.toISOString(),
      lastVerifiedAt: typeof record.lastVerifiedAt === 'string' ? record.lastVerifiedAt : record.lastVerifiedAt.toISOString(),
      verificationCount: record.verificationCount,
      failureCount: record.failureCount,
    };
  }

  @Post('revoke')
  @HttpCode(HttpStatus.OK)
  @ApiOperation({
    summary: 'Revoke trust for an agent-principal pair',
    description: 'Immediately revokes trust and invalidates cache.',
  })
  @ApiResponse({ status: 200, description: 'Trust revoked', type: TrustRecordResponseDto })
  @ApiResponse({ status: 404, description: 'Trust record not found' })
  async revokeTrust(
    @Body() dto: RevokeTrustRequestDto,
  ): Promise<TrustRecordResponseDto> {
    const record = await this.dnsService.revokeTrust(
      dto.agentId,
      dto.principalId,
      dto.reason,
    );
    
    if (!record) {
      throw new NotFoundException('Trust record not found');
    }
    
    return {
      id: record.id,
      agentId: record.agentId,
      principalId: record.principalId,
      trustScore: record.trustScore,
      trusted: record.trusted,
      revoked: record.revoked,
      registeredAt: typeof record.registeredAt === 'string' ? record.registeredAt : record.registeredAt.toISOString(),
      lastVerifiedAt: typeof record.lastVerifiedAt === 'string' ? record.lastVerifiedAt : record.lastVerifiedAt.toISOString(),
      verificationCount: record.verificationCount,
      failureCount: record.failureCount,
    };
  }

  @Post('reinstate')
  @HttpCode(HttpStatus.OK)
  @ApiOperation({
    summary: 'Reinstate previously revoked trust',
    description: 'Reinstates trust if revoked. Trust score remains degraded.',
  })
  @ApiResponse({ status: 200, description: 'Trust reinstated', type: TrustRecordResponseDto })
  @ApiResponse({ status: 404, description: 'Trust record not found' })
  async reinstateTrust(
    @Body() dto: ResolveQueryDto,
  ): Promise<TrustRecordResponseDto> {
    const record = await this.dnsService.reinstateTrust(dto.agentId, dto.principalId);
    
    if (!record) {
      throw new NotFoundException('Trust record not found');
    }
    
    return {
      id: record.id,
      agentId: record.agentId,
      principalId: record.principalId,
      trustScore: record.trustScore,
      trusted: record.trusted,
      revoked: record.revoked,
      registeredAt: typeof record.registeredAt === 'string' ? record.registeredAt : record.registeredAt.toISOString(),
      lastVerifiedAt: typeof record.lastVerifiedAt === 'string' ? record.lastVerifiedAt : record.lastVerifiedAt.toISOString(),
      verificationCount: record.verificationCount,
      failureCount: record.failureCount,
    };
  }

  @Get('records/:principalId')
  @ApiOperation({
    summary: 'Get all trust records for a principal',
    description: 'Returns all agents trusted by a principal.',
  })
  @ApiResponse({ status: 200, description: 'Array of trust records', type: [TrustRecordResponseDto] })
  async getRecordsForPrincipal(
    @Param('principalId') principalId: string,
  ): Promise<TrustRecordResponseDto[]> {
    const records = await this.dnsService.getTrustRecordsForPrincipal(principalId);
    
    return records.map(record => ({
      id: record.id,
      agentId: record.agentId,
      principalId: record.principalId,
      trustScore: record.trustScore,
      trusted: record.trusted,
      revoked: record.revoked,
      registeredAt: typeof record.registeredAt === 'string' ? record.registeredAt : record.registeredAt.toISOString(),
      lastVerifiedAt: typeof record.lastVerifiedAt === 'string' ? record.lastVerifiedAt : record.lastVerifiedAt.toISOString(),
      verificationCount: record.verificationCount,
      failureCount: record.failureCount,
    }));
  }

  @Get('record')
  @ApiOperation({
    summary: 'Get a specific trust record',
    description: 'Returns detailed trust record for an agent-principal pair.',
  })
  @ApiQuery({ name: 'agentId', description: 'Agent identifier' })
  @ApiQuery({ name: 'principalId', description: 'Principal identifier' })
  @ApiResponse({ status: 200, description: 'Trust record', type: TrustRecordResponseDto })
  @ApiResponse({ status: 404, description: 'Trust record not found' })
  async getRecord(
    @Query('agentId') agentId: string,
    @Query('principalId') principalId: string,
  ): Promise<TrustRecordResponseDto> {
    const record = await this.dnsService.getTrustRecord(agentId, principalId);
    
    if (!record) {
      throw new NotFoundException('Trust record not found');
    }
    
    return {
      id: record.id,
      agentId: record.agentId,
      principalId: record.principalId,
      trustScore: record.trustScore,
      trusted: record.trusted,
      revoked: record.revoked,
      registeredAt: typeof record.registeredAt === 'string' ? record.registeredAt : record.registeredAt.toISOString(),
      lastVerifiedAt: typeof record.lastVerifiedAt === 'string' ? record.lastVerifiedAt : record.lastVerifiedAt.toISOString(),
      verificationCount: record.verificationCount,
      failureCount: record.failureCount,
    };
  }
}
