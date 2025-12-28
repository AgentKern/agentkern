/**
 * AgentKern Identity - WebAuthn/Passkey Service
 * 
 * Implements WebAuthn for passwordless authentication and Passkey binding.
 * This enables principals to bind their Passkeys to their identity.
 */

import { Injectable, Logger } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';
import {
  generateRegistrationOptions,
  verifyRegistrationResponse,
  generateAuthenticationOptions,
  verifyAuthenticationResponse,
} from '@simplewebauthn/server';
import type {
  GenerateRegistrationOptionsOpts,
  VerifyRegistrationResponseOpts,
  GenerateAuthenticationOptionsOpts,
  VerifyAuthenticationResponseOpts,
  VerifiedRegistrationResponse,
  VerifiedAuthenticationResponse,
} from '@simplewebauthn/server';
import type {
  AuthenticatorTransportFuture,
  CredentialDeviceType,
  PublicKeyCredentialCreationOptionsJSON,
  PublicKeyCredentialRequestOptionsJSON,
  RegistrationResponseJSON,
  AuthenticationResponseJSON,
} from '@simplewebauthn/types';

// Stored credential for a user
export interface StoredCredential {
  id: string;
  credentialPublicKey: Uint8Array;
  counter: number;
  credentialDeviceType: CredentialDeviceType;
  credentialBackedUp: boolean;
  transports?: AuthenticatorTransportFuture[];
}

// Principal with credentials
export interface PrincipalCredentials {
  principalId: string;
  credentials: StoredCredential[];
  currentChallenge?: string;
}

@Injectable()
export class WebAuthnService {
  private readonly logger = new Logger(WebAuthnService.name);
  
  // RP (Relying Party) configuration
  private rpName: string;
  private rpID: string;
  private origin: string;

  // In-memory storage (use database in production)
  private principals: Map<string, PrincipalCredentials> = new Map();

  constructor(private readonly configService: ConfigService) {
    this.rpName = this.configService.get('WEBAUTHN_RP_NAME', 'AgentKern Identity');
    this.rpID = this.configService.get('WEBAUTHN_RP_ID', 'localhost');
    this.origin = this.configService.get('WEBAUTHN_ORIGIN', 'http://localhost:5004');
    
    this.logger.log(`🔐 WebAuthn initialized for RP: ${this.rpName} (${this.rpID})`);
  }

  /**
   * Generate registration options for a new principal
   */
  async generateRegistrationOptions(
    principalId: string,
    userName: string,
    displayName: string,
  ): Promise<PublicKeyCredentialCreationOptionsJSON> {
    // Get existing credentials to exclude
    const principal = this.principals.get(principalId);
    const existingCredentials = principal?.credentials || [];

    const options: GenerateRegistrationOptionsOpts = {
      rpName: this.rpName,
      rpID: this.rpID,
      userID: new TextEncoder().encode(principalId),
      userName,
      userDisplayName: displayName,
      attestationType: 'none',
      excludeCredentials: existingCredentials.map((cred) => ({
        id: cred.id,
        transports: cred.transports,
      })),
      authenticatorSelection: {
        residentKey: 'preferred',
        userVerification: 'preferred',
        authenticatorAttachment: 'platform', // Prefer Passkeys
      },
    };

    const registrationOptions = await generateRegistrationOptions(options);

    // Store challenge for verification
    this.storeChallenge(principalId, registrationOptions.challenge);

    this.logger.debug(`Generated registration options for: ${userName}`);

    return registrationOptions;
  }

