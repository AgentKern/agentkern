/**
 * WebAuthn Service Full Coverage Tests
 */

import { Test, TestingModule } from '@nestjs/testing';
import { ConfigService } from '@nestjs/config';
import { WebAuthnService } from './webauthn.service';
import * as SimpleWebAuthnServer from '@simplewebauthn/server';

// Mock the entire library
jest.mock('@simplewebauthn/server');

describe('WebAuthnService Full Coverage', () => {
  let service: WebAuthnService;
  
  // Typed mocks
  const mockGenerateRegistrationOptions = SimpleWebAuthnServer.generateRegistrationOptions as jest.Mock;
  const mockVerifyRegistrationResponse = SimpleWebAuthnServer.verifyRegistrationResponse as jest.Mock;
  const mockGenerateAuthenticationOptions = SimpleWebAuthnServer.generateAuthenticationOptions as jest.Mock;
  const mockVerifyAuthenticationResponse = SimpleWebAuthnServer.verifyAuthenticationResponse as jest.Mock;

  beforeEach(async () => {
    jest.clearAllMocks();

    // Default implementations
    mockGenerateRegistrationOptions.mockResolvedValue({
      challenge: 'mock-challenge',
      rp: { name: 'AgentProof', id: 'localhost' },
      user: { id: 'mock-user-id', name: 'user', displayName: 'user' },
      pubKeyCredParams: [],
      timeout: 60000,
      attestation: 'none',
      excludeCredentials: [],
      authenticatorSelection: { userVerification: 'preferred' },
    });

    mockGenerateAuthenticationOptions.mockResolvedValue({
      challenge: 'mock-challenge',
      rpId: 'localhost',
      allowCredentials: [],
      userVerification: 'preferred',
    });

    const module: TestingModule = await Test.createTestingModule({
      providers: [
        WebAuthnService,
        {
          provide: ConfigService,
          useValue: {
            get: jest.fn().mockImplementation((key: string, defaultValue?: any) => {
              const config: Record<string, any> = {
                'WEBAUTHN_RP_NAME': 'AgentProof',
                'WEBAUTHN_RP_ID': 'localhost',
                'WEBAUTHN_ORIGIN': 'http://localhost:3000',
              };
              return config[key] ?? defaultValue;
            }),
          },
        },
      ],
    }).compile();

    service = module.get<WebAuthnService>(WebAuthnService);
  });

  describe('generateRegistrationOptions', () => {
    it('should generate registration options for a new principal', async () => {
      const options = await service.generateRegistrationOptions(
        'principal-1',
        'user@example.com',
        'Test User',
      );

      expect(options).toBeDefined();
      expect(mockGenerateRegistrationOptions).toHaveBeenCalled();
      expect(options.challenge).toBe('mock-challenge');
    });

    it('should handle re-registration with excluded credentials', async () => {
      // First register a credential to populate the principal
      mockVerifyRegistrationResponse.mockResolvedValueOnce({
        verified: true,
        registrationInfo: {
          credential: { id: 'cred-id', publicKey: Buffer.from('pk'), counter: 0 },
          credentialDeviceType: 'singleDevice',
          credentialBackedUp: false,
        },
      });

      // Simulating the flow: Start -> Verify -> Start Again
      await service.generateRegistrationOptions('principal-re', 'user@test.com');
      // Mock successful verification to store credential
      await service.verifyRegistration('principal-re', {
          id: 'cred-id-raw',
          rawId: 'cred-id-raw',
          response: { transports: [] },
          type: 'public-key'
      } as any);

      // Now generate again
      await service.generateRegistrationOptions('principal-re', 'user@test.com');
      
      // Check if excludeCredentials was passed
      const lastCall = mockGenerateRegistrationOptions.mock.calls[mockGenerateRegistrationOptions.mock.calls.length - 1][0];
      expect(lastCall.excludeCredentials).toBeDefined();
    });
  });

  describe('verifyRegistration', () => {
    it('should successfully verify and store credential', async () => {
      // Setup: Generate options first to create principal with challenge
      await service.generateRegistrationOptions('verify-success', 'user@test.com');

      // Mock success response from library
      mockVerifyRegistrationResponse.mockResolvedValueOnce({
        verified: true,
        registrationInfo: {
          credential: { 
            id: 'cred-123', 
            publicKey: Buffer.from('mock-public-key'), 
            counter: 0 
          },
          credentialDeviceType: 'singleDevice',
          credentialBackedUp: true,
        },
      });

      const result = await service.verifyRegistration('verify-success', {
        id: 'cred-123-raw',
        rawId: 'cred-123-raw',
        type: 'public-key',
        response: { 
            clientDataJSON: 'mock-client-data',
            attestationObject: 'mock-attestation',
            transports: ['usb'] 
        },
      } as any);

      expect(result.verified).toBe(true);
      expect(result.credentialId).toBeDefined();
      expect(mockVerifyRegistrationResponse).toHaveBeenCalled();

      // Verify credential is stored
      const creds = service.getCredentials('verify-success');
      expect(creds.length).toBe(1);
    });

    it('should handle verification failure from library', async () => {
      await service.generateRegistrationOptions('verify-fail', 'user@test.com');

      mockVerifyRegistrationResponse.mockResolvedValueOnce({
        verified: false,
      });

      const result = await service.verifyRegistration('verify-fail', {
        // ... valid structure, but mock returns false
        id: 'fail', rawId: 'fail', type: 'public-key', response: {}
      } as any);

      expect(result.verified).toBe(false);
      expect(result.error).toBe('Verification failed');
    });

    it('should catch and return errors during verification', async () => {
      await service.generateRegistrationOptions('verify-error', 'user@test.com');

      mockVerifyRegistrationResponse.mockRejectedValueOnce(new Error('Library error'));

      const result = await service.verifyRegistration('verify-error', {
          id: 'error', rawId: 'error', type: 'public-key', response: {}
      } as any);

      expect(result.verified).toBe(false);
      expect(result.error).toBe('Library error');
    });

    it('should return error when no challenge found', async () => {
      const result = await service.verifyRegistration('unknown', {} as any);
      expect(result.verified).toBe(false);
      expect(result.error).toBe('No challenge found');
    });
  });

  describe('generateAuthenticationOptions', () => {
    it('should generate options for existing principal with credentials', async () => {
      // Setup principal with credential
      const principalId = 'auth-opt-success';
      await service.generateRegistrationOptions(principalId, 'user@test.com');
      
      mockVerifyRegistrationResponse.mockResolvedValueOnce({
        verified: true,
        registrationInfo: {
          credential: { id: 'cred-1', publicKey: Buffer.from('pk'), counter: 0 },
          credentialDeviceType: 'singleDevice',
          credentialBackedUp: false,
        },
      });
      await service.verifyRegistration(principalId, { response: { transports: [] } } as any);

      // Test auth options generation
      const options = await service.generateAuthenticationOptions(principalId);
      
      expect(options).toBeDefined();
      expect(options?.challenge).toBe('mock-challenge');
      expect(mockGenerateAuthenticationOptions).toHaveBeenCalled();
    });

    it('should return null for unknown principal', async () => {
      const options = await service.generateAuthenticationOptions('unknown');
      expect(options).toBeNull();
    });
    
    it('should return null for principal with no credentials', async () => {
        // Register but don't verify => no credentials
        await service.generateRegistrationOptions('no-creds', 'user@test.com');
        const options = await service.generateAuthenticationOptions('no-creds');
        expect(options).toBeNull();
    });
  });

  describe('verifyAuthentication', () => {
    it('should successfully verify authentication', async () => {
      const principalId = 'auth-verify-success';
      const rawCredId = 'cred-1';
      const storedCredId = Buffer.from(rawCredId).toString('base64url');
      
      // 1. Register
      await service.generateRegistrationOptions(principalId, 'user@test.com');
      mockVerifyRegistrationResponse.mockResolvedValueOnce({
        verified: true,
        registrationInfo: {
          credential: { id: rawCredId, publicKey: Buffer.from('pk'), counter: 0 },
          credentialDeviceType: 'singleDevice',
          credentialBackedUp: false,
        },
      });
      await service.verifyRegistration(principalId, { response: { transports: [] } } as any);

      // 2. Start Auth
      await service.generateAuthenticationOptions(principalId);

      // 3. Verify Auth
      mockVerifyAuthenticationResponse.mockResolvedValueOnce({
        verified: true,
        authenticationInfo: {
          newCounter: 1,
          credentialID: storedCredId, 
        },
      });

      const result = await service.verifyAuthentication(principalId, {
        id: storedCredId, 
        rawId: storedCredId,
        type: 'public-key',
        response: {
            clientDataJSON: 'json',
            authenticatorData: 'authdata',
            signature: 'sig',
            userHandle: 'handle'
        }
      } as any);

      expect(result.verified).toBe(true);
      expect(mockVerifyAuthenticationResponse).toHaveBeenCalled();
      
      // Verify counter updated
      const creds = service.getCredentials(principalId);
      expect(creds[0].counter).toBe(1);
    });

    it('should fail when library verification fails', async () => {
        const principalId = 'auth-verify-fail';
        const rawCredId = 'cred-1';
        const storedCredId = Buffer.from(rawCredId).toString('base64url');

        // Setup done...
        await service.generateRegistrationOptions(principalId, 'user@test.com');
        mockVerifyRegistrationResponse.mockResolvedValueOnce({
            verified: true,
            registrationInfo: {
              credential: { id: rawCredId, publicKey: Buffer.from('pk'), counter: 0 },
              credentialDeviceType: 'singleDevice',
            credentialBackedUp: false,
            },
        });
        await service.verifyRegistration(principalId, { response: { transports: [] } } as any);
        await service.generateAuthenticationOptions(principalId);

        mockVerifyAuthenticationResponse.mockResolvedValueOnce({ verified: false });

        const result = await service.verifyAuthentication(principalId, {
            id: storedCredId,
            response: {}
        } as any);

        expect(result.verified).toBe(false);
        expect(result.error).toBe('Verification failed');
    });

    it('should return error for credential not found', async () => {
        const principalId = 'auth-cred-missing';
        await service.generateRegistrationOptions(principalId, 'user@test.com');
        // Register a credential
        mockVerifyRegistrationResponse.mockResolvedValueOnce({
            verified: true,
            registrationInfo: { credential: { id: 'c1', publicKey: Buffer.from('pk'), counter: 0 }, credentialDeviceType: 'x', credentialBackedUp: false },
        });
        await service.verifyRegistration(principalId, { response: { transports: [] } } as any);
        await service.generateAuthenticationOptions(principalId);

        // Try to verify with WRONG credential ID
        const result = await service.verifyAuthentication(principalId, {
            id: 'wrong-id',
            response: {}
        } as any);
        
        expect(result.verified).toBe(false);
        expect(result.error).toBe('Credential not found');
    });

     it('should handle exceptions during verification', async () => {
        const principalId = 'auth-verify-except';
        const rawCredId = 'c1';
        const storedCredId = Buffer.from(rawCredId).toString('base64url');

        await service.generateRegistrationOptions(principalId, 'user@test.com');
        mockVerifyRegistrationResponse.mockResolvedValueOnce({
            verified: true,
            registrationInfo: { credential: { id: rawCredId, publicKey: Buffer.from('pk'), counter: 0 }, credentialDeviceType: 'x', credentialBackedUp: false },
        });
        await service.verifyRegistration(principalId, { response: { transports: [] } } as any);
        await service.generateAuthenticationOptions(principalId);

        mockVerifyAuthenticationResponse.mockRejectedValueOnce(new Error('Auth Library Error'));

        const result = await service.verifyAuthentication(principalId, {
            id: storedCredId,
            response: {}
        } as any);
        
        expect(result.verified).toBe(false);
        expect(result.error).toBe('Auth Library Error');
    });
  });

  describe('revokeCredential', () => {
      it('should revoke an existing credential', async () => {
          const principalId = 'revoke-success';
          await service.generateRegistrationOptions(principalId, 'user@test.com');
           mockVerifyRegistrationResponse.mockResolvedValueOnce({
            verified: true,
            registrationInfo: { credential: { id: 'c1', publicKey: Buffer.from('pk'), counter: 0 }, credentialDeviceType: 'x', credentialBackedUp: false },
          });
          const regResult = await service.verifyRegistration(principalId, { response: { transports: [] } } as any);
          const credId = regResult.credentialId;

          const revoked = service.revokeCredential(principalId, credId!);
          expect(revoked).toBe(true);
          expect(service.getCredentials(principalId).length).toBe(0);
      });
      
      it('should return false if credential does not exist', async () => {
          const principalId = 'revoke-fail';
          await service.generateRegistrationOptions(principalId, 'user@test.com');
          const revoked = service.revokeCredential(principalId, 'missing');
          expect(revoked).toBe(false);
      });
  });

  describe('private methods coverage', () => {
    it('should create principal if missing when storing challenge directly', () => {
      const principalId = 'direct-challenge';
      (service as any).storeChallenge(principalId, 'challenge-123');
      
      const principal = (service as any).principals.get(principalId);
      expect(principal).toBeDefined();
      expect(principal.currentChallenge).toBe('challenge-123');
    });

    it('should create principal if missing when storing credential directly', () => {
      const principalId = 'direct-credential';
      const cred = { 
          id: 'cred-1', 
          credentialPublicKey: Buffer.from('pk'), 
          counter: 0,
          transports: undefined,
          credentialDeviceType: 'singleDevice',
          credentialBackedUp: false
      };
      
      (service as any).storeCredential(principalId, cred);
      
      const principal = (service as any).principals.get(principalId);
      expect(principal).toBeDefined();
      expect(principal.credentials.length).toBe(1);
    });
  });
});

