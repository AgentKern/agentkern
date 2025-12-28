/**
 * AgentKern Identity - Mesh Module
 * 
 * Module for Trust Mesh P2P network.
 */

import { Module } from '@nestjs/common';
import { MeshController } from '../controllers/mesh.controller';
import { MeshNodeService } from '../services/mesh-node.service';
import { MeshGateway } from '../gateways/mesh.gateway';
import { AuditLoggerService } from '../services/audit-logger.service';

@Module({
  controllers: [MeshController],
  providers: [MeshNodeService, MeshGateway, AuditLoggerService],
  exports: [MeshNodeService, MeshGateway],
})
export class MeshModule {}
