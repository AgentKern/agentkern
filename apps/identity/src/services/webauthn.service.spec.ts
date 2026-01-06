import { Test, TestingModule } from '@nestjs/testing';
import { getRepositoryToken } from '@nestjs/typeorm';
import { ConfigService } from '@nestjs/config';
import { Repository } from 'typeorm';
import { WebAuthnService } from './webauthn.service';
import {
  WebAuthnCredentialEntity,
  WebAuthnChallengeEntity,
} from '../entities/webauthn-credential.entity';

// Mock @simplewebauthn/server
jest.mock('@simplewebauthn/server', () => ({
  generateRegistrationOptions: jest.fn().mockResolvedValue({
    challenge: 'test-challenge',
    rp: { id: 'localhost', name: 'Test' },
    user: { id: 'user-id', name: 'test', displayName: 'Test User' },
  }),
  verifyRegistrationResponse: jest.fn(),
  generateAuthenticationOptions: jest.fn().mockResolvedValue({
    challenge: 'test-auth-challenge',
    allowCredentials: [],
  }),
  verifyAuthenticationResponse: jest.fn(),
}));

describe('WebAuthnService', () => {
  let service: WebAuthnService;
  let credentialRepository: jest.Mocked<Repository<WebAuthnCredentialEntity>>;
  let challengeRepository: jest.Mocked<Repository<WebAuthnChallengeEntity>>;

  const mockCredential: Partial<WebAuthnCredentialEntity> = {
    id: 'cred-123',
    credentialId: 'credential-id-base64',
    principalId: 'principal-123',
    publicKey: Buffer.from('test-public-key'),
    counter: 5,
    credentialDeviceType: 'singleDevice' as any,
    credentialBackedUp: false,
    transports: ['internal'] as any,
    deviceName: 'Test Device',
    isActive: true,
    createdAt: new Date(),
    lastUsedAt: new Date(),
  };

  beforeEach(async () => {
    const mockCredentialRepository = {
      find: jest.fn(),
      findOne: jest.fn(),
      save: jest.fn(),
      create: jest.fn((data) => data),
      update: jest.fn(),
      createQueryBuilder: jest.fn().mockReturnValue({
        delete: jest.fn().mockReturnThis(),
        from: jest.fn().mockReturnThis(),
        where: jest.fn().mockReturnThis(),
        execute: jest.fn().mockResolvedValue({ affected: 1 }),
      }),
    };

    const mockChallengeRepository = {
      findOne: jest.fn(),
      save: jest.fn(),
      create: jest.fn((data) => data),
      delete: jest.fn(),
      createQueryBuilder: jest.fn().mockReturnValue({
        delete: jest.fn().mockReturnThis(),
        from: jest.fn().mockReturnThis(),
        where: jest.fn().mockReturnThis(),
        execute: jest.fn().mockResolvedValue({ affected: 1 }),
      }),
    };

    const mockConfigService = {
      get: jest.fn((key: string, defaultValue?: string) => {
        const config: Record<string, string> = {
          RP_NAME: 'AgentKern Test',
          RP_ID: 'localhost',
          ORIGIN: 'http://localhost:3000',
        };
        return config[key] || defaultValue;
      }),
    };

    const module: TestingModule = await Test.createTestingModule({
      providers: [
        WebAuthnService,
        {
          provide: getRepositoryToken(WebAuthnCredentialEntity),
          useValue: mockCredentialRepository,
        },
        {
          provide: getRepositoryToken(WebAuthnChallengeEntity),
          useValue: mockChallengeRepository,
        },
        {
          provide: ConfigService,
          useValue: mockConfigService,
        },
      ],
    }).compile();

    service = module.get<WebAuthnService>(WebAuthnService);
    credentialRepository = module.get(getRepositoryToken(WebAuthnCredentialEntity));
    challengeRepository = module.get(getRepositoryToken(WebAuthnChallengeEntity));
    configService = module.get(ConfigService);
  });

  it('should be defined', () => {
    expect(service).toBeDefined();
  });

  describe('generateRegistrationOptions', () => {
    it('should generate registration options for a new principal', async () => {
      credentialRepository.find.mockResolvedValue([]);
      challengeRepository.save.mockResolvedValue({} as WebAuthnChallengeEntity);

      const result = await service.generateRegistrationOptions(
        'principal-123',
        'testuser',
        'Test User',
      );

      expect(result).toBeDefined();
      expect(result.challenge).toBeDefined();
    });

    it('should exclude existing credentials from registration', async () => {
      credentialRepository.find.mockResolvedValue([mockCredential] as WebAuthnCredentialEntity[]);
      challengeRepository.save.mockResolvedValue({} as WebAuthnChallengeEntity);

      const result = await service.generateRegistrationOptions(
        'principal-123',
        'testuser',
        'Test User',
      );

      expect(result).toBeDefined();
      expect(credentialRepository.find).toHaveBeenCalledWith({
        where: { principalId: 'principal-123', isActive: true },
      });
    });
  });

  describe('generateAuthenticationOptions', () => {
    it('should generate authentication options for existing credentials', async () => {
      credentialRepository.find.mockResolvedValue([mockCredential] as WebAuthnCredentialEntity[]);
      challengeRepository.save.mockResolvedValue({} as WebAuthnChallengeEntity);

      const result = await service.generateAuthenticationOptions('principal-123');

      expect(result).toBeDefined();
    });

    it('should return null when no credentials exist', async () => {
      credentialRepository.find.mockResolvedValue([]);

      const result = await service.generateAuthenticationOptions('principal-123');

      expect(result).toBeNull();
    });
  });

  describe('getCredentials', () => {
    it('should return all active credentials for a principal', async () => {
      const credentials = [
        mockCredential,
        { ...mockCredential, id: 'cred-456', credentialId: 'cred-456-base64' },
      ];
      credentialRepository.find.mockResolvedValue(credentials as WebAuthnCredentialEntity[]);

      const result = await service.getCredentials('principal-123');

      expect(result).toHaveLength(2);
      expect(credentialRepository.find).toHaveBeenCalledWith({
        where: { principalId: 'principal-123', isActive: true },
      });
    });

    it('should return empty array when no credentials exist', async () => {
      credentialRepository.find.mockResolvedValue([]);

      const result = await service.getCredentials('nonexistent');

      expect(result).toEqual([]);
    });
  });

  describe('updateCredentialName', () => {
    it('should update device name for a credential', async () => {
      credentialRepository.update.mockResolvedValue({ affected: 1 } as any);

      const result = await service.updateCredentialName(
        'principal-123',
        'cred-123',
        'My Laptop',
      );

      expect(result).toBe(true);
      expect(credentialRepository.update).toHaveBeenCalledWith(
        { id: 'cred-123', principalId: 'principal-123', isActive: true },
        { deviceName: 'My Laptop' },
      );
    });

    it('should return false when credential not found', async () => {
      credentialRepository.update.mockResolvedValue({ affected: 0 } as any);

      const result = await service.updateCredentialName(
        'principal-123',
        'nonexistent',
        'Name',
      );

      expect(result).toBe(false);
    });
  });

  describe('revokeCredential', () => {
    it('should revoke credential with reason', async () => {
      credentialRepository.update.mockResolvedValue({ affected: 1 } as any);

      const result = await service.revokeCredential(
        'principal-123',
        'cred-123',
        'Lost device',
      );

      expect(result).toBe(true);
      expect(credentialRepository.update).toHaveBeenCalledWith(
        { id: 'cred-123', principalId: 'principal-123', isActive: true },
        expect.objectContaining({ isActive: false }),
      );
    });

    it('should return false when credential not found', async () => {
      credentialRepository.update.mockResolvedValue({ affected: 0 } as any);

      const result = await service.revokeCredential('principal-123', 'nonexistent');

      expect(result).toBe(false);
    });
  });
});
