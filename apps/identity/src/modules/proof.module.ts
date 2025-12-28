/**
 * AgentKern Identity - Proof Module
 * 
 * Main module for Liability Proof operations.
 * Wires together all services, controllers, and dependencies.
 */

import { Module } from '@nestjs/common';
import { ProofController } from '../controllers/proof.controller';
import { ProofVerificationService } from '../services/proof-verification.service';
import { ProofSigningService } from '../services/proof-signing.service';
import { AuditLoggerService } from '../services/audit-logger.service';

@Module({
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
