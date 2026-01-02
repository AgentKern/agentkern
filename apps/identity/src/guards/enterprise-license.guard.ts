/**
 * AgentKernIdentity - Enterprise License Guard
 *
 * Thin guard that integrates with AgentKern's ee/ licensing system.
 * Provides feature gating for enterprise-only functionality.
 *
 * Architecture:
 * - Checks LICENSE_KEY environment variable for basic enterprise detection
 * - Optionally calls ee/ license service for detailed entitlement checks
 * - Gracefully degrades when ee/ service is unavailable
 */

import {
  Injectable,
  CanActivate,
  ExecutionContext,
  SetMetadata,
  Logger,
} from '@nestjs/common';
import { Reflector } from '@nestjs/core';
import { ConfigService } from '@nestjs/config';

// Metadata keys for decorators
export const ENTERPRISE_REQUIRED = 'enterprise_required';
export const REQUIRED_FEATURES = 'required_features';

/**
 * Mark a route as enterprise-only
 */
export const EnterpriseOnly = () => SetMetadata(ENTERPRISE_REQUIRED, true);

/**
 * Require specific enterprise features/entitlements
 */
export const RequireFeatures = (...features: string[]) =>
  SetMetadata(REQUIRED_FEATURES, features);

/**
 * License tiers (aligned with ee/ licensing)
 */
export enum LicenseTier {
  COMMUNITY = 'community',
  PROFESSIONAL = 'professional',
  ENTERPRISE = 'enterprise',
}

/**
 * License info structure
 */
export interface LicenseInfo {
  valid: boolean;
  tier: LicenseTier;
  features: string[];
  expiresAt?: Date;
  orgId?: string;
}

@Injectable()
export class EnterpriseLicenseGuard implements CanActivate {
  private readonly logger = new Logger(EnterpriseLicenseGuard.name);
  private cachedLicense: LicenseInfo | null = null;
  private cacheExpiry: Date | null = null;
  private readonly CACHE_TTL_MS = 60000; // 1 minute cache

  constructor(
    private readonly reflector: Reflector,
    private readonly configService: ConfigService,
  ) {}

  async canActivate(context: ExecutionContext): Promise<boolean> {
    // Check if route requires enterprise
    const requiresEnterprise = this.reflector.getAllAndOverride<boolean>(
      ENTERPRISE_REQUIRED,
      [context.getHandler(), context.getClass()],
    );

    const requiredFeatures = this.reflector.getAllAndOverride<string[]>(
      REQUIRED_FEATURES,
      [context.getHandler(), context.getClass()],
    );

    // If no enterprise requirements, allow access
    if (!requiresEnterprise && !requiredFeatures?.length) {
      return true;
    }

    // Get license info
    const license = await this.getLicenseInfo();

    // Check if enterprise license is valid
    if (requiresEnterprise && !this.isEnterpriseOrHigher(license)) {
      this.logger.warn('Enterprise license required but not present');
      return false;
    }

    // Check required features
    if (requiredFeatures?.length) {
      const hasAllFeatures = requiredFeatures.every((f) =>
        license.features.includes(f),
      );
      if (!hasAllFeatures) {
        this.logger.warn(
          `Missing required features: ${requiredFeatures.filter((f) => !license.features.includes(f)).join(', ')}`,
        );
        return false;
      }
    }

    return true;
  }

  /**
   * Get license info (cached)
   */
  private async getLicenseInfo(): Promise<LicenseInfo> {
    // Check cache
    if (
      this.cachedLicense &&
      this.cacheExpiry &&
      new Date() < this.cacheExpiry
    ) {
      return this.cachedLicense;
    }

    // Try to get license from ee/ service first
    const eeServiceUrl = this.configService.get<string>(
      'EE_LICENSE_SERVICE_URL',
    );
    if (eeServiceUrl) {
      try {
        const license = await this.fetchFromEeService(eeServiceUrl);
        this.updateCache(license);
        return license;
      } catch (error) {
        this.logger.warn(`Failed to fetch from ee/ license service: ${error}`);
        // Fall through to local check
      }
    }

    // Fallback: check local LICENSE_KEY
    const license = this.checkLocalLicense();
    this.updateCache(license);
    return license;
  }

