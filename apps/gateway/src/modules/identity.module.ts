/**
 * AgentKern Gateway - Identity Module
 * 
 * The "Passport" - Agent authentication & liability tracking.
 * Per MANIFESTO: Every agent request is signed (AgentKern-Identity).
 */

import { Module } from '@nestjs/common';
import { IdentityController } from '../controllers/identity.controller';
import { IdentityService } from '../services/identity.service';

@Module({
  controllers: [IdentityController],
  providers: [IdentityService],
  exports: [IdentityService],
})
export class IdentityModule {}
