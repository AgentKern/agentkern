/**
 * AgentKern Gateway - Arbiter Module
 * 
 * The "Traffic Control" - Conflict resolution & coordination.
 * Per ARCHITECTURE: Implements Atomic Business Locks via Raft.
 */

import { Module } from '@nestjs/common';
import { ArbiterController } from '../controllers/arbiter.controller';
import { ArbiterService } from '../services/arbiter.service';

@Module({
  controllers: [ArbiterController],
  providers: [ArbiterService],
  exports: [ArbiterService],
})
export class ArbiterModule {}
