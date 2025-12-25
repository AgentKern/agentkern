/**
 * AgentProof - Proof Verification Service Tests
 * 
 * Unit tests for proof verification logic.
 * Follows mandate: 100% testing coverage.
 */

import { Test, TestingModule } from '@nestjs/testing';
import { ProofVerificationService, PublicKeyInfo } from './proof-verification.service';
import { createProofPayload, serializeProofHeader, LiabilityProof } from '../domain/liability-proof.entity';
import * as jose from 'jose';

describe('ProofVerificationService', () => {
  let service: ProofVerificationService;

  const mockPrincipal = {
    id: 'user-123',
    credentialId: 'cred-456',
  };

  const mockAgent = {
    id: 'agent-789',
    name: 'test-agent',
    version: '1.0.0',
  };

  const mockIntent = {
    action: 'transfer',
    target: {
      service: 'api.bank.com',
      endpoint: '/v1/transfers',
      method: 'POST' as const,
    },
  };

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [ProofVerificationService],
    }).compile();

    service = module.get<ProofVerificationService>(ProofVerificationService);
  });

  it('should be defined', () => {
    expect(service).toBeDefined();
  });

  describe('verifyProof', () => {
    it('should reject invalid proof format', async () => {
      const result = await service.verifyProof('invalid-format');

      expect(result.valid).toBe(false);
      expect(result.errors).toContain('Invalid proof format');
    });

    it('should reject expired proof', async () => {
      const expiredPayload = createProofPayload(mockPrincipal, mockAgent, mockIntent, {
        expiresInSeconds: -60, // Expired 60 seconds ago
      });

      const proof: LiabilityProof = {
        version: 'v1',
        payload: expiredPayload,
        signature: 'mock-signature',
      };

      const header = serializeProofHeader(proof);
      const result = await service.verifyProof(header);

      expect(result.valid).toBe(false);
      expect(result.errors).toContain('Proof has expired');
    });

    it('should reject proof with future issuedAt', async () => {
      const futurePayload = {
        ...createProofPayload(mockPrincipal, mockAgent, mockIntent),
        issuedAt: new Date(Date.now() + 60000).toISOString(), // 1 minute in future
      };

      const proof: LiabilityProof = {
        version: 'v1',
        payload: futurePayload,
        signature: 'mock-signature',
      };

      const header = serializeProofHeader(proof);
      const result = await service.verifyProof(header);

      expect(result.valid).toBe(false);
      expect(result.errors).toContain('Proof issuedAt is in the future');
    });

    it('should reject proof without registered public key', async () => {
      const payload = createProofPayload(mockPrincipal, mockAgent, mockIntent);

      const proof: LiabilityProof = {
        version: 'v1',
        payload,
        signature: 'mock-signature',
      };

      const header = serializeProofHeader(proof);
      const result = await service.verifyProof(header);

      expect(result.valid).toBe(false);
      expect(result.errors).toContain('Invalid signature');
    });

    it('should include proofId in result even for failures', async () => {
      const payload = createProofPayload(mockPrincipal, mockAgent, mockIntent, {
        expiresInSeconds: -60,
      });

      const proof: LiabilityProof = {
        version: 'v1',
        payload,
        signature: 'mock-signature',
      };

      const header = serializeProofHeader(proof);
      const result = await service.verifyProof(header);

      expect(result.proofId).toBe(payload.proofId);
    });

    it('should return valid result with principal and agent info on success', async () => {
      // Generate real keys for proper verification
      const { publicKey, privateKey } = await jose.generateKeyPair('ES256');
      const publicKeyPem = await jose.exportSPKI(publicKey);

      // Register the key
      service.registerPublicKey({
        principalId: mockPrincipal.id,
        credentialId: mockPrincipal.credentialId,
        publicKey: publicKeyPem,
        algorithm: 'ES256',
      });

      const payload = createProofPayload(mockPrincipal, mockAgent, mockIntent);
      const payloadJson = JSON.stringify(payload);
      
      // Create real signature
      const jws = await new jose.CompactSign(new TextEncoder().encode(payloadJson))
        .setProtectedHeader({ alg: 'ES256' })
        .sign(privateKey);
      
      const signaturePart = jws.split('.')[2];

      const proof: LiabilityProof = {
        version: 'v1',
        payload,
        signature: signaturePart,
      };

      const header = serializeProofHeader(proof);
      const result = await service.verifyProof(header);

      expect(result.valid).toBe(true);
      expect(result.principalId).toBe(mockPrincipal.id);
      expect(result.agentId).toBe(mockAgent.id);
      expect(result.intent?.action).toBe('transfer');
    });
  });

  describe('registerPublicKey', () => {
    it('should register a public key successfully', () => {
      const keyInfo: PublicKeyInfo = {
        principalId: 'user-123',
        credentialId: 'cred-456',
        publicKey: '-----BEGIN PUBLIC KEY-----\ntest\n-----END PUBLIC KEY-----',
        algorithm: 'ES256',
      };

      // Should not throw
      expect(() => service.registerPublicKey(keyInfo)).not.toThrow();
    });
  });

  describe('verifySignature (private method)', () => {
    it('should return false when key import fails', async () => {
      // Register an invalid key
      service.registerPublicKey({
        principalId: 'bad-key-user',
        credentialId: 'bad-cred',
        publicKey: 'invalid-not-a-real-key',
        algorithm: 'ES256',
      });

      const payload = createProofPayload(
        { id: 'bad-key-user', credentialId: 'bad-cred' },
        mockAgent,
        mockIntent,
      );

      const proof: LiabilityProof = {
        version: 'v1',
        payload,
        signature: 'fake-sig',
      };

      const header = serializeProofHeader(proof);
      const result = await service.verifyProof(header);

      expect(result.valid).toBe(false);
      expect(result.errors).toContain('Invalid signature');
    });
  });

  describe('validateConstraints', () => {
    it('should pass validation for proof without constraints', async () => {
      const payload = createProofPayload(mockPrincipal, mockAgent, mockIntent);
      // No constraints - should only fail on signature, not constraints
      const proof: LiabilityProof = {
        version: 'v1',
        payload,
        signature: 'mock',
      };

      const header = serializeProofHeader(proof);
      const result = await service.verifyProof(header);

      // Should only have signature error, not constraint errors
      expect(result.errors?.includes('Invalid signature')).toBe(true);
      expect(result.errors?.length).toBe(1);
    });

    it('should reject proof outside valid hours', async () => {
      // Set hours that are definitely not now
      const currentHour = new Date().getUTCHours();
      const invalidStart = (currentHour + 2) % 24;
      const invalidEnd = (currentHour + 4) % 24;

      const payload = createProofPayload(mockPrincipal, mockAgent, mockIntent, {
        constraints: {
          validHours: { start: invalidStart, end: invalidEnd },
        },
      });

      const proof: LiabilityProof = {
        version: 'v1',
        payload,
        signature: 'mock',
      };

      const header = serializeProofHeader(proof);
      const result = await service.verifyProof(header);

      const hasTimeError = result.errors?.some((e) =>
        e.includes('Action not allowed outside valid hours'),
      );
      expect(hasTimeError).toBe(true);
    });

    it('should log geoFence constraints', async () => {
      const payload = createProofPayload(mockPrincipal, mockAgent, mockIntent, {
        constraints: {
          geoFence: ['US', 'CA'],
        },
      });

      const proof: LiabilityProof = {
        version: 'v1',
        payload,
        signature: 'mock',
      };

      const header = serializeProofHeader(proof);
      const result = await service.verifyProof(header);

      // GeoFence is logged but not validated - only signature error
      expect(result.errors?.length).toBe(1);
    });
  });
});

