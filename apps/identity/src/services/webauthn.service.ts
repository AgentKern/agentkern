/**
 * AgentKernIdentity - WebAuthn/Passkey Service
 *
 * Production-ready implementation with TypeORM persistence.
 * Implements WebAuthn for passwordless authentication and Passkey binding.
 *
 * Per W3C WebAuthn Level 2 spec and FIDO Alliance best practices:
 * - Credentials persisted to PostgreSQL via TypeORM
 * - Challenges are short-lived with expiration enforcement
 * - Counter validation for replay attack prevention
 * - Support for multi-device passkeys (cloud-synced)
 *
 * References:
 * - https://www.w3.org/TR/webauthn-2/
 * - https://fidoalliance.org/specifications/
 * - https://simplewebauthn.dev/
 */

import { Injectable, Logger, OnModuleInit } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository, LessThan } from 'typeorm';
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
  PublicKeyCredentialCreationOptionsJSON,
  PublicKeyCredentialRequestOptionsJSON,
  RegistrationResponseJSON,
  AuthenticationResponseJSON,
} from '@simplewebauthn/types';
import {
  WebAuthnCredentialEntity,
  WebAuthnChallengeEntity,
  CredentialDeviceType,
  AuthenticatorTransport,
} from '../entities/webauthn-credential.entity';

/**
 * Interface for credential data returned to callers
 */
export interface StoredCredential {
  id: string;
  credentialPublicKey: Uint8Array;
  counter: number;
  credentialDeviceType: CredentialDeviceType;
  credentialBackedUp: boolean;
  transports?: AuthenticatorTransport[];
  deviceName?: string;
  createdAt: Date;
  lastUsedAt: Date;
}

/**
 * Challenge timeout in milliseconds (60 seconds per WebAuthn recommendation)
 */
const CHALLENGE_TIMEOUT_MS = 60000;

@Injectable()
export class WebAuthnService implements OnModuleInit {
  private readonly logger = new Logger(WebAuthnService.name);

  // Relying Party (RP) configuration
  private rpName: string;
  private rpID: string;
  private origin: string;

  constructor(
    private readonly configService: ConfigService,
    @InjectRepository(WebAuthnCredentialEntity)
    private readonly credentialRepository: Repository<WebAuthnCredentialEntity>,
    @InjectRepository(WebAuthnChallengeEntity)
    private readonly challengeRepository: Repository<WebAuthnChallengeEntity>,
  ) {
    this.rpName = this.configService.get('WEBAUTHN_RP_NAME', 'AgentKernIdentity');
    this.rpID = this.configService.get('WEBAUTHN_RP_ID', 'localhost');
    this.origin = this.configService.get('WEBAUTHN_ORIGIN', 'http://localhost:5004');
  }

  async onModuleInit(): Promise<void> {
    // Clean up any expired challenges on startup
    await this.cleanupExpiredChallenges();
    this.logger.log(`🔐 WebAuthn initialized for RP: ${this.rpName} (${this.rpID})`);
  }

  /**
   * Clean up expired challenges
   * Called on module init; can be called manually or via external scheduler
   */
  async cleanupExpiredChallenges(): Promise<void> {
    const result = await this.challengeRepository.delete({
      expiresAt: LessThan(new Date()),
    });
    if (result.affected && result.affected > 0) {
      this.logger.debug(`Cleaned up ${result.affected} expired challenges`);
    }
  }

  /**
   * Generate registration options for a new principal
   */
  async generateRegistrationOptions(
    principalId: string,
    userName: string,
    displayName: string,
  ): Promise<PublicKeyCredentialCreationOptionsJSON> {
    // Get existing credentials to exclude (prevent re-registration of same authenticator)
    const existingCredentials = await this.credentialRepository.find({
      where: { principalId, isActive: true },
    });

    // Generate a unique WebAuthn user ID (base64url-encoded random bytes)
    const webauthnUserId = Buffer.from(crypto.getRandomValues(new Uint8Array(32))).toString('base64url');

    const options: GenerateRegistrationOptionsOpts = {
      rpName: this.rpName,
      rpID: this.rpID,
      userID: new TextEncoder().encode(webauthnUserId),
      userName,
      userDisplayName: displayName,
      attestationType: 'none', // Direct attestation not needed for most use cases
      excludeCredentials: existingCredentials.map((cred) => ({
        id: cred.id,
        transports: cred.transports as AuthenticatorTransportFuture[],
      })),
      authenticatorSelection: {
        residentKey: 'preferred', // Allow discoverable credentials
        userVerification: 'preferred', // Prefer biometrics but don't require
        authenticatorAttachment: 'platform', // Prefer platform authenticators (passkeys)
      },
    };

    const registrationOptions = await generateRegistrationOptions(options);

    // Store challenge with expiration
    await this.storeChallenge(principalId, registrationOptions.challenge, 'registration');

    this.logger.debug(`Generated registration options for: ${userName}`);

    return registrationOptions;
  }

