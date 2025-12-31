/**
 * AgentKernIdentity - WebAuthn DTOs
 *
 * Data Transfer Objects for WebAuthn endpoints.
 * Types follow W3C WebAuthn Level 3 specification (2025).
 *
 * @see https://www.w3.org/TR/webauthn-3/
 * @see https://simplewebauthn.dev/docs/packages/types
 */

import { IsString, IsNotEmpty, IsOptional, IsObject } from 'class-validator';
import { ApiProperty, ApiPropertyOptional } from '@nestjs/swagger';

// ============================================================================
// WebAuthn Types (W3C WebAuthn Level 3 Specification)
// ============================================================================

/**
 * AuthenticatorAttestationResponse JSON representation.
 * Used during credential registration.
 */
export interface AuthenticatorAttestationResponseJSON {
  clientDataJSON: string; // Base64URL encoded
  attestationObject: string; // Base64URL encoded
  transports?: AuthenticatorTransport[];
  publicKeyAlgorithm?: number;
  publicKey?: string; // Base64URL encoded
  authenticatorData?: string; // Base64URL encoded
}

/**
 * AuthenticatorAssertionResponse JSON representation.
 * Used during authentication.
 */
export interface AuthenticatorAssertionResponseJSON {
  clientDataJSON: string; // Base64URL encoded
  authenticatorData: string; // Base64URL encoded
  signature: string; // Base64URL encoded
  userHandle?: string | null; // Base64URL encoded
}

/**
 * PublicKeyCredential extension results.
 */
export interface AuthenticationExtensionsClientOutputs {
  appid?: boolean;
  credProps?: { rk?: boolean };
  largeBlob?: { blob?: string; written?: boolean };
  prf?: { enabled?: boolean; results?: { first?: string; second?: string } };
}

/**
 * Registration response from authenticator (RegistrationResponseJSON).
 */
export interface RegistrationResponseJSON {
  id: string; // Base64URL credential ID
  rawId: string; // Base64URL raw credential ID
  response: AuthenticatorAttestationResponseJSON;
  authenticatorAttachment?: AuthenticatorAttachment;
  clientExtensionResults: AuthenticationExtensionsClientOutputs;
  type: 'public-key';
}

/**
 * Authentication response from authenticator (AuthenticationResponseJSON).
 */
export interface AuthenticationResponseJSON {
  id: string; // Base64URL credential ID
  rawId: string; // Base64URL raw credential ID
  response: AuthenticatorAssertionResponseJSON;
  authenticatorAttachment?: AuthenticatorAttachment;
  clientExtensionResults: AuthenticationExtensionsClientOutputs;
  type: 'public-key';
}

/**
 * Authenticator transport hints.
 */
export type AuthenticatorTransport =
  | 'usb'
  | 'nfc'
  | 'ble'
  | 'smart-card'
  | 'hybrid'
  | 'internal';

/**
 * Authenticator attachment modality.
 */
export type AuthenticatorAttachment = 'platform' | 'cross-platform';

/**
 * Public key credential creation options returned to client.
 */
export interface PublicKeyCredentialCreationOptionsJSON {
  rp: { id?: string; name: string };
  user: { id: string; name: string; displayName: string };
  challenge: string; // Base64URL
  pubKeyCredParams: Array<{ alg: number; type: 'public-key' }>;
  timeout?: number;
  excludeCredentials?: Array<{
    id: string;
    type: 'public-key';
    transports?: AuthenticatorTransport[];
  }>;
  authenticatorSelection?: {
    authenticatorAttachment?: AuthenticatorAttachment;
    residentKey?: 'discouraged' | 'preferred' | 'required';
    requireResidentKey?: boolean;
    userVerification?: 'required' | 'preferred' | 'discouraged';
  };
  attestation?: 'none' | 'indirect' | 'direct' | 'enterprise';
  extensions?: Record<string, unknown>;
}

/**
 * Public key credential request options returned to client.
 */
export interface PublicKeyCredentialRequestOptionsJSON {
  challenge: string; // Base64URL
  timeout?: number;
  rpId?: string;
  allowCredentials?: Array<{
    id: string;
    type: 'public-key';
    transports?: AuthenticatorTransport[];
  }>;
  userVerification?: 'required' | 'preferred' | 'discouraged';
  extensions?: Record<string, unknown>;
}

// ============================================================================
// Request DTOs
// ============================================================================

export class StartRegistrationRequestDto {
  @ApiProperty({ description: 'Principal ID' })
  @IsString()
  @IsNotEmpty()
  principalId: string;

  @ApiProperty({ description: 'User name (email or username)' })
  @IsString()
  @IsNotEmpty()
  userName: string;

  @ApiPropertyOptional({ description: 'Display name' })
  @IsOptional()
  @IsString()
  displayName?: string;
}

export class VerifyRegistrationRequestDto {
  @ApiProperty({ description: 'Principal ID' })
  @IsString()
  @IsNotEmpty()
  principalId: string;

  @ApiProperty({
    description: 'Registration response from authenticator (RegistrationResponseJSON)',
  })
  @IsObject()
  response: RegistrationResponseJSON;
}

export class StartAuthenticationRequestDto {
  @ApiProperty({ description: 'Principal ID' })
  @IsString()
  @IsNotEmpty()
  principalId: string;
}

export class VerifyAuthenticationRequestDto {
  @ApiProperty({ description: 'Principal ID' })
  @IsString()
  @IsNotEmpty()
  principalId: string;

  @ApiProperty({
    description: 'Authentication response from authenticator (AuthenticationResponseJSON)',
  })
  @IsObject()
  response: AuthenticationResponseJSON;
}

// ============================================================================
// Response DTOs
// ============================================================================

export class RegistrationOptionsResponseDto {
  @ApiProperty({ description: 'WebAuthn registration options' })
  options: PublicKeyCredentialCreationOptionsJSON;
}

export class AuthenticationOptionsResponseDto {
  @ApiProperty({ description: 'WebAuthn authentication options' })
  options: PublicKeyCredentialRequestOptionsJSON;
}

export class VerificationResultDto {
  @ApiProperty({ description: 'Whether verification succeeded' })
  verified: boolean;

  @ApiPropertyOptional({ description: 'Credential ID' })
  credentialId?: string;

  @ApiPropertyOptional({ description: 'Error message if failed' })
  error?: string;
}
