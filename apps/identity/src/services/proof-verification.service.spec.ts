import { Test, TestingModule } from '@nestjs/testing';
import { getRepositoryToken } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import {
  ProofVerificationService,
  VerificationResult,
  PublicKeyInfo,
} from './proof-verification.service';
import { VerificationKeyEntity } from '../entities/verification-key.entity';

describe('ProofVerificationService', () => {
  let service: ProofVerificationService;
  let keyRepository: jest.Mocked<Repository<VerificationKeyEntity>>;

  const mockKeyEntity: Partial<VerificationKeyEntity> = {
    id: '123e4567-e89b-12d3-a456-426614174000',
    principalId: 'principal-123',
    credentialId: 'credential-456',
    publicKey: '-----BEGIN PUBLIC KEY-----\nMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAE...\n-----END PUBLIC KEY-----',
    algorithm: 'ES256',
    format: 'pem',
    active: true,
    usageCount: 5,
    createdAt: new Date(),
    updatedAt: new Date(),
    lastUsedAt: new Date(),
  };

  beforeEach(async () => {
    const mockKeyRepository = {
      findOne: jest.fn(),
      find: jest.fn(),
      save: jest.fn(),
      create: jest.fn((data) => ({ ...data, id: 'new-id' })),
      update: jest.fn(),
      count: jest.fn().mockResolvedValue(5),
    };

    const module: TestingModule = await Test.createTestingModule({
      providers: [
        ProofVerificationService,
        {
          provide: getRepositoryToken(VerificationKeyEntity),
          useValue: mockKeyRepository,
        },
      ],
    }).compile();

    service = module.get<ProofVerificationService>(ProofVerificationService);
    keyRepository = module.get(getRepositoryToken(VerificationKeyEntity));
  });

  it('should be defined', () => {
    expect(service).toBeDefined();
  });

  describe('verifyProof', () => {
    it('should return invalid for malformed proof header', async () => {
      const result = await service.verifyProof('invalid-header');

      expect(result.valid).toBe(false);
      expect(result.errors).toContain('Invalid proof format');
    });

    it('should return invalid for empty proof header', async () => {
      const result = await service.verifyProof('');

      expect(result.valid).toBe(false);
      expect(result.errors).toContain('Invalid proof format');
    });
  });

  describe('registerPublicKey', () => {
    it('should create a new key when none exists', async () => {
      keyRepository.findOne.mockResolvedValue(null);
      keyRepository.save.mockResolvedValue(mockKeyEntity as VerificationKeyEntity);

      const keyInfo: PublicKeyInfo = {
        principalId: 'principal-123',
        credentialId: 'credential-456',
        publicKey: 'test-public-key',
        algorithm: 'ES256',
      };

      const result = await service.registerPublicKey(keyInfo);

      expect(result).toBeDefined();
      expect(keyRepository.create).toHaveBeenCalled();
      expect(keyRepository.save).toHaveBeenCalled();
    });

    it('should update existing key when found', async () => {
      const existingKey = { ...mockKeyEntity };
      keyRepository.findOne.mockResolvedValue(existingKey as VerificationKeyEntity);
      keyRepository.save.mockResolvedValue(existingKey as VerificationKeyEntity);

      const keyInfo: PublicKeyInfo = {
        principalId: 'principal-123',
        credentialId: 'credential-456',
        publicKey: 'updated-public-key',
        algorithm: 'ES384',
      };

      const result = await service.registerPublicKey(keyInfo);

      expect(result).toBeDefined();
      expect(existingKey.publicKey).toBe('updated-public-key');
      expect(existingKey.algorithm).toBe('ES384');
      expect(existingKey.active).toBe(true);
    });
  });

  describe('revokeKey', () => {
    it('should revoke key and return true when key exists', async () => {
      keyRepository.update.mockResolvedValue({ affected: 1 } as any);

      const result = await service.revokeKey('principal-123', 'credential-456');

      expect(result).toBe(true);
      expect(keyRepository.update).toHaveBeenCalledWith(
        { principalId: 'principal-123', credentialId: 'credential-456' },
        { active: false },
      );
    });

    it('should return false when key does not exist', async () => {
      keyRepository.update.mockResolvedValue({ affected: 0 } as any);

      const result = await service.revokeKey('nonexistent', 'key');

      expect(result).toBe(false);
    });
  });

  describe('getActiveKeys', () => {
    it('should return all active keys for a principal', async () => {
      const keys = [
        { ...mockKeyEntity, credentialId: 'cred-1' },
        { ...mockKeyEntity, credentialId: 'cred-2' },
      ];
      keyRepository.find.mockResolvedValue(keys as VerificationKeyEntity[]);

      const result = await service.getActiveKeys('principal-123');

      expect(result).toHaveLength(2);
      expect(keyRepository.find).toHaveBeenCalledWith({
        where: { principalId: 'principal-123', active: true },
        order: { createdAt: 'DESC' },
      });
    });

    it('should return empty array when no keys found', async () => {
      keyRepository.find.mockResolvedValue([]);

      const result = await service.getActiveKeys('nonexistent');

      expect(result).toEqual([]);
    });
  });
});
