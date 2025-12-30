/**
 * AgentKernIdentity - WebAuthn Module
 *
 * Module for Passkey authentication with persistent storage.
 * Uses TypeORM repositories for credential and challenge persistence.
 */

import { Module } from '@nestjs/common';
import { TypeOrmModule } from '@nestjs/typeorm';
import { WebAuthnController } from '../controllers/webauthn.controller';
import { WebAuthnService } from '../services/webauthn.service';
import { WebAuthnCredentialEntity, WebAuthnChallengeEntity } from '../entities/webauthn-credential.entity';

@Module({
  imports: [
    TypeOrmModule.forFeature([WebAuthnCredentialEntity, WebAuthnChallengeEntity]),
  ],
  controllers: [WebAuthnController],
  providers: [WebAuthnService],
  exports: [WebAuthnService],
})
export class WebAuthnModule {}