  /**
   * Verify registration response
   */
  async verifyRegistration(
    principalId: string,
    response: RegistrationResponseJSON,
  ): Promise<{
    verified: boolean;
    credentialId?: string;
    error?: string;
  }> {
    const principal = this.principals.get(principalId);
    const expectedChallenge = principal?.currentChallenge;

    if (!expectedChallenge) {
      return { verified: false, error: 'No challenge found' };
    }

    try {
      const opts: VerifyRegistrationResponseOpts = {
        response,
        expectedChallenge,
        expectedOrigin: this.origin,
        expectedRPID: this.rpID,
      };

      const verification: VerifiedRegistrationResponse =
        await verifyRegistrationResponse(opts);

      if (verification.verified && verification.registrationInfo) {
        const { credential, credentialDeviceType, credentialBackedUp } =
          verification.registrationInfo;

        // Store credential
        const storedCredential: StoredCredential = {
          id: Buffer.from(credential.id).toString('base64url'),
          credentialPublicKey: credential.publicKey,
          counter: credential.counter,
          credentialDeviceType,
          credentialBackedUp,
          transports: response.response.transports,
        };

        this.storeCredential(principalId, storedCredential);

        this.logger.log(`✅ Registered credential for principal: ${principalId}`);

        return {
          verified: true,
          credentialId: storedCredential.id,
        };
      }

      return { verified: false, error: 'Verification failed' };
    } catch (error) {
      this.logger.error(`Registration verification failed: ${error.message}`);
      return { verified: false, error: error.message };
    }
  }

  /**
   * Generate authentication options
   */
  async generateAuthenticationOptions(
    principalId: string,
  ): Promise<PublicKeyCredentialRequestOptionsJSON | null> {
    const principal = this.principals.get(principalId);

    if (!principal || principal.credentials.length === 0) {
      return null;
    }

    const opts: GenerateAuthenticationOptionsOpts = {
      rpID: this.rpID,
      allowCredentials: principal.credentials.map((cred) => ({
        id: cred.id,
        transports: cred.transports,
      })),
      userVerification: 'preferred',
    };

    const authOptions = await generateAuthenticationOptions(opts);

    // Store challenge
    this.storeChallenge(principalId, authOptions.challenge);

    return authOptions;
  }

  /**
   * Verify authentication response
   */
  async verifyAuthentication(
    principalId: string,
    response: AuthenticationResponseJSON,
  ): Promise<{
    verified: boolean;
    credentialId?: string;
    error?: string;
  }> {
    const principal = this.principals.get(principalId);
    const expectedChallenge = principal?.currentChallenge;

    if (!expectedChallenge) {
      return { verified: false, error: 'No challenge found' };
    }

    // Find the credential used
    const credentialId = response.id;
    const credential = principal?.credentials.find(
      (c) => c.id === credentialId,
    );

    if (!credential) {
      return { verified: false, error: 'Credential not found' };
    }

    try {
      const opts: VerifyAuthenticationResponseOpts = {
        response,
        expectedChallenge,
        expectedOrigin: this.origin,
        expectedRPID: this.rpID,
        credential: {
          id: credential.id,
          publicKey: credential.credentialPublicKey as Uint8Array<ArrayBuffer>,
          counter: credential.counter,
          transports: credential.transports,
        },
      };

      const verification: VerifiedAuthenticationResponse =
        await verifyAuthenticationResponse(opts);

      if (verification.verified) {
        // Update counter
        credential.counter = verification.authenticationInfo.newCounter;

        this.logger.log(`✅ Authenticated principal: ${principalId}`);

        return { verified: true, credentialId: credential.id };
      }

      return { verified: false, error: 'Verification failed' };
    } catch (error) {
      this.logger.error(`Authentication verification failed: ${error.message}`);
      return { verified: false, error: error.message };
    }
  }

  /**
   * Get credentials for a principal
   */
  getCredentials(principalId: string): StoredCredential[] {
    return this.principals.get(principalId)?.credentials || [];
  }

  /**
   * Revoke a credential
   */
  revokeCredential(principalId: string, credentialId: string): boolean {
    const principal = this.principals.get(principalId);
    if (!principal) return false;

    const index = principal.credentials.findIndex((c) => c.id === credentialId);
    if (index === -1) return false;

    principal.credentials.splice(index, 1);
    this.logger.log(`🚨 Revoked credential ${credentialId} for ${principalId}`);
    return true;
  }

  private storeChallenge(principalId: string, challenge: string): void {
    let principal = this.principals.get(principalId);
    if (!principal) {
      principal = { principalId, credentials: [] };
      this.principals.set(principalId, principal);
    }
    principal.currentChallenge = challenge;
  }

  private storeCredential(
    principalId: string,
    credential: StoredCredential,
  ): void {
    let principal = this.principals.get(principalId);
    if (!principal) {
      principal = { principalId, credentials: [] };
      this.principals.set(principalId, principal);
    }
    principal.credentials.push(credential);
  }
}
