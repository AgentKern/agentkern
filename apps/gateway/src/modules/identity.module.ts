/**
 * AgentKern Gateway - Identity Module
 * 
 * The "Passport" - Agent authentication & liability tracking.
 * Per MANIFESTO: Every agent request is signed (AgentKern-Identity).
 * 
 * This module proxies requests to the AgentKern Identity service.
 * In production, Identity runs as a separate service at http://identity:3000
 */

import { Module } from '@nestjs/common';
import { HttpModule } from '@nestjs/axios';
import { IdentityController } from '../controllers/identity.controller';

@Module({
  imports: [
    HttpModule.register({
      baseURL: process.env.IDENTITY_SERVICE_URL || 'http://localhost:3000',
      timeout: 5000,
    }),
  ],
  controllers: [IdentityController],
  providers: [],
  exports: [],
})
export class IdentityModule {}
