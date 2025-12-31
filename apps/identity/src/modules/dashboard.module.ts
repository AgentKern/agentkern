/**
 * AgentKernIdentity - Dashboard Module
 *
 * Enterprise dashboard for administration.
 * Note: Policy management is handled by Rust Gate package.
 * AuditLoggerService is provided globally via SecurityModule.
 */

import { Module } from '@nestjs/common';
import { DashboardController } from '../controllers/dashboard.controller';

@Module({
  controllers: [DashboardController],
  // AuditLoggerService is available via @Global() SecurityModule
})
export class DashboardModule {}

