/**
 * VeriMantle Gateway - Nexus Module
 * 
 * Protocol translation and agent discovery module.
 * Part of the 6-Pillar Architecture.
 */

import { Module } from '@nestjs/common';
import { NexusController } from '../controllers/nexus.controller';
import { NexusService } from '../services/nexus.service';

@Module({
  controllers: [NexusController],
  providers: [NexusService],
  exports: [NexusService],
})
export class NexusModule {}
