/**
 * Proof Signing Service Tests
 */

import { Test, TestingModule } from '@nestjs/testing';
import { ProofSigningService } from './proof-signing.service';

describe('ProofSigningService', () => {
  let service: ProofSigningService;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [ProofSigningService],
    }).compile();

    service = module.get<ProofSigningService>(ProofSigningService);
  });

  describe('generateKeyPair', () => {
    it('should generate a key pair', async () => {
      const keyPair = await service.generateKeyPair();
      expect(keyPair.publicKey).toBeDefined();
      expect(keyPair.privateKey).toBeDefined();
      expect(keyPair.publicKey.includes('BEGIN PUBLIC KEY')).toBe(true);
      expect(keyPair.privateKey.includes('BEGIN PRIVATE KEY')).toBe(true);
    });
  });

  describe('createSignedProof', () => {
    it('should create a signed proof', async () => {
      const keyPair = await service.generateKeyPair();
      
      const result = await service.createSignedProof({
        principal: { id: 'principal-1', credentialId: 'cred-1' },
        agent: { id: 'agent-1', name: 'Test Agent', version: '1.0.0' },
        intent: {
          action: 'transfer',
          target: { service: 'bank', endpoint: '/transfer', method: 'POST' },
          parameters: { amount: 100 },
        },
        privateKey: keyPair.privateKey,
      });
      
      expect(result.version).toBe('v1');
      expect(result.signature).toBeDefined();
      expect(result.payload.proofId).toBeDefined();
      expect(result.payload.principal.id).toBe('principal-1');
    });

    it('should include custom constraints', async () => {
      const keyPair = await service.generateKeyPair();
      
      const result = await service.createSignedProof({
        principal: { id: 'p1', credentialId: 'c1' },
        agent: { id: 'a1', name: 'Agent', version: '1.0' },
        intent: {
          action: 'test',
          target: { service: 's', endpoint: '/e', method: 'GET' },
        },
        constraints: {
          maxAmount: 1000,
          allowedMethods: ['GET', 'POST'],
        },
        expiresInSeconds: 60,
        privateKey: keyPair.privateKey,
      });
      
      expect(result.payload.constraints?.maxAmount).toBe(1000);
    });
  });
});
