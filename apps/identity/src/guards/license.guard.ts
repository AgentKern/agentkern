/**
 * AgentKernIdentity - License Guard
 * 
 * Guard that checks if the current license allows access to a feature.
 */

import {
  Injectable,
  CanActivate,
  ExecutionContext,
  ForbiddenException,
  SetMetadata,
} from '@nestjs/common';
import { Reflector } from '@nestjs/core';
import { LicenseService, LicenseTier } from '../services/license.service';

// Decorator to mark routes as requiring a specific feature
export const RequireFeature = (feature: string) => SetMetadata('requiredFeature', feature);

// Decorator to mark routes as requiring a specific tier
export const RequireTier = (tier: LicenseTier) => SetMetadata('requiredTier', tier);

// Decorator for enterprise-only routes
export const EnterpriseOnly = () => RequireTier(LicenseTier.ENTERPRISE);

// Decorator for pro+ routes
export const ProOnly = () => RequireTier(LicenseTier.PRO);

@Injectable()
export class LicenseGuard implements CanActivate {
  constructor(
    private readonly reflector: Reflector,
    private readonly licenseService: LicenseService,
  ) {}

  canActivate(context: ExecutionContext): boolean {
    // Check for required feature
    const requiredFeature = this.reflector.get<string>(
      'requiredFeature',
      context.getHandler(),
    );

    if (requiredFeature && !this.licenseService.hasFeature(requiredFeature)) {
      const license = this.licenseService.getLicenseInfo();
      throw new ForbiddenException({
        error: 'Feature not available',
        message: `The '${requiredFeature}' feature requires a higher license tier`,
        currentTier: license.tier,
        upgrade: 'https://agentkern-identity.io/pricing',
      });
    }

    // Check for required tier
    const requiredTier = this.reflector.get<LicenseTier>(
      'requiredTier',
      context.getHandler(),
    );

    if (requiredTier && !this.licenseService.hasTier(requiredTier)) {
      const license = this.licenseService.getLicenseInfo();
      throw new ForbiddenException({
        error: 'License tier insufficient',
        message: `This endpoint requires ${requiredTier.toUpperCase()} tier or higher`,
        currentTier: license.tier,
        requiredTier: requiredTier,
        upgrade: 'https://agentkern-identity.io/pricing',
      });
    }

    return true;
  }
}
