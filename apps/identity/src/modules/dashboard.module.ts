/**
 * AgentKernIdentity - Dashboard Module
 * 
 * Enterprise dashboard and policy management.
 */

import { Module } from '@nestjs/common';
import { DashboardController } from '../controllers/dashboard.controller';
import { PolicyService } from '../services/policy.service';
import { AuditLoggerService } from '../services/audit-logger.service';

@Module({
  controllers: [DashboardController],
  providers: [PolicyService, AuditLoggerService],
  exports: [PolicyService],
})
export class DashboardModule {}
