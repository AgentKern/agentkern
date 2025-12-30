/**
 * AgentKernIdentity - Proof Module
 *
 * Main module for Liability Proof operations.
 * Wires together services, controllers, and TypeORM repositories.
 */

import { Module } from '@nestjs/common';
import { TypeOrmModule } from '@nestjs/typeorm';
import { ProofController } from '../controllers/proof.controller';
import { ProofVerificationService } from '../services/proof-verification.service';
import { ProofSigningService } from '../services/proof-signing.service';
import { AuditLoggerService } from '../services/audit-logger.service';
import { VerificationKeyEntity } from '../entities/verification-key.entity';
import { AuditEventEntity } from '../entities/audit-event.entity';

@Module({
  imports: [
    TypeOrmModule.forFeature([VerificationKeyEntity, AuditEventEntity]),
  ],
  controllers: [ProofController],
  providers: [
    ProofVerificationService,
    ProofSigningService,
    AuditLoggerService,
  ],
  exports: [
    ProofVerificationService,
    ProofSigningService,
    AuditLoggerService,
  ],
})
export class ProofModule {}
