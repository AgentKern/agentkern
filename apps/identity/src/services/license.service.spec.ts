/**
 * License Service Tests
 */

import { Test, TestingModule } from '@nestjs/testing';
import { ConfigService } from '@nestjs/config';
import { LicenseService, LicenseTier } from './license.service';

describe('LicenseService', () => {
  let service: LicenseService;
  let configService: ConfigService;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [
        LicenseService,
        {
          provide: ConfigService,
          useValue: {
            get: jest.fn().mockImplementation((key: string, defaultValue?: any) => defaultValue),
          },
        },
      ],
    }).compile();

    service = module.get<LicenseService>(LicenseService);
    configService = module.get<ConfigService>(ConfigService);
    await service.onModuleInit();
  });

  describe('getLicenseInfo', () => {
    it('should return OSS tier by default', () => {
      const info = service.getLicenseInfo();
      expect(info.tier).toBe(LicenseTier.OSS);
      expect(info.valid).toBe(true);
    });

    it('should have basic features in OSS tier', () => {
      const info = service.getLicenseInfo();
      expect(info.features).toContain('proof.create');
      expect(info.features).toContain('proof.verify');
      expect(info.features).toContain('dns.resolve');
    });
  });

  describe('hasFeature', () => {
    it('should return true for OSS features', () => {
      expect(service.hasFeature('proof.create')).toBe(true);
      expect(service.hasFeature('proof.verify')).toBe(true);
    });

    it('should return false for Enterprise features in OSS tier', () => {
      expect(service.hasFeature('dashboard.stats')).toBe(false);
      expect(service.hasFeature('policies.write')).toBe(false);
    });
  });

  describe('hasTier', () => {
    it('should return true for OSS tier', () => {
      expect(service.hasTier(LicenseTier.OSS)).toBe(true);
    });

    it('should return false for higher tiers', () => {
      expect(service.hasTier(LicenseTier.PRO)).toBe(false);
      expect(service.hasTier(LicenseTier.ENTERPRISE)).toBe(false);
    });
  });

  describe('generateLicenseKey', () => {
    it('should generate a valid license key', () => {
      const futureDate = new Date();
      futureDate.setFullYear(futureDate.getFullYear() + 1);
      
      const key = service.generateLicenseKey(
        LicenseTier.ENTERPRISE,
        'TestOrg',
        futureDate,
      );
      
      expect(key).toBeDefined();
      expect(typeof key).toBe('string');
      expect(key.length).toBeGreaterThan(10);
    });
  });

  describe('validateLicense', () => {
    it('should reject invalid license format', async () => {
      const result = await service.validateLicense('invalid-key');
      expect(result.tier).toBe(LicenseTier.OSS);
    });

    it('should validate and set correct tier for valid key', async () => {
      const futureDate = new Date();
      futureDate.setFullYear(futureDate.getFullYear() + 1);
      
      const key = service.generateLicenseKey(
        LicenseTier.ENTERPRISE,
        'TestOrg',
        futureDate,
      );
      
      const result = await service.validateLicense(key);
      expect(result.tier).toBe(LicenseTier.ENTERPRISE);
      expect(result.valid).toBe(true);
      expect(result.organization).toBe('TestOrg');
    });
  });
});
