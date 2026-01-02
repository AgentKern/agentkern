/**
 * AgentKernIdentity - DNS Module
 *
 * Module for Intent DNS resolution.
 * AuditLoggerService is provided globally via SecurityModule.
 */

import { Module } from '@nestjs/common';
import { TypeOrmModule } from '@nestjs/typeorm';
import { DnsController } from '../controllers/dns.controller';
import { DnsResolutionService } from '../services/dns-resolution.service';
import { TrustRecordEntity } from '../entities/trust-record.entity';

@Module({
  imports: [TypeOrmModule.forFeature([TrustRecordEntity])],
  controllers: [DnsController],
  providers: [DnsResolutionService],
  exports: [DnsResolutionService],
})
export class DnsModule {}
