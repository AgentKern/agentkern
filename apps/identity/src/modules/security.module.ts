/**
 * AgentKernIdentity - Security Module
 *
 * Central module for core security services.
 * Provides AI agent security infrastructure including:
 * - Agent sandboxing and budget enforcement
 * - Audit logging
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
import { AgentRecordEntity } from '../entities/agent-record.entity';
import { CspReportController } from '../controllers/csp-report.controller';

@Global()
@Module({
  imports: [ConfigModule, TypeOrmModule.forFeature([AgentRecordEntity])],
  controllers: [CspReportController],
  providers: [
    // Core security services
    AgentSandboxService,
    AuditLoggerService,
  ],
  exports: [
    AgentSandboxService,
    AuditLoggerService,
  ],
})
export class SecurityModule {}
