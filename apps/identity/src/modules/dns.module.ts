/**
 * AgentKernIdentity - DNS Module
 * 
 * Module for Intent DNS resolution.
 */

import { Module } from '@nestjs/common';
import { DnsController } from '../controllers/dns.controller';
import { DnsResolutionService } from '../services/dns-resolution.service';
import { AuditLoggerService } from '../services/audit-logger.service';

@Module({
  controllers: [DnsController],
  providers: [DnsResolutionService, AuditLoggerService],
  exports: [DnsResolutionService],
})
export class DnsModule {}
