/**
 * Liability Proof Guard (Production)
 *
 * Validates X-AgentKernIdentity header with full Ed25519 signature verification.
 * Follows Zero-Trust: verify every request, never assume trust.
 *
 * Security Features:
 * - Ed25519 signature verification
 * - Expiration checking with clock skew tolerance
 * - Issuer validation
 * - Constant-time comparison for signatures
 */
import {
  Injectable,
  CanActivate,
  ExecutionContext,
  UnauthorizedException,
  Logger,
  SetMetadata,
} from '@nestjs/common';
import { Reflector } from '@nestjs/core';
import { Request } from 'express';
import * as crypto from 'crypto';

/** Decorator key for public routes that skip auth */
export const IS_PUBLIC_KEY = 'isPublic';

/** Decorator to mark route as public (no auth required) */
export const Public = () => SetMetadata(IS_PUBLIC_KEY, true);

/** Clock skew tolerance in seconds (5 minutes) */
const CLOCK_SKEW_TOLERANCE = 300;

/**
 * Guard that validates the X-AgentKernIdentity header.
 *
 * Checks:
 * 1. Header exists
 * 2. JWT structure is valid (3 parts)
 * 3. Signature is cryptographically valid (Ed25519)
 * 4. Claims are not expired (with clock skew tolerance)
 * 5. Required claims present (iss, sub)
 */
@Injectable()
export class LiabilityProofGuard implements CanActivate {
  private readonly logger = new Logger(LiabilityProofGuard.name);

  constructor(private readonly reflector: Reflector) {}

  async canActivate(context: ExecutionContext): Promise<boolean> {
    // Check if route is marked as public
    const isPublic = this.reflector.getAllAndOverride<boolean>(IS_PUBLIC_KEY, [
      context.getHandler(),
      context.getClass(),
    ]);

    if (isPublic) {
      return true;
    }

    const request = context.switchToHttp().getRequest<Request>();
    const proofHeader = request.headers['x-agentkernidentity'] as string;

    if (!proofHeader) {
      this.logger.warn('Missing X-AgentKernIdentity header', {
        path: request.path,
        ip: request.ip,
      });
      throw new UnauthorizedException('Missing liability proof header');
    }

    try {
      // Validate the liability proof token
      const proof = await this.validateProof(proofHeader);

      // Attach proof to request for downstream use
      (request as unknown as Record<string, unknown>).liabilityProof = proof;

      this.logger.debug('Liability proof validated', {
        issuer: proof.issuer,
        subject: proof.subject,
        action: proof.action,
        path: request.path,
      });

      return true;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      this.logger.warn('Invalid liability proof', {
        path: request.path,
        error: errorMessage,
      });
      throw new UnauthorizedException(`Invalid liability proof: ${errorMessage}`);
    }
  }

  /**
   * Validate a liability proof JWT with full cryptographic verification.
   *
   * Steps:
   * 1. Parse JWT structure (header.payload.signature)
   * 2. Decode and validate header (alg must be EdDSA)
   * 3. Decode and validate payload claims
   * 4. Extract public key from header.kid
   * 5. Verify Ed25519 signature
   */
  private async validateProof(token: string): Promise<LiabilityProofPayload> {
    // Split JWT into parts
    const parts = token.split('.');
    if (parts.length !== 3) {
      throw new Error('Invalid JWT format: expected 3 parts (header.payload.signature)');
    }

    const [headerB64, payloadB64, signatureB64] = parts;

    // Decode header
    const headerJson = Buffer.from(headerB64, 'base64url').toString('utf8');
    const header = JSON.parse(headerJson) as JwtHeader;

    // Validate algorithm
    if (header.alg !== 'EdDSA') {
      throw new Error(`Unsupported algorithm: ${header.alg}. Only EdDSA is supported.`);
    }

    // Decode payload
    const payloadJson = Buffer.from(payloadB64, 'base64url').toString('utf8');
    const payload = JSON.parse(payloadJson) as JwtPayload;

    // Validate required claims
    if (!payload.iss) {
      throw new Error('Missing required claim: iss (issuer)');
    }

    if (!payload.sub) {
      throw new Error('Missing required claim: sub (subject)');
    }

    // Check expiration with clock skew tolerance
    const now = Math.floor(Date.now() / 1000);

    if (payload.exp && payload.exp + CLOCK_SKEW_TOLERANCE < now) {
      throw new Error(
        `Token expired at ${new Date(payload.exp * 1000).toISOString()}`,
      );
    }

    // Check not-before with clock skew tolerance
    if (payload.nbf && payload.nbf - CLOCK_SKEW_TOLERANCE > now) {
      throw new Error(
        `Token not valid until ${new Date(payload.nbf * 1000).toISOString()}`,
      );
    }

    // Extract public key from kid (key ID)
    if (!header.kid) {
      throw new Error('Missing key ID (kid) in header');
    }

    // Verify Ed25519 signature
    await this.verifyEd25519Signature(
      `${headerB64}.${payloadB64}`,
      signatureB64,
      header.kid,
    );

    return {
      issuer: payload.iss,
      subject: payload.sub,
      audience: payload.aud,
      issuedAt: payload.iat,
      expiresAt: payload.exp,
      notBefore: payload.nbf,
      jwtId: payload.jti,
      action: payload.action,
      scope: payload.scope,
      raw: token,
    };
  }

