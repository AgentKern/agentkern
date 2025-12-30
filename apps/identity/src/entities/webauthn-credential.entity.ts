/**
 * AgentKernIdentity - WebAuthn Credential Entity
 *
 * Per industry best practices (SimpleWebAuthn, FIDO Alliance, W3C WebAuthn spec):
 * - Stores authenticator credential data persistently
 * - Tracks signature counter for replay attack prevention
 * - Supports multi-device passkey backup status
 *
 * References:
 * - https://simplewebauthn.dev/docs/packages/server
 * - https://www.w3.org/TR/webauthn-2/
 * - https://fidoalliance.org/specifications/
 */

import {
  Entity,
  Column,
  PrimaryColumn,
  CreateDateColumn,
  UpdateDateColumn,
  Index,
  ManyToOne,
  JoinColumn,
} from 'typeorm';

/**
 * CredentialDeviceType per WebAuthn spec
 * - singleDevice: Credential bound to single authenticator (USB key, etc.)
 * - multiDevice: Credential can be synced across devices (iCloud Keychain, Google Password Manager)
 */
export type CredentialDeviceType = 'singleDevice' | 'multiDevice';

/**
 * AuthenticatorTransport per WebAuthn spec
 * Indicates how the browser communicates with the authenticator
 */
export type AuthenticatorTransport = 'usb' | 'nfc' | 'ble' | 'internal' | 'hybrid';

/**
 * WebAuthn Credential Entity
 *
 * Stores all data required to verify future WebAuthn authentication attempts.
 * Each credential is linked to a principal (user) and contains the cryptographic
 * public key and metadata needed for verification.
 */
@Entity('webauthn_credentials')
@Index(['principalId'])
export class WebAuthnCredentialEntity {
  /**
   * Credential ID from the authenticator (base64url encoded)
   * This is the unique identifier provided by the authenticator during registration.
   * Used to look up the credential during authentication.
   */
  @PrimaryColumn('varchar', { length: 512 })
  id: string;

  /**
   * Principal (user) who owns this credential
   */
  @Column('varchar', { length: 255 })
  @Index()
  principalId: string;

  /**
   * The raw public key bytes from the authenticator.
   * Stored as bytea (PostgreSQL) for exact byte preservation.
   * This is the public key used to verify signatures during authentication.
   *
   * SECURITY: This is NOT a secret, but must not be modified.
   */
  @Column('bytea')
  credentialPublicKey: Buffer;

  /**
   * WebAuthn User ID - a user handle unique per relying party.
   * Base64url encoded, used by the authenticator to look up credentials.
   * Per spec: should be random, not PII, maximum 64 bytes.
   */
  @Column('varchar', { length: 128 })
  webauthnUserId: string;

  /**
   * Signature counter - CRITICAL for security.
   * Must be incremented by the authenticator on each authentication.
   * If the received counter is not greater than stored, credential may be cloned.
   *
   * Per WebAuthn spec: If counter goes backwards, consider credential compromised.
   */
  @Column('integer', { default: 0 })
  counter: number;

  /**
   * Device type indicating if credential is single-device or multi-device (synced)
   */
  @Column('varchar', { length: 20 })
  credentialDeviceType: CredentialDeviceType;

  /**
   * Indicates if the passkey is backed up (cloud synced)
   * True for passkeys stored in iCloud Keychain, Google Password Manager, etc.
   */
  @Column('boolean', { default: false })
  credentialBackedUp: boolean;

  /**
   * Transports the authenticator supports for communication.
   * Used to optimize the authentication UX by hinting available transports.
   */
  @Column('simple-array', { nullable: true })
  transports?: AuthenticatorTransport[];

  /**
   * Authenticator Attestation Globally Unique Identifier.
   * Identifies the type/model of authenticator (e.g., YubiKey 5).
   * Useful for policy decisions (e.g., only allow hardware keys).
   */
  @Column('varchar', { length: 36, nullable: true })
  aaguid?: string;

  /**
   * Human-readable name for the credential/device.
   * Allows users to identify which device a passkey belongs to.
   */
  @Column('varchar', { length: 255, nullable: true })
  deviceName?: string;

  /**
   * When this credential was registered
   */
  @CreateDateColumn()
  createdAt: Date;

  /**
   * Last time this credential was used for authentication
   */
  @UpdateDateColumn()
  lastUsedAt: Date;

  /**
   * Whether this credential is still active (not revoked)
   */
  @Column('boolean', { default: true })
  isActive: boolean;

  /**
   * When this credential was revoked (if applicable)
   */
  @Column('timestamp', { nullable: true })
  revokedAt?: Date;

  /**
   * Reason for revocation (if applicable)
   */
  @Column('text', { nullable: true })
  revocationReason?: string;
}

/**
 * WebAuthn Challenge Entity
 *
 * Stores short-lived challenges for registration and authentication flows.
 * Challenges MUST be:
 * - Random and unpredictable (at least 16 bytes of entropy)
 * - Single-use (invalidated after verification)
 * - Time-limited (typically 60-120 seconds)
 *
 * Per WebAuthn spec: Challenges prevent replay attacks.
 */
@Entity('webauthn_challenges')
@Index(['expiresAt'])
export class WebAuthnChallengeEntity {
  /**
   * Principal ID this challenge is for
   */
  @PrimaryColumn('varchar', { length: 255 })
  principalId: string;

  /**
   * The challenge value (base64url encoded random bytes)
   * Minimum 16 bytes (128 bits) of entropy per spec.
   */
  @Column('varchar', { length: 128 })
  challenge: string;

  /**
   * Type of flow this challenge is for
   */
  @Column('varchar', { length: 20 })
  flowType: 'registration' | 'authentication';

  /**
   * When this challenge was created
   */
  @CreateDateColumn()
  createdAt: Date;

  /**
   * When this challenge expires (typically createdAt + 60 seconds)
   */
  @Column('timestamp')
  expiresAt: Date;

  /**
   * Whether this challenge has been used (consumed)
   * Once used, a challenge must never be reused.
   */
  @Column('boolean', { default: false })
  consumed: boolean;
}
