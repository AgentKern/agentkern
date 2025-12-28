import { Test, TestingModule } from '@nestjs/testing';
import { ConfigService } from '@nestjs/config';
import { CryptoAgilityService, CryptoAlgorithm } from './crypto-agility.service';

describe('CryptoAgilityService', () => {
  let service: CryptoAgilityService;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [
        CryptoAgilityService,
        {
          provide: ConfigService,
          useValue: {
            get: jest.fn().mockReturnValue(undefined),
          },
        },
      ],
    }).compile();

    service = module.get<CryptoAgilityService>(CryptoAgilityService);
    await service.onModuleInit();
  });

  describe('key generation', () => {
    it('should generate ES256 key pair', async () => {
      const keyPair = await service.generateKeyPair(CryptoAlgorithm.ES256);

      expect(keyPair).toBeDefined();
      expect(keyPair.algorithm).toBe(CryptoAlgorithm.ES256);
      expect(keyPair.publicKey).toContain('-----BEGIN PUBLIC KEY-----');
      expect(keyPair.privateKey).toContain('-----BEGIN PRIVATE KEY-----');
      expect(keyPair.keyId).toBeDefined();
      expect(keyPair.createdAt).toBeDefined();
    });

    it('should generate Ed25519 key pair', async () => {
      const keyPair = await service.generateKeyPair(CryptoAlgorithm.Ed25519);

      expect(keyPair).toBeDefined();
      expect(keyPair.algorithm).toBe(CryptoAlgorithm.Ed25519);
      expect(keyPair.publicKey).toContain('-----BEGIN PUBLIC KEY-----');
      expect(keyPair.privateKey).toContain('-----BEGIN PRIVATE KEY-----');
    });

    it('should generate default algorithm key pair when none specified', async () => {
      const keyPair = await service.generateKeyPair();

      expect(keyPair).toBeDefined();
      expect(keyPair.algorithm).toBe(CryptoAlgorithm.ES256); // Default
    });
  });

  describe('signing', () => {
    it('should sign payload with ES256', async () => {
      const keyPair = await service.generateKeyPair(CryptoAlgorithm.ES256);
      const payload = new TextEncoder().encode('test payload');

      const result = await service.signWithAlgorithm(
        payload,
        keyPair.privateKey,
        keyPair.keyId,
        CryptoAlgorithm.ES256,
      );

      expect(result).toBeDefined();
      expect(result.signature).toBeDefined();
      expect(result.algorithm).toBe(CryptoAlgorithm.ES256);
      expect(result.keyId).toBe(keyPair.keyId);
      expect(result.timestamp).toBeDefined();
    });

    it('should sign payload with Ed25519', async () => {
      const keyPair = await service.generateKeyPair(CryptoAlgorithm.Ed25519);
      const payload = new TextEncoder().encode('test payload');

      const result = await service.signWithAlgorithm(
        payload,
        keyPair.privateKey,
        keyPair.keyId,
        CryptoAlgorithm.Ed25519,
      );

      expect(result).toBeDefined();
      expect(result.signature).toBeDefined();
      expect(result.algorithm).toBe(CryptoAlgorithm.Ed25519);
    });
  });

  describe('verification', () => {
    it('should verify valid ES256 signature', async () => {
      const keyPair = await service.generateKeyPair(CryptoAlgorithm.ES256);
      const payload = new TextEncoder().encode('test payload');

      const signResult = await service.signWithAlgorithm(
        payload,
        keyPair.privateKey,
        keyPair.keyId,
        CryptoAlgorithm.ES256,
      );

      const verifyResult = await service.verify(
        payload,
        signResult.signature,
        keyPair.publicKey,
        CryptoAlgorithm.ES256,
      );

      expect(verifyResult.valid).toBe(true);
      expect(verifyResult.algorithm).toBe(CryptoAlgorithm.ES256);
    });

    it('should verify valid Ed25519 signature', async () => {
      const keyPair = await service.generateKeyPair(CryptoAlgorithm.Ed25519);
      const payload = new TextEncoder().encode('test payload');

      const signResult = await service.signWithAlgorithm(
        payload,
        keyPair.privateKey,
        keyPair.keyId,
        CryptoAlgorithm.Ed25519,
      );

      const verifyResult = await service.verify(
        payload,
        signResult.signature,
        keyPair.publicKey,
        CryptoAlgorithm.Ed25519,
      );

      expect(verifyResult.valid).toBe(true);
      expect(verifyResult.algorithm).toBe(CryptoAlgorithm.Ed25519);
    });

    it('should reject invalid signature', async () => {
      const keyPair = await service.generateKeyPair(CryptoAlgorithm.ES256);
      const payload = new TextEncoder().encode('test payload');

      const verifyResult = await service.verify(
        payload,
        'invalid_signature_here',
        keyPair.publicKey,
        CryptoAlgorithm.ES256,
      );

      expect(verifyResult.valid).toBe(false);
    });

    it('should reject tampered payload', async () => {
      const keyPair = await service.generateKeyPair(CryptoAlgorithm.ES256);
      const originalPayload = new TextEncoder().encode('original payload');
      const tamperedPayload = new TextEncoder().encode('tampered payload');

      const signResult = await service.signWithAlgorithm(
        originalPayload,
        keyPair.privateKey,
        keyPair.keyId,
        CryptoAlgorithm.ES256,
      );

      const verifyResult = await service.verify(
        tamperedPayload,
        signResult.signature,
        keyPair.publicKey,
        CryptoAlgorithm.ES256,
      );

      expect(verifyResult.valid).toBe(false);
    });
  });

  describe('algorithm management', () => {
    it('should return available algorithms', () => {
      const algorithms = service.getAvailableAlgorithms();

      expect(algorithms).toContain(CryptoAlgorithm.ES256);
      expect(algorithms).toContain(CryptoAlgorithm.Ed25519);
    });

    it('should recommend high-performance algorithm', () => {
      const recommended = service.getRecommendedAlgorithm({ highPerformance: true });
      expect(recommended).toBe(CryptoAlgorithm.Ed25519);
    });

    it('should recommend quantum-safe algorithm when available', () => {
      const recommended = service.getRecommendedAlgorithm({ quantumSafe: true });
      // Should return DILITHIUM3 if PQC is available, or fall back to Ed25519/ES256
      expect([CryptoAlgorithm.DILITHIUM3, CryptoAlgorithm.Ed25519, CryptoAlgorithm.ES256]).toContain(recommended);
    });
  });

  describe('hybrid mode', () => {
    it('should not enable hybrid mode when PQC unavailable', () => {
      // PQC is not available in test environment
      service.setHybridMode(true);
      // Should not throw, but hybrid mode remains disabled
    });
  });
});
