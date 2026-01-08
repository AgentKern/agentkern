
import { Test, TestingModule } from '@nestjs/testing';
import { INestApplication, Controller, Get, UseGuards, Req } from '@nestjs/common';
import request from 'supertest';
import { App } from 'supertest/types';
import * as crypto from 'crypto';
import { LiabilityProofGuard, OptionalAuthGuard, LiabilityProofPayload } from '../../src/guards/liability-proof.guard';
import { Reflector } from '@nestjs/core';
import { getServer } from '../test-types';

// ============================================================================
// Test Controller to expose guards
// ============================================================================
@Controller('test-security')
class SecurityTestController {
  @Get('strict')
  @UseGuards(LiabilityProofGuard)
  strictAuth(@Req() req: any) {
    return { 
      message: 'secure', 
      user: req.liabilityProof 
    };
  }

  @Get('optional')
  @UseGuards(OptionalAuthGuard)
  optionalAuth(@Req() req: any) {
    return { 
      message: 'optional', 
      user: req.liabilityProof || null
    };
  }
}

// ============================================================================
// Auth Bypass Test Suite
// ============================================================================
describe('Security: Auth Bypass (e2e)', () => {
  let app: INestApplication<App>;
  let keyPair: crypto.KeyPairKeyObjectResult;

  beforeAll(async () => {
    // Generate valid Ed25519 keypair for testing
    keyPair = crypto.generateKeyPairSync('ed25519', {
      publicKeyEncoding: { type: 'spki', format: 'pem' },
      privateKeyEncoding: { type: 'pkcs8', format: 'pem' },
    });

    const moduleFixture: TestingModule = await Test.createTestingModule({
      controllers: [SecurityTestController],
      providers: [LiabilityProofGuard, OptionalAuthGuard, Reflector],
    }).compile();

    app = moduleFixture.createNestApplication();
    await app.init();
  });

  afterAll(async () => {
    if (app) {
      await app.close();
    }
  });

  // Helper to generate JWT
  const generateToken = (payload: any, sign: boolean = true, wrongKey: boolean = false) => {
    const header = { alg: 'EdDSA', typ: 'JWT', kid: generateBase64Key(keyPair.publicKey) };
    const headerB64 = Buffer.from(JSON.stringify(header)).toString('base64url');
    const payloadB64 = Buffer.from(JSON.stringify(payload)).toString('base64url');
    const signingInput = `${headerB64}.${payloadB64}`;

    let signatureB64 = '';
    if (sign) {
      const keyToUse = wrongKey 
        ? crypto.generateKeyPairSync('ed25519', { privateKeyEncoding: { type: 'pkcs8', format: 'pem' } }).privateKey 
        : keyPair.privateKey;
      
      const signature = crypto.sign(null, Buffer.from(signingInput), keyToUse);
      signatureB64 = signature.toString('base64url');
    } else {
      signatureB64 = Buffer.alloc(64).fill(0).toString('base64url'); // Dummy signature
    }

    return `${headerB64}.${payloadB64}.${signatureB64}`;
  };

  const generateBase64Key = (pem: string | Buffer) => {
    // Extract raw key from PEM for kid (simplified for test)
    // In real app, kid is likely public key or hash. Guard expects base64url public key.
    // We need to parse SPKI to get raw key. 
    // For test simplicity, we'll assume the guard handles the key properly if we pass what verifyEd25519Signature expects?
    // Wait, the guard takes kid as publicKeyB64.
    // So we need to provide the raw 32-byte public key in base64url.
    // createPublicKey can import PEM and export raw buffer? No, likely need to strip header/footer.
    
    // Actually, Node crypto createPublicKey supports getting type 'spki' and format 'der'.
    // Then we need to slice the OID prefix for Ed25519.
    const k = crypto.createPublicKey(pem);
    const der = k.export({ format: 'der', type: 'spki' });
    // Ed25519 OID prefix in SPKI is 12 bytes: 30 2a 30 05 06 03 2b 65 70 03 21 00
    // The rest 32 bytes is the key.
    const raw = der.subarray(12);
    return raw.toString('base64url');
  };

  describe('LiabilityProofGuard (Strict)', () => {
    it('should reject requests without header', () => {
      return request(getServer(app))
        .get('/test-security/strict')
        .expect(401)
        .expect((res) => {
          expect(res.body.message).toContain('Missing liability proof header');
        });
    });

    it('should reject invalid JWT format', () => {
      return request(getServer(app))
        .get('/test-security/strict')
        .set('X-AgentKernIdentity', 'invalid.token')
        .expect(401);
    });

    it('should reject valid format but invalid signature', () => {
      const token = generateToken({ iss: 'test', sub: 'test' }, true, true); // Wrong key
      return request(getServer(app))
        .get('/test-security/strict')
        .set('X-AgentKernIdentity', token)
        .expect(401)
        .expect((res) => {
          expect(res.body.message).toContain('Signature verification failed');
        });
    });

    it('should reject expired tokens', () => {
      const token = generateToken({ 
        iss: 'test', 
        sub: 'test',
        exp: Math.floor(Date.now() / 1000) - 3600 // Expired 1 hour ago
      });
      
      return request(getServer(app))
        .get('/test-security/strict')
        .set('X-AgentKernIdentity', token)
        .expect(401)
        .expect((res) => {
          expect(res.body.message).toContain('Token expired');
        });
    });

    it('should accept valid tokens and set verified=true', () => {
      const token = generateToken({ 
        iss: 'test', 
        sub: 'test',
        exp: Math.floor(Date.now() / 1000) + 3600 
      });
      
      return request(getServer(app))
        .get('/test-security/strict')
        .set('X-AgentKernIdentity', token)
        .expect(200)
        .expect((res) => {
          expect(res.body.user.verified).toBe(true);
          expect(res.body.user.issuer).toBe('test');
        });
    });
  });

  describe('OptionalAuthGuard', () => {
    it('should allow requests without header', () => {
      return request(getServer(app))
        .get('/test-security/optional')
        .expect(200)
        .expect((res) => {
          expect(res.body.message).toBe('optional');
          expect(res.body.user).toBeNull();
        });
    });

    it('should accept header and set verified=false (as per current design)', () => {
      // Note: OptionalAuthGuard currently does NOT verify signatures, just parses.
      // So verified should be false.
      const token = generateToken({ iss: 'test', sub: 'test' }, false); // No signature needed technically 
      
      return request(getServer(app))
        .get('/test-security/optional')
        .set('X-AgentKernIdentity', token)
        .expect(200)
        .expect((res) => {
          expect(res.body.user).toBeDefined();
          expect(res.body.user.verified).toBe(false);
        });
    });
  });
});
