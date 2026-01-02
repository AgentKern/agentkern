/**
 * AgentKernIdentity - Nexus Module
 *
 * Protocol translation and agent discovery.
 * Now with TypeORM persistence for agent registry.
 */

import { Module } from '@nestjs/common';
import { TypeOrmModule } from '@nestjs/typeorm';
import {
  NexusController,
  WellKnownController,
} from '../controllers/nexus.controller';
import { NexusService } from '../services/nexus.service';
import { NexusAgentEntity } from '../entities/nexus-agent.entity';
import { NexusAgentRepository } from '../repositories/nexus-agent.repository';

@Module({
  imports: [TypeOrmModule.forFeature([NexusAgentEntity])],
  controllers: [NexusController, WellKnownController],
  providers: [NexusService, NexusAgentRepository],
  exports: [NexusService, NexusAgentRepository],
})
export class NexusModule {}
