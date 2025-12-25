/**
 * LicenseGuard Full Coverage Tests
 */

import { Test, TestingModule } from '@nestjs/testing';
import { ExecutionContext, ForbiddenException } from '@nestjs/common';
import { Reflector } from '@nestjs/core';
import { LicenseGuard, ENTERPRISE_ONLY_KEY, REQUIRED_TIER_KEY, REQUIRED_FEATURE_KEY } from './license.guard';
import { LicenseService, LicenseTier } from '../services/license.service';

describe('LicenseGuard Full Coverage', () => {
  let guard: LicenseGuard;
  let licenseService: LicenseService;
  let reflector: Reflector;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [
        LicenseGuard,
        {
          provide: LicenseService,
          useValue: {
            hasFeature: jest.fn().mockReturnValue(true),
            hasTier: jest.fn().mockReturnValue(true),
            getLicenseInfo: jest.fn().mockReturnValue({
              tier: LicenseTier.OSS,
              valid: true,
              features: [],
            }),
          },
        },
        Reflector,
      ],
    }).compile();

    guard = module.get<LicenseGuard>(LicenseGuard);
    licenseService = module.get<LicenseService>(LicenseService);
    reflector = module.get<Reflector>(Reflector);
  });

  function createMockExecutionContext(overrides = {}): ExecutionContext {
    return {
      getHandler: jest.fn().mockReturnValue(() => {}),
      getClass: jest.fn().mockReturnValue(class {}),
      switchToHttp: jest.fn().mockReturnValue({
        getRequest: jest.fn().mockReturnValue({}),
        getResponse: jest.fn().mockReturnValue({
          status: jest.fn().mockReturnThis(),
          json: jest.fn(),
        }),
      }),
      getArgs: jest.fn(),
      getArgByIndex: jest.fn(),
      switchToRpc: jest.fn(),
      switchToWs: jest.fn(),
      getType: jest.fn(),
      ...overrides,
    } as unknown as ExecutionContext;
  }

  it('should be defined', () => {
    expect(guard).toBeDefined();
  });

  describe('canActivate', () => {
    it('should return true when no enterprise decorator', () => {
      const mockContext = createMockExecutionContext();
      jest.spyOn(reflector, 'getAllAndOverride').mockReturnValue(undefined);
      
      const result = guard.canActivate(mockContext);
      expect(result).toBe(true);
    });

    it('should return true when enterprise check passes', () => {
      const mockContext = createMockExecutionContext();
      jest.spyOn(reflector, 'getAllAndOverride')
        .mockImplementation((key: string) => {
          if (key === ENTERPRISE_ONLY_KEY) return true;
          return undefined;
        });
      jest.spyOn(licenseService, 'hasTier').mockReturnValue(true);
      
      const result = guard.canActivate(mockContext);
      expect(result).toBe(true);
    });

    it('should check required tier', () => {
      const mockContext = createMockExecutionContext();
      jest.spyOn(reflector, 'getAllAndOverride')
        .mockImplementation((key: string) => {
          if (key === REQUIRED_TIER_KEY) return LicenseTier.ENTERPRISE;
          return undefined;
        });
      jest.spyOn(licenseService, 'hasTier').mockReturnValue(true);
      
      const result = guard.canActivate(mockContext);
      expect(result).toBe(true);
    });

    it('should check required feature', () => {
      const mockContext = createMockExecutionContext();
      jest.spyOn(reflector, 'getAllAndOverride')
        .mockImplementation((key: string) => {
          if (key === REQUIRED_FEATURE_KEY) return 'dashboard.stats';
          return undefined;
        });
      jest.spyOn(licenseService, 'hasFeature').mockReturnValue(true);
      
      const result = guard.canActivate(mockContext);
      expect(result).toBe(true);
    });
  });
});