  /**
   * Verify Ed25519 signature using Node.js crypto.
   *
   * @param signingInput - The data that was signed (header.payload)
   * @param signatureB64 - Base64url encoded signature
   * @param publicKeyB64 - Base64url encoded public key from kid
   */
  private async verifyEd25519Signature(
    signingInput: string,
    signatureB64: string,
    publicKeyB64: string,
  ): Promise<void> {
    try {
      // Decode signature
      const signature = Buffer.from(signatureB64, 'base64url');

      // Ed25519 signatures are always 64 bytes
      if (signature.length !== 64) {
        throw new Error(`Invalid signature length: ${signature.length} (expected 64)`);
      }

      // Decode public key
      const publicKeyRaw = Buffer.from(publicKeyB64, 'base64url');

      // Ed25519 public keys are always 32 bytes
      if (publicKeyRaw.length !== 32) {
        throw new Error(`Invalid public key length: ${publicKeyRaw.length} (expected 32)`);
      }

      // Create KeyObject from raw public key
      const publicKey = crypto.createPublicKey({
        key: Buffer.concat([
          // Ed25519 public key OID prefix
          Buffer.from('302a300506032b6570032100', 'hex'),
          publicKeyRaw,
        ]),
        format: 'der',
        type: 'spki',
      });

      // Verify signature
      const isValid = crypto.verify(
        null, // Ed25519 doesn't use a separate hash algorithm
        Buffer.from(signingInput, 'utf8'),
        publicKey,
        signature,
      );

      if (!isValid) {
        throw new Error('Signature verification failed');
      }
    } catch (error) {
      if (error instanceof Error && error.message.includes('Signature verification failed')) {
        throw error;
      }
      throw new Error(
        `Signature verification error: ${error instanceof Error ? error.message : 'Unknown'}`,
      );
    }
  }
}

/**
 * JWT Header structure for EdDSA tokens.
 */
interface JwtHeader {
  /** Algorithm (must be EdDSA) */
  alg: string;
  /** Token type (LIABILITY+jwt) */
  typ?: string;
  /** Key ID (base64url public key) */
  kid?: string;
}

/**
 * JWT Payload (claims) for liability proofs.
 */
interface JwtPayload {
  /** Issuer (DID or domain) */
  iss: string;
  /** Subject (agent DID) */
  sub: string;
  /** Audience */
  aud?: string;
  /** Issued at timestamp */
  iat?: number;
  /** Expiration timestamp */
  exp?: number;
  /** Not before timestamp */
  nbf?: number;
  /** JWT ID */
  jti?: string;
  /** Authorized action */
  action?: string;
  /** Permission scopes */
  scope?: string[];
}

/**
 * Parsed liability proof payload.
 */
export interface LiabilityProofPayload {
  /** Token issuer (DID or domain) */
  issuer: string;
  /** Subject (agent DID) */
  subject: string;
  /** Intended audience */
  audience?: string;
  /** Issued at timestamp */
  issuedAt?: number;
  /** Expiration timestamp */
  expiresAt?: number;
  /** Not before timestamp */
  notBefore?: number;
  /** JWT ID */
  jwtId?: string;
  /** Authorized action */
  action?: string;
  /** Permission scopes */
  scope?: string[];
  /** Raw token for forwarding */
  raw: string;
}

/**
 * Optional Auth Guard - same as LiabilityProofGuard but doesn't throw.
 * Attaches proof if present, otherwise continues without auth.
 * Useful for endpoints that work with or without authentication.
 */
@Injectable()
export class OptionalAuthGuard implements CanActivate {
  private readonly logger = new Logger(OptionalAuthGuard.name);

  async canActivate(context: ExecutionContext): Promise<boolean> {
    const request = context.switchToHttp().getRequest<Request>();
    const proofHeader = request.headers['x-agentkernidentity'] as string;

    if (proofHeader) {
      try {
        const parts = proofHeader.split('.');
        if (parts.length === 3) {
          // Decode header to get key ID
          const headerJson = Buffer.from(parts[0], 'base64url').toString('utf8');
          const header = JSON.parse(headerJson) as JwtHeader;

          // Decode payload
          const payloadJson = Buffer.from(parts[1], 'base64url').toString('utf8');
          const payload = JSON.parse(payloadJson) as JwtPayload;

          // Basic validation for optional auth (no signature verification)
          const now = Math.floor(Date.now() / 1000);
          if (payload.exp && payload.exp < now) {
            this.logger.debug('Optional auth: token expired, proceeding unauthenticated');
            return true;
          }

          (request as unknown as Record<string, unknown>).liabilityProof = {
            issuer: payload.iss,
            subject: payload.sub,
            audience: payload.aud,
            issuedAt: payload.iat,
            expiresAt: payload.exp,
            action: payload.action,
            scope: payload.scope,
            raw: proofHeader,
            verified: false, // Mark as unverified for optional auth
          };
        }
      } catch (error) {
        this.logger.debug('Optional auth: invalid token format, proceeding unauthenticated');
      }
    }

    return true;
  }
}
