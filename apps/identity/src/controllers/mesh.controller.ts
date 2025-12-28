/**
 * AgentKern Identity - Mesh Controller
 * 
 * REST API endpoints for Trust Mesh management.
 */

import {
  Controller,
  Get,
  Post,
  Body,
  Param,
  HttpCode,
  HttpStatus,
} from '@nestjs/common';
import {
  ApiTags,
  ApiOperation,
  ApiResponse,
} from '@nestjs/swagger';
import { MeshNodeService } from '../services/mesh-node.service';
import { TrustEvent } from '../domain/mesh.entity';
import {
  ConnectPeerRequestDto,
  MeshNodeResponseDto,
  MeshStatsResponseDto,
  BroadcastTrustUpdateRequestDto,
  BroadcastRevocationRequestDto,
} from '../dto/mesh.dto';

@ApiTags('Trust Mesh')
@Controller('api/v1/mesh')
export class MeshController {
  constructor(private readonly meshService: MeshNodeService) {}

  @Get('node')
  @ApiOperation({
    summary: 'Get local node info',
    description: 'Returns information about this mesh node.',
  })
  @ApiResponse({ status: 200, description: 'Node info', type: MeshNodeResponseDto })
  getNodeInfo(): MeshNodeResponseDto {
    return this.meshService.getNodeInfo();
  }

  @Get('stats')
  @ApiOperation({
    summary: 'Get mesh statistics',
    description: 'Returns mesh network statistics for this node.',
  })
  @ApiResponse({ status: 200, description: 'Mesh stats', type: MeshStatsResponseDto })
  getMeshStats(): MeshStatsResponseDto {
    return this.meshService.getMeshStats();
  }

  @Get('peers')
  @ApiOperation({
    summary: 'Get connected peers',
    description: 'Returns list of currently connected peer nodes.',
  })
  @ApiResponse({ status: 200, description: 'Connected peers', type: [MeshNodeResponseDto] })
  getConnectedPeers(): MeshNodeResponseDto[] {
    return this.meshService.getConnectedPeers();
  }

  @Post('peers/connect')
  @HttpCode(HttpStatus.CREATED)
  @ApiOperation({
    summary: 'Connect to a peer',
    description: 'Establishes connection to a peer mesh node.',
  })
  @ApiResponse({ status: 201, description: 'Connected successfully' })
  @ApiResponse({ status: 400, description: 'Failed to connect' })
  async connectToPeer(
    @Body() dto: ConnectPeerRequestDto,
  ): Promise<{ success: boolean; message: string }> {
    const success = await this.meshService.connectToPeer(dto.endpoint);
    return {
      success,
      message: success ? 'Connected to peer' : 'Failed to connect',
    };
  }

  @Post('peers/:peerId/disconnect')
  @HttpCode(HttpStatus.OK)
  @ApiOperation({
    summary: 'Disconnect from a peer',
    description: 'Terminates connection to a peer mesh node.',
  })
  @ApiResponse({ status: 200, description: 'Disconnected' })
  disconnectFromPeer(
    @Param('peerId') peerId: string,
  ): { success: boolean } {
    this.meshService.disconnectFromPeer(peerId);
    return { success: true };
  }

  @Post('broadcast/trust')
  @HttpCode(HttpStatus.OK)
  @ApiOperation({
    summary: 'Broadcast trust update',
    description: 'Broadcasts a trust update to the mesh network.',
  })
  @ApiResponse({ status: 200, description: 'Broadcast sent' })
  async broadcastTrustUpdate(
    @Body() dto: BroadcastTrustUpdateRequestDto,
  ): Promise<{ success: boolean }> {
    await this.meshService.broadcastTrustUpdate(
      dto.agentId,
      dto.principalId,
      dto.trustScore,
      dto.event as TrustEvent,
      dto.previousScore,
    );
    return { success: true };
  }

  @Post('broadcast/revocation')
  @HttpCode(HttpStatus.OK)
  @ApiOperation({
    summary: 'Broadcast revocation',
    description: 'Broadcasts a critical revocation to the mesh network.',
  })
  @ApiResponse({ status: 200, description: 'Revocation broadcast sent' })
  async broadcastRevocation(
    @Body() dto: BroadcastRevocationRequestDto,
  ): Promise<{ success: boolean }> {
    await this.meshService.broadcastRevocation(
      dto.agentId,
      dto.principalId,
      dto.reason,
      'local-admin', // In production, from authenticated user
    );
    return { success: true };
  }
}
