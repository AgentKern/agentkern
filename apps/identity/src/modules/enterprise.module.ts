/**
 * AgentKernIdentity - Enterprise Module
 *
 * Provides enterprise license integration for the Identity app.
 * Connects with AgentKern's ee/ licensing system.
 */

import { Module, Global } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { EnterpriseLicenseGuard } from '../guards/enterprise-license.guard';

@Global()
@Module({
  imports: [ConfigModule],
  providers: [EnterpriseLicenseGuard],
  exports: [EnterpriseLicenseGuard],
})
export class EnterpriseModule {}
