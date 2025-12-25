/**
 * AgentProof - WebAuthn Module
 * 
 * Module for Passkey authentication.
 */

import { Module } from '@nestjs/common';
import { WebAuthnController } from '../controllers/webauthn.controller';
import { WebAuthnService } from '../services/webauthn.service';

@Module({
  controllers: [WebAuthnController],
  providers: [WebAuthnService],
  exports: [WebAuthnService],
})
export class WebAuthnModule {}