  /**
   * Verify registration response and persist credential
   */
  async verifyRegistration(
    principalId: string,
    response: RegistrationResponseJSON,
  ): Promise<{
    verified: boolean;
    credentialId?: string;
    error?: string;
  }> {
    // Retrieve and validate challenge
    const challengeEntity = await this.challengeRepository.findOne({
      where: { principalId, flowType: 'registration', consumed: false },
    });

    if (!challengeEntity) {
      return { verified: false, error: 'No active challenge found' };
    }

    if (new Date() > challengeEntity.expiresAt) {
      await this.challengeRepository.delete({ principalId, flowType: 'registration' });
      return { verified: false, error: 'Challenge expired' };
    }

    try {
      const opts: VerifyRegistrationResponseOpts = {
        response,
        expectedChallenge: challengeEntity.challenge,
        expectedOrigin: this.origin,
        expectedRPID: this.rpID,
      };

      const verification: VerifiedRegistrationResponse = await verifyRegistrationResponse(opts);

      if (verification.verified && verification.registrationInfo) {
        const { credential, credentialDeviceType, credentialBackedUp } = verification.registrationInfo;

        // Create and persist credential entity
        const credentialEntity = this.credentialRepository.create({
          id: Buffer.from(credential.id).toString('base64url'),
          principalId,
          credentialPublicKey: Buffer.from(credential.publicKey),
          webauthnUserId: Buffer.from(response.response.publicKey || '').toString('base64url'),
          counter: credential.counter,
          credentialDeviceType: credentialDeviceType as CredentialDeviceType,
          credentialBackedUp,
          transports: response.response.transports as AuthenticatorTransport[],
          isActive: true,
        });

        await this.credentialRepository.save(credentialEntity);

        // Mark challenge as consumed (single-use)
        await this.challengeRepository.delete({ principalId, flowType: 'registration' });

        this.logger.log(`✅ Registered credential for principal: ${principalId}`);

        return {
          verified: true,
          credentialId: credentialEntity.id,
        };
      }

      return { verified: false, error: 'Verification failed' };
    } catch (error) {
      this.logger.error(`Registration verification failed: ${(error as Error).message}`);
      return { verified: false, error: (error as Error).message };
    }
  }

