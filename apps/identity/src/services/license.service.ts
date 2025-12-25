/**
 * AgentProof - License Service
 * 
 * Gates enterprise features behind license validation.
 * 
 * Tiers:
 * - OSS: Core protocol, SDKs (free)
 * - Pro: Hosted DNS, analytics (usage-based)
 * - Enterprise: Dashboard, policies, compliance (license)
 */

import { Injectable, Logger, OnModuleInit } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';
import * as crypto from 'crypto';

export enum LicenseTier {
  OSS = 'oss',
  PRO = 'pro',
  ENTERPRISE = 'enterprise',
}

export interface LicenseInfo {
  tier: LicenseTier;
  valid: boolean;
  expiresAt?: string;
  features: string[];
  organization?: string;
  maxAgents?: number;
  maxPrincipals?: number;
}

// Feature flags for each tier
const TIER_FEATURES: Record<LicenseTier, string[]> = {
  [LicenseTier.OSS]: [
    'proof.create',
    'proof.verify',
    'dns.resolve',
    'dns.register',
    'mesh.node',
    'mesh.stats',
  ],
  [LicenseTier.PRO]: [
    'proof.create',
    'proof.verify',
    'dns.resolve',
    'dns.register',
    'dns.revoke',
    'dns.batch',
    'mesh.node',
    'mesh.stats',
    'mesh.peers',
    'analytics.basic',
  ],
  [LicenseTier.ENTERPRISE]: [
    'proof.create',
    'proof.verify',
    'proof.audit',
    'dns.resolve',
    'dns.register',
    'dns.revoke',
    'dns.batch',
    'mesh.node',
    'mesh.stats',
    'mesh.peers',
    'mesh.broadcast',
    'dashboard.stats',
    'dashboard.trends',
    'dashboard.agents',
    'policies.read',
    'policies.write',
    'policies.delete',
    'compliance.report',
    'compliance.audit',
    'analytics.advanced',
  ],
};

@Injectable()
export class LicenseService implements OnModuleInit {
  private readonly logger = new Logger(LicenseService.name);
  private licenseInfo: LicenseInfo;

  constructor(private readonly configService: ConfigService) {
    // Default to OSS tier
    this.licenseInfo = {
      tier: LicenseTier.OSS,
      valid: true,
      features: TIER_FEATURES[LicenseTier.OSS],
    };
  }

  async onModuleInit(): Promise<void> {
    const licenseKey = this.configService.get<string>('LICENSE_KEY');
    
    if (licenseKey) {
      await this.validateLicense(licenseKey);
    } else {
      this.logger.log('🆓 Running in Open Source mode (no license key)');
      this.logger.log('   Enterprise features are disabled');
      this.logger.log('   Set LICENSE_KEY environment variable to unlock');
    }
  }

  /**
   * Validate a license key
   */
  async validateLicense(licenseKey: string): Promise<LicenseInfo> {
    try {
      // Decode and verify license
      // Format: base64(tier.org.expiry.signature)
      const decoded = Buffer.from(licenseKey, 'base64').toString('utf8');
      const parts = decoded.split('.');
      
      if (parts.length !== 4) {
        throw new Error('Invalid license format');
      }

      const [tier, organization, expiryTimestamp, signature] = parts;
      
      // Verify signature (in production, use proper asymmetric verification)
      const payload = `${tier}.${organization}.${expiryTimestamp}`;
      const expectedSignature = this.generateSignature(payload);
      
      if (signature !== expectedSignature) {
        throw new Error('Invalid license signature');
      }

      // Check expiry
      const expiresAt = new Date(parseInt(expiryTimestamp, 10) * 1000);
      if (expiresAt < new Date()) {
        throw new Error('License expired');
      }

      // Determine tier
      const licenseTier = tier as LicenseTier;
      if (!Object.values(LicenseTier).includes(licenseTier)) {
        throw new Error('Invalid license tier');
      }

      this.licenseInfo = {
        tier: licenseTier,
        valid: true,
        expiresAt: expiresAt.toISOString(),
        features: TIER_FEATURES[licenseTier],
        organization,
      };

      this.logger.log(`🔑 License validated: ${licenseTier.toUpperCase()} tier`);
      this.logger.log(`   Organization: ${organization}`);
      this.logger.log(`   Expires: ${expiresAt.toISOString()}`);

      return this.licenseInfo;
    } catch (error) {
      this.logger.warn(`⚠️ License validation failed: ${error.message}`);
      this.logger.log('   Falling back to Open Source mode');
      
      this.licenseInfo = {
        tier: LicenseTier.OSS,
        valid: true,
        features: TIER_FEATURES[LicenseTier.OSS],
      };

      return this.licenseInfo;
    }
  }

  /**
   * Get current license info
   */
  getLicenseInfo(): LicenseInfo {
    return this.licenseInfo;
  }

  /**
   * Check if a feature is available
   */
  hasFeature(feature: string): boolean {
    return this.licenseInfo.features.includes(feature);
  }

  /**
   * Check if current tier is at least the specified tier
   */
  hasTier(tier: LicenseTier): boolean {
    const tierOrder = [LicenseTier.OSS, LicenseTier.PRO, LicenseTier.ENTERPRISE];
    const currentIndex = tierOrder.indexOf(this.licenseInfo.tier);
    const requiredIndex = tierOrder.indexOf(tier);
    return currentIndex >= requiredIndex;
  }

  /**
   * Generate a license key (for admin use)
   */
  generateLicenseKey(
    tier: LicenseTier,
    organization: string,
    expiryDate: Date,
  ): string {
    const expiryTimestamp = Math.floor(expiryDate.getTime() / 1000);
    const payload = `${tier}.${organization}.${expiryTimestamp}`;
    const signature = this.generateSignature(payload);
    const licenseData = `${payload}.${signature}`;
    return Buffer.from(licenseData).toString('base64');
  }

  private generateSignature(payload: string): string {
    // In production, use asymmetric keys (e.g., Ed25519)
    const secret = this.configService.get('LICENSE_SECRET', 'agentproof-license-secret');
    return crypto
      .createHmac('sha256', secret)
      .update(payload)
      .digest('hex')
      .substring(0, 16);
  }
}
