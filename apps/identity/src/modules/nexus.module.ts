/**
 * AgentKernIdentity - Nexus Module
 * 
 * Protocol translation and agent discovery.
 * Merged from apps/gateway for consolidated architecture.
 */

import { Module } from '@nestjs/common';
import { NexusController, WellKnownController } from '../controllers/nexus.controller';
import { NexusService } from '../services/nexus.service';

@Module({
  controllers: [NexusController, WellKnownController],
  providers: [NexusService],
  exports: [NexusService],
})
export class NexusModule {}