  /**
   * Fetch license from ee/ license service
   */
  private async fetchFromEeService(serviceUrl: string): Promise<LicenseInfo> {
    const licenseKey = this.configService.get<string>('LICENSE_KEY');

    const response = await fetch(`${serviceUrl}/api/v1/license/validate`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-License-Key': licenseKey || '',
      },
      body: JSON.stringify({ licenseKey }),
    });

    if (!response.ok) {
      throw new Error(`License service returned ${response.status}`);
    }

    return response.json() as Promise<LicenseInfo>;
  }

  /**
   * Check license from local environment (fallback)
   * Checks AGENTKERN_LICENSE_KEY (per ee/ docs) then LICENSE_KEY for compatibility
   */
  private checkLocalLicense(): LicenseInfo {
    const licenseKey =
      this.configService.get<string>('AGENTKERN_LICENSE_KEY') ||
      this.configService.get<string>('LICENSE_KEY');

    if (!licenseKey) {
      return {
        valid: true,
        tier: LicenseTier.COMMUNITY,
        features: ['basic'],
      };
    }

    // Parse license key format: TIER-ORG-EXPIRY-SIGNATURE
    // Example: ENT-org123-20251231-abc123
    const parts = licenseKey.split('-');

    if (parts.length < 2) {
      this.logger.warn('Invalid license key format');
      return {
        valid: false,
        tier: LicenseTier.COMMUNITY,
        features: [],
      };
    }

    const tierCode = parts[0].toUpperCase();
    const tier = this.parseTier(tierCode);
    const features = this.getFeaturesForTier(tier);

    // Check expiry if present
    let expiresAt: Date | undefined;
    if (parts.length >= 3) {
      const expiryStr = parts[2];
      if (/^\d{8}$/.test(expiryStr)) {
        expiresAt = new Date(
          `${expiryStr.slice(0, 4)}-${expiryStr.slice(4, 6)}-${expiryStr.slice(6, 8)}`,
        );
        if (expiresAt < new Date()) {
          this.logger.warn('License has expired');
          return {
            valid: false,
            tier,
            features: [],
            expiresAt,
          };
        }
      }
    }

    return {
      valid: true,
      tier,
      features,
      expiresAt,
      orgId: parts.length >= 2 ? parts[1] : undefined,
    };
  }

  private parseTier(code: string): LicenseTier {
    switch (code) {
      case 'ENT':
      case 'ENTERPRISE':
        return LicenseTier.ENTERPRISE;
      case 'PRO':
      case 'PROFESSIONAL':
        return LicenseTier.PROFESSIONAL;
      default:
        return LicenseTier.COMMUNITY;
    }
  }

  private getFeaturesForTier(tier: LicenseTier): string[] {
    const features: string[] = ['basic'];

    if (tier === LicenseTier.PROFESSIONAL || tier === LicenseTier.ENTERPRISE) {
      features.push(
        'dashboard.stats',
        'dashboard.trends',
        'dashboard.compliance',
        'audit.export',
        'trust.advanced',
      );
    }

    if (tier === LicenseTier.ENTERPRISE) {
      features.push(
        'sso',
        'multitenancy',
        'custom-policies',
        'priority-support',
        'sla-guarantee',
        'dedicated-support',
      );
    }

    return features;
  }

  private isEnterpriseOrHigher(license: LicenseInfo): boolean {
    return (
      license.valid &&
      (license.tier === LicenseTier.ENTERPRISE ||
        license.tier === LicenseTier.PROFESSIONAL)
    );
  }

  private updateCache(license: LicenseInfo): void {
    this.cachedLicense = license;
    this.cacheExpiry = new Date(Date.now() + this.CACHE_TTL_MS);
  }

  /**
   * Get current license info (for status endpoints)
   */
  async getCurrentLicense(): Promise<LicenseInfo> {
    return this.getLicenseInfo();
  }
}