  /**
   * Generate authentication options for a principal
   */
  async generateAuthenticationOptions(
    principalId: string,
  ): Promise<PublicKeyCredentialRequestOptionsJSON | null> {
    const credentials = await this.credentialRepository.find({
      where: { principalId, isActive: true },
    });

    if (credentials.length === 0) {
      return null;
    }

    const opts: GenerateAuthenticationOptionsOpts = {
      rpID: this.rpID,
      allowCredentials: credentials.map((cred) => ({
        id: cred.id,
        transports: cred.transports as AuthenticatorTransportFuture[],
      })),
      userVerification: 'preferred',
    };

    const authOptions = await generateAuthenticationOptions(opts);

    // Store challenge
    await this.storeChallenge(principalId, authOptions.challenge, 'authentication');

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
    // Retrieve and validate challenge
    const challengeEntity = await this.challengeRepository.findOne({
      where: { principalId, flowType: 'authentication', consumed: false },
    });

    if (!challengeEntity) {
      return { verified: false, error: 'No active challenge found' };
    }

    if (new Date() > challengeEntity.expiresAt) {
      await this.challengeRepository.delete({ principalId, flowType: 'authentication' });
      return { verified: false, error: 'Challenge expired' };
    }

    // Find the credential being used
    const credentialId = response.id;
    const credential = await this.credentialRepository.findOne({
      where: { id: credentialId, principalId, isActive: true },
    });

    if (!credential) {
      return { verified: false, error: 'Credential not found or revoked' };
    }

    try {
      const opts: VerifyAuthenticationResponseOpts = {
        response,
        expectedChallenge: challengeEntity.challenge,
        expectedOrigin: this.origin,
        expectedRPID: this.rpID,
        credential: {
          id: credential.id,
          publicKey: credential.credentialPublicKey as Uint8Array<ArrayBuffer>,
          counter: credential.counter,
          transports: credential.transports as AuthenticatorTransportFuture[],
        },
      };

      const verification: VerifiedAuthenticationResponse = await verifyAuthenticationResponse(opts);

      if (verification.verified) {
        // CRITICAL: Update counter to prevent replay attacks
        // Per WebAuthn spec: if newCounter <= storedCounter, credential may be cloned
        const newCounter = verification.authenticationInfo.newCounter;
        if (newCounter <= credential.counter && credential.counter !== 0) {
          this.logger.warn(
            `⚠️ Counter did not increase for credential ${credentialId}. ` +
            `Old: ${credential.counter}, New: ${newCounter}. Possible cloning attack!`
          );
          // You may choose to revoke the credential here for maximum security
        }

        credential.counter = newCounter;
        credential.lastUsedAt = new Date();
        await this.credentialRepository.save(credential);

        // Mark challenge as consumed
        await this.challengeRepository.delete({ principalId, flowType: 'authentication' });

        this.logger.log(`✅ Authenticated principal: ${principalId}`);

        return { verified: true, credentialId: credential.id };
      }

      return { verified: false, error: 'Verification failed' };
    } catch (error) {
      this.logger.error(`Authentication verification failed: ${(error as Error).message}`);
      return { verified: false, error: (error as Error).message };
    }
  }

  /**
   * Get all active credentials for a principal
   */
  async getCredentials(principalId: string): Promise<StoredCredential[]> {
    const credentials = await this.credentialRepository.find({
      where: { principalId, isActive: true },
    });

    return credentials.map((cred) => ({
      id: cred.id,
      credentialPublicKey: cred.credentialPublicKey,
      counter: cred.counter,
      credentialDeviceType: cred.credentialDeviceType,
      credentialBackedUp: cred.credentialBackedUp,
      transports: cred.transports,
      deviceName: cred.deviceName,
      createdAt: cred.createdAt,
      lastUsedAt: cred.lastUsedAt,
    }));
  }

  /**
   * Update device name for a credential (user-friendly identification)
   */
  async updateCredentialName(
    principalId: string,
    credentialId: string,
    deviceName: string,
  ): Promise<boolean> {
    const result = await this.credentialRepository.update(
      { id: credentialId, principalId, isActive: true },
      { deviceName },
    );
    return (result.affected || 0) > 0;
  }

  /**
   * Revoke a credential (soft delete)
   */
  async revokeCredential(
    principalId: string,
    credentialId: string,
    reason?: string,
  ): Promise<boolean> {
    const result = await this.credentialRepository.update(
      { id: credentialId, principalId, isActive: true },
      {
        isActive: false,
        revokedAt: new Date(),
        revocationReason: reason || 'User requested revocation',
      },
    );

    if (result.affected && result.affected > 0) {
      this.logger.log(`🚨 Revoked credential ${credentialId} for ${principalId}: ${reason}`);
      return true;
    }
    return false;
  }

  /**
   * Store a challenge with expiration
   */
  private async storeChallenge(
    principalId: string,
    challenge: string,
    flowType: 'registration' | 'authentication',
  ): Promise<void> {
    // Remove any existing challenges for this principal and flow type
    await this.challengeRepository.delete({ principalId, flowType });

    // Create new challenge with expiration
    const challengeEntity = this.challengeRepository.create({
      principalId,
      challenge,
      flowType,
      expiresAt: new Date(Date.now() + CHALLENGE_TIMEOUT_MS),
      consumed: false,
    });

    await this.challengeRepository.save(challengeEntity);
  }
}
