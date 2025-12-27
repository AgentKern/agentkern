/**
 * AgentKern Gateway - Synapse Module
 * 
 * The "Memory" - Cross-agent state & intent tracking.
 * Per ARCHITECTURE: Uses CRDTs for eventual consistency.
 */

import { Module } from '@nestjs/common';
import { SynapseController } from '../controllers/synapse.controller';
import { SynapseService } from '../services/synapse.service';

@Module({
  controllers: [SynapseController],
  providers: [SynapseService],
  exports: [SynapseService],
})
export class SynapseModule {}
