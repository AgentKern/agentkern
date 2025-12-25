/**
 * AgentProof - Liability Proof Entity Tests
 * 
 * Unit tests for domain entity functions.
 * Follows mandate: 100% testing coverage.
 */

import {
  parseProofHeader,
  serializeProofHeader,
  createProofPayload,
  LiabilityProof,
  LiabilityProofPayload,
} from './liability-proof.entity';

describe('LiabilityProofEntity', () => {
  const mockPrincipal = {
    id: 'user-123',
    credentialId: 'cred-456',
    deviceAttestation: 'attestation-hash',
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
    parameters: { amount: 1000 },
  };

  describe('createProofPayload', () => {
    it('should create a valid proof payload with default options', () => {
      const payload = createProofPayload(mockPrincipal, mockAgent, mockIntent);

      expect(payload.version).toBe('1.0');
      expect(payload.proofId).toBeDefined();
      expect(payload.issuedAt).toBeDefined();
      expect(payload.expiresAt).toBeDefined();
      expect(payload.principal).toEqual(mockPrincipal);
      expect(payload.agent).toEqual(mockAgent);
      expect(payload.intent).toEqual(mockIntent);
      expect(payload.liability.acceptedBy).toBe('principal');
      expect(payload.liability.disputeWindowHours).toBe(72);
    });

    it('should respect custom expiration time', () => {
      const payload = createProofPayload(mockPrincipal, mockAgent, mockIntent, {
        expiresInSeconds: 600,
      });

      const issuedAt = new Date(payload.issuedAt).getTime();
      const expiresAt = new Date(payload.expiresAt).getTime();
      const diffSeconds = (expiresAt - issuedAt) / 1000;

      expect(diffSeconds).toBe(600);
    });

    it('should include constraints when provided', () => {
      const constraints = {
        maxAmount: 5000,
        geoFence: ['US', 'CA'],
      };

      const payload = createProofPayload(mockPrincipal, mockAgent, mockIntent, {
        constraints,
      });

      expect(payload.constraints).toEqual(constraints);
    });

    it('should respect custom dispute window', () => {
      const payload = createProofPayload(mockPrincipal, mockAgent, mockIntent, {
        disputeWindowHours: 24,
      });

      expect(payload.liability.disputeWindowHours).toBe(24);
    });
  });

  describe('parseProofHeader', () => {
    it('should parse a valid proof header', () => {
      const payload: LiabilityProofPayload = {
        version: '1.0',
        proofId: 'test-id',
        issuedAt: new Date().toISOString(),
        expiresAt: new Date(Date.now() + 300000).toISOString(),
        principal: mockPrincipal,
        agent: mockAgent,
        intent: mockIntent,
        liability: {
          acceptedBy: 'principal',
          termsVersion: '1.0',
          disputeWindowHours: 72,
        },
      };

      const payloadBase64 = Buffer.from(JSON.stringify(payload)).toString('base64url');
      const header = `v1.${payloadBase64}.mock-signature`;

      const result = parseProofHeader(header);

      expect(result).not.toBeNull();
      expect(result!.version).toBe('v1');
      expect(result!.payload.proofId).toBe('test-id');
      expect(result!.signature).toBe('mock-signature');
    });

    it('should return null for invalid header format', () => {
      expect(parseProofHeader('invalid')).toBeNull();
      expect(parseProofHeader('only.two')).toBeNull();
      expect(parseProofHeader('')).toBeNull();
    });

    it('should return null for invalid base64 payload', () => {
      const result = parseProofHeader('v1.!!!invalid-base64!!!.signature');
      expect(result).toBeNull();
    });

    it('should return null for invalid JSON payload', () => {
      const invalidJson = Buffer.from('not-json').toString('base64url');
      const result = parseProofHeader(`v1.${invalidJson}.signature`);
      expect(result).toBeNull();
    });
  });

  describe('serializeProofHeader', () => {
    it('should serialize a proof to header format', () => {
      const proof: LiabilityProof = {
        version: 'v1',
        payload: {
          version: '1.0',
          proofId: 'test-id',
          issuedAt: '2025-12-24T10:00:00Z',
          expiresAt: '2025-12-24T10:05:00Z',
          principal: mockPrincipal,
          agent: mockAgent,
          intent: mockIntent,
          liability: {
            acceptedBy: 'principal',
            termsVersion: '1.0',
            disputeWindowHours: 72,
          },
        },
        signature: 'test-signature',
      };

      const header = serializeProofHeader(proof);
      const parts = header.split('.');

      expect(parts.length).toBe(3);
      expect(parts[0]).toBe('v1');
      expect(parts[2]).toBe('test-signature');

      // Verify payload can be decoded
      const decodedPayload = JSON.parse(
        Buffer.from(parts[1], 'base64url').toString('utf-8'),
      );
      expect(decodedPayload.proofId).toBe('test-id');
    });

    it('should produce a header that can be parsed back', () => {
      const proof: LiabilityProof = {
        version: 'v1',
        payload: createProofPayload(mockPrincipal, mockAgent, mockIntent),
        signature: 'roundtrip-signature',
      };

      const header = serializeProofHeader(proof);
      const parsed = parseProofHeader(header);

      expect(parsed).not.toBeNull();
      expect(parsed!.payload.proofId).toBe(proof.payload.proofId);
      expect(parsed!.signature).toBe('roundtrip-signature');
    });
  });
});
