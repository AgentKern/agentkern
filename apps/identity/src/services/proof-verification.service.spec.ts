import { Test, TestingModule } from '@nestjs/testing';
import { getRepositoryToken } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import {
  ProofVerificationService,
  VerificationResult,
  PublicKeyInfo,
} from './proof-verification.service';
import { VerificationKeyEntity } from '../entities/verification-key.entity';

// Mock jose library
jest.mock('jose', () => ({
  importSPKI: jest.fn(),
  compactVerify: jest.fn(),
}));

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
    const mockProofPayload = {
      proofId: 'test-proof-id',
      principal: { id: 'principal-123', credentialId: 'cred-123' },
      agent: { id: 'agent-123' },
      intent: { action: 'test', target: { service: 'svc', endpoint: '/ep' } },
      liability: { acceptedBy: 'agent-123' },
      issuedAt: new Date().toISOString(),
      expiresAt: new Date(Date.now() + 3600000).toISOString(),
    };

    const mockHeader = `v1.${Buffer.from(JSON.stringify(mockProofPayload)).toString('base64url')}.signature`;

    beforeEach(() => {
       // Mock console logs to keep output clean
       jest.spyOn((service as any).logger, 'log').mockImplementation();
       jest.spyOn((service as any).logger, 'warn').mockImplementation();
       jest.spyOn((service as any).logger, 'error').mockImplementation();
    });

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

    it('should verify valid proof successfully', async () => {
       // Mock DB key lookup
       keyRepository.findOne.mockResolvedValue(mockKeyEntity as VerificationKeyEntity);
       keyRepository.update.mockResolvedValue({} as any);

       // Mock JOSE
       const jose = require('jose');
       jose.importSPKI.mockResolvedValue({}); 
       jose.compactVerify.mockResolvedValue(true);

       const result = await service.verifyProof(mockHeader);

       expect(result.valid).toBe(true);
       expect(result.principalId).toBe('principal-123');
       expect(keyRepository.findOne).toHaveBeenCalled();
       expect(keyRepository.update).toHaveBeenCalled(); // Usage count update
    });

    it('should fail if proof is expired', async () => {
       const expiredPayload = { ...mockProofPayload, expiresAt: new Date(Date.now() - 1000).toISOString() };
       const expiredHeader = `v1.${Buffer.from(JSON.stringify(expiredPayload)).toString('base64url')}.signature`;
       
       // Even if signature is valid (we'll mock it to be sure logic flows)
       keyRepository.findOne.mockResolvedValue(mockKeyEntity as VerificationKeyEntity);
       const jose = require('jose');
       jose.importSPKI.mockResolvedValue({}); 
       jose.compactVerify.mockResolvedValue(true);

       const result = await service.verifyProof(expiredHeader);

       expect(result.valid).toBe(false);
       expect(result.errors).toContain('Proof has expired');
    });

    it('should fail if issuedAt is in future', async () => {
       const futurePayload = { ...mockProofPayload, issuedAt: new Date(Date.now() + 10000).toISOString() };
       const futureHeader = `v1.${Buffer.from(JSON.stringify(futurePayload)).toString('base64url')}.signature`;
       
       const result = await service.verifyProof(futureHeader);

       expect(result.valid).toBe(false);
       expect(result.errors).toContain('Proof issuedAt is in the future');
    });

    it('should fail if signature is invalid', async () => {
       keyRepository.findOne.mockResolvedValue(mockKeyEntity as VerificationKeyEntity);
       const jose = require('jose');
       jose.importSPKI.mockResolvedValue({}); 
       jose.compactVerify.mockRejectedValue(new Error('Invalid sig'));

       const result = await service.verifyProof(mockHeader);

       expect(result.valid).toBe(false);
       expect(result.errors).toContain('Invalid signature');
       expect(keyRepository.update).not.toHaveBeenCalled();
    });

    it('should fail if public key not found', async () => {
       keyRepository.findOne.mockResolvedValue(null);

       const result = await service.verifyProof(mockHeader);

       expect(result.valid).toBe(false);
       expect(result.errors).toContain('Invalid signature');
    });

    it('should fail if public key expired', async () => {
       keyRepository.findOne.mockResolvedValue({ 
         ...mockKeyEntity, 
         expiresAt: new Date(Date.now() - 1000),
       } as VerificationKeyEntity);

       const result = await service.verifyProof(mockHeader);

       expect(result.valid).toBe(false);
       expect(result.errors).toContain('Invalid signature'); // logic returns false for signatureValid if key invalid
    });

    it('should validate time constraints', async () => {
       const constrainedPayload = { 
         ...mockProofPayload, 
         constraints: { 
           validHours: { start: 0, end: 1 } // If verify runs at any other time it fails
         } 
       };
       // Mock current time to be OUTSIDE range (unsafe assumption unless robust mocking)
       // Better: pick a range that definitely excludes now
       const nowHour = new Date().getUTCHours();
       const impossibleRange = { start: (nowHour + 2) % 24, end: (nowHour + 3) % 24 }; // Approximate
       
       constrainedPayload.constraints.validHours = impossibleRange;

       const constrainedHeader = `v1.${Buffer.from(JSON.stringify(constrainedPayload)).toString('base64url')}.signature`;
       
       keyRepository.findOne.mockResolvedValue(mockKeyEntity as VerificationKeyEntity);
       const jose = require('jose');
       jose.importSPKI.mockResolvedValue({}); 
       jose.compactVerify.mockResolvedValue(true);

       const result = await service.verifyProof(constrainedHeader);

       expect(result.valid).toBe(false);
       expect(result.errors[0]).toContain('Action not allowed outside valid hours');
    });

    it('should verify Hybrid-PQC signature when bridge is present', async () => {
       const pqcPayload = { ...mockProofPayload };
       const payloadJson = JSON.stringify(pqcPayload);
       const signature = 'classic-sig~pqc-sig';
       const header = `v1.${Buffer.from(payloadJson).toString('base64url')}.${signature}`;

       // Mock logic
       keyRepository.findOne.mockResolvedValue(mockKeyEntity as VerificationKeyEntity);
       keyRepository.update.mockResolvedValue({} as any);

       const jose = require('jose');
       jose.importSPKI.mockResolvedValue({});
       jose.compactVerify.mockResolvedValue(true); // Classic valid

       // Inject Mock Bridge
       const mockBridge = { cryptoVerifyHybrid: jest.fn().mockReturnValue(true) };
       (service as any).bridge = mockBridge;
       (service as any).bridgeLoaded = true;

       const result = await service.verifyProof(header);
       
       expect(result.valid).toBe(true);
       expect(mockBridge.cryptoVerifyHybrid).toHaveBeenCalledWith(
           payloadJson,
           'pqc-sig',
           mockKeyEntity.publicKey
       );
    });

    it('should fail Hybrid-PQC signature if PQC invalid', async () => {
       const signature = 'classic-sig~pqc-sig';
       const header = `v1.${Buffer.from(JSON.stringify(mockProofPayload)).toString('base64url')}.${signature}`;

       keyRepository.findOne.mockResolvedValue(mockKeyEntity as VerificationKeyEntity);
       const jose = require('jose');
       jose.importSPKI.mockResolvedValue({});
       jose.compactVerify.mockResolvedValue(true); // Classic valid

       // Inject Mock Bridge
       const mockBridge = { cryptoVerifyHybrid: jest.fn().mockReturnValue(false) };
       (service as any).bridge = mockBridge;
       (service as any).bridgeLoaded = true;

       const result = await service.verifyProof(header);
       
       expect(result.valid).toBe(false);
       expect(result.errors).toContain('Invalid signature'); // Generic error from catch/false return
    });

    it('should fail Hybrid-PQC signature if bridge not loaded', async () => {
       const signature = 'classic-sig~pqc-sig';
       const header = `v1.${Buffer.from(JSON.stringify(mockProofPayload)).toString('base64url')}.${signature}`;

       keyRepository.findOne.mockResolvedValue(mockKeyEntity as VerificationKeyEntity);
       const jose = require('jose');
       jose.importSPKI.mockResolvedValue({});
       jose.compactVerify.mockResolvedValue(true); // Classic valid

       // Bridge NOT loaded
       (service as any).bridgeLoaded = false;

       const result = await service.verifyProof(header);
       
       expect(result.valid).toBe(false); // Returns false if bridge not loaded
    });
  });

  describe('onModuleInit', () => {
     it('should log active key count', async () => {
        keyRepository.count.mockResolvedValue(10);
        const logSpy = jest.spyOn((service as any).logger, 'log').mockImplementation();
        
        await service.onModuleInit();

        expect(keyRepository.count).toHaveBeenCalledWith({ where: { active: true } });
        expect(logSpy).toHaveBeenCalledWith(expect.stringContaining('10 active keys'));
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
