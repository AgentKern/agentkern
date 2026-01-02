/**
 * AgentKernIdentity - Proof Module
 *
 * Main module for Liability Proof operations.
 * Wires together services, controllers, and TypeORM repositories.
 * AuditLoggerService is provided globally via SecurityModule.
 */

import { Module } from '@nestjs/common';
import { TypeOrmModule } from '@nestjs/typeorm';
import { ProofController } from '../controllers/proof.controller';
import { ProofVerificationService } from '../services/proof-verification.service';
import { ProofSigningService } from '../services/proof-signing.service';
import { VerificationKeyEntity } from '../entities/verification-key.entity';

@Module({
  imports: [TypeOrmModule.forFeature([VerificationKeyEntity])],
  controllers: [ProofController],
  providers: [ProofVerificationService, ProofSigningService],
  exports: [ProofVerificationService, ProofSigningService],
})
export class ProofModule {}
