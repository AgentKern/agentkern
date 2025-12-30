/**
 * AgentKernIdentity - Security Module
 *
 * Central module for all security services and guards.
 * Provides AI agent security infrastructure including:
 * - Prompt injection protection
 * - Crypto-agility for quantum-safe migration
 * - Agent sandboxing and budget enforcement
 * - CSP violation monitoring
 */

import { Module, Global } from '@nestjs/common';
import { APP_GUARD } from '@nestjs/core';
import { ConfigModule } from '@nestjs/config';
import { TypeOrmModule } from '@nestjs/typeorm';
import { PromptInjectionGuard } from '../guards/prompt-injection.guard';
import { CryptoAgilityService } from '../services/crypto-agility.service';
import { AgentSandboxService } from '../services/agent-sandbox.service';
import { AuditLoggerService } from '../services/audit-logger.service';
import { AIGovernanceService } from '../services/ai-governance.service';
import { PrivacyEngineeringService } from '../services/privacy-engineering.service';
import { SustainabilityService } from '../services/sustainability.service';
import { AgentRecordEntity } from '../entities/agent-record.entity';
import { CspReportController } from '../controllers/csp-report.controller';

@Global()
@Module({
  imports: [ConfigModule, TypeOrmModule.forFeature([AgentRecordEntity])],
  controllers: [CspReportController],
  providers: [
    // Core security services
    CryptoAgilityService,
    AgentSandboxService,
    AuditLoggerService,
    AIGovernanceService,
    PrivacyEngineeringService,
    SustainabilityService,

    // Global prompt injection protection
    PromptInjectionGuard,
  ],
  exports: [
    CryptoAgilityService,
    AgentSandboxService,
    AuditLoggerService,
    AIGovernanceService,
    PrivacyEngineeringService,
    SustainabilityService,
    PromptInjectionGuard,
  ],
})
export class SecurityModule {}

