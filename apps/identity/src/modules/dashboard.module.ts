/**
 * AgentKernIdentity - Dashboard Module
 *
 * Enterprise dashboard for administration.
 * Note: Policy management is handled by Rust Gate package.
 */

import { Module } from '@nestjs/common';
import { DashboardController } from '../controllers/dashboard.controller';
import { AuditLoggerService } from '../services/audit-logger.service';

@Module({
  controllers: [DashboardController],
  providers: [AuditLoggerService],
})
export class DashboardModule {}
