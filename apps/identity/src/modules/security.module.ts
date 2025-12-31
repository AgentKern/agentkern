/**
 * AgentKernIdentity - Security Module
 *
 * Central module for core security services.
 * Provides AI agent security infrastructure including:
 * - Agent sandboxing and budget enforcement
 * - Audit logging
 * - Trust scoring and reputation
 * - CSP violation monitoring
 *
 * Note: Crypto-agility and prompt injection protection are handled
 * by the Rust Gate package for production-grade implementation.
 * AI governance, privacy, and sustainability are in packages/governance.
 */

import { Module, Global } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { TypeOrmModule } from '@nestjs/typeorm';
import { AgentSandboxService } from '../services/agent-sandbox.service';
import { AuditLoggerService } from '../services/audit-logger.service';
import { GateService } from '../services/gate.service';
import { TrustService } from '../services/trust.service';
import { AgentRecordEntity } from '../entities/agent-record.entity';
import { SystemConfigEntity } from '../entities/system-config.entity';
import { AuditEventEntity } from '../entities/audit-event.entity';
import { TrustScoreEntity, TrustEventEntity } from '../entities/trust-event.entity';
import { CspReportController } from '../controllers/csp-report.controller';
import { AgentsController } from '../controllers/agents.controller';

@Global()
@Module({
  imports: [
    ConfigModule,
    TypeOrmModule.forFeature([
      AgentRecordEntity,
      SystemConfigEntity,
      AuditEventEntity,
      TrustScoreEntity,
      TrustEventEntity,
    ]),
  ],
  controllers: [CspReportController, AgentsController],
  providers: [
    // Core security services
    GateService,
    TrustService,
    AgentSandboxService,
    AuditLoggerService,
  ],
  exports: [
    GateService,
    TrustService,
    AgentSandboxService,
    AuditLoggerService,
  ],
})
export class SecurityModule {}

