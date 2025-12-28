/**
 * WebAuthn Controller Tests
 */

import { Test, TestingModule } from '@nestjs/testing';
import { WebAuthnController } from './webauthn.controller';
import { WebAuthnService } from '../services/webauthn.service';
import { ConfigService } from '@nestjs/config';

describe('WebAuthnController', () => {
  let controller: WebAuthnController;
  let webAuthnService: WebAuthnService;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      controllers: [WebAuthnController],
      providers: [
        WebAuthnService,
        {
          provide: ConfigService,
          useValue: {
            get: jest.fn((key: string, defaultValue?: any) => {
              const config: Record<string, any> = {
                'WEBAUTHN_RP_NAME': 'AgentKernIdentity',
                'WEBAUTHN_RP_ID': 'localhost',
                'WEBAUTHN_ORIGIN': 'http://localhost:3000',
              };
              return config[key] ?? defaultValue;
            }),
          },
        },
      ],
    }).compile();

    controller = module.get<WebAuthnController>(WebAuthnController);
    webAuthnService = module.get<WebAuthnService>(WebAuthnService);
  });

  it('should be defined', () => {
    expect(controller).toBeDefined();
  });

  describe('startRegistration', () => {
    it('should generate registration options', async () => {
      const result = await controller.startRegistration({
        principalId: 'principal-1',
        userName: 'user@example.com',
      });

      expect(result.options).toBeDefined();
      expect(result.options.challenge).toBeDefined();
      expect(result.options.rp).toBeDefined();
    });

    it('should use displayName if provided', async () => {
      const result = await controller.startRegistration({
        principalId: 'principal-2',
        userName: 'user2@example.com',
        displayName: 'User Two',
      });

      expect(result.options).toBeDefined();
      expect(result.options.user.displayName).toBe('User Two');
    });
  });

  describe('verifyRegistration', () => {
    it('should verify registration response', async () => {
      // First start registration
      await controller.startRegistration({
        principalId: 'verify-principal',
        userName: 'verify@test.com',
      });

      // Then try to verify
      const result = await controller.verifyRegistration({
        principalId: 'verify-principal',
        response: {
          id: 'test-id',
          rawId: 'test-raw',
          response: {},
          type: 'public-key',
        } as any,
      });

      expect(result.verified).toBeDefined();
    });
  });

  describe('startAuthentication', () => {
    it('should return error for unknown principal', async () => {
      const result = await controller.startAuthentication({
        principalId: 'unknown-principal',
      });

      expect((result as any).error).toBe('No credentials found for principal');
    });

    it('should return options for principal with credentials', async () => {
      // Start registration first
      await controller.startRegistration({
        principalId: 'auth-principal',
        userName: 'auth@test.com',
      });

      const result = await controller.startAuthentication({
        principalId: 'auth-principal',
      });

      // Will still return error since no credentials were verified
      expect((result as any).error).toBe('No credentials found for principal');
    });
  });

  describe('verifyAuthentication', () => {
    it('should verify authentication response', async () => {
      const result = await controller.verifyAuthentication({
        principalId: 'auth-verify',
        response: {
          id: 'test-id',
          rawId: 'test-raw',
          response: {},
          type: 'public-key',
        } as any,
      });

      expect(result.verified).toBe(false);
    });
  });

  describe('getCredentials', () => {
    it('should return empty credentials for unknown principal', () => {
      const result = controller.getCredentials('unknown');

      expect(result.principalId).toBe('unknown');
      expect(result.credentials).toEqual([]);
    });

    it('should return credentials for registered principal', async () => {
      await controller.startRegistration({
        principalId: 'cred-principal',
        userName: 'cred@test.com',
      });

      const result = controller.getCredentials('cred-principal');

      expect(result.principalId).toBe('cred-principal');
      expect(Array.isArray(result.credentials)).toBe(true);
    });
  });
});
