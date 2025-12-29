/**
 * AgentKernIdentity - Mesh Module
 * 
 * Module for Trust Mesh P2P network.
 */

import { Module } from '@nestjs/common';
import { TypeOrmModule } from '@nestjs/typeorm';
import { MeshController } from '../controllers/mesh.controller';
import { MeshNodeService } from '../services/mesh-node.service';
import { MeshGateway } from '../gateways/mesh.gateway';
import { AuditLoggerService } from '../services/audit-logger.service';
import { MeshPeerEntity, NodeIdentityEntity } from '../entities/mesh-node.entity';

@Module({
  imports: [TypeOrmModule.forFeature([MeshPeerEntity, NodeIdentityEntity])],
  controllers: [MeshController],
  providers: [MeshNodeService, MeshGateway, AuditLoggerService],
  exports: [MeshNodeService, MeshGateway],
})
export class MeshModule {}

