/**
 * AgentProof - License Module
 * 
 * Provides license validation and feature gating.
 */

import { Module, Global } from '@nestjs/common';
import { LicenseService } from '../services/license.service';
import { LicenseGuard } from '../guards/license.guard';

@Global()
@Module({
  providers: [LicenseService, LicenseGuard],
  exports: [LicenseService, LicenseGuard],
})
export class LicenseModule {}
