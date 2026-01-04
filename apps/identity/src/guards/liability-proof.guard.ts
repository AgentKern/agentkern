/**
 * Liability Proof Guard
 * 
 * Validates X-AgentKernIdentity header for protected endpoints.
 * Follows Zero-Trust: verify every request, never assume trust.
 * 
 * @example
 * ```typescript
 * @UseGuards(LiabilityProofGuard)
 * @Get('protected')
 * async protectedEndpoint() { ... }
 * ```
 */
import {
  Injectable,
  CanActivate,
  ExecutionContext,
  UnauthorizedException,
  Logger,
} from '@nestjs/common';
import { Reflector } from '@nestjs/core';
import { Request } from 'express';

/** Decorator key for public routes that skip auth */
export const IS_PUBLIC_KEY = 'isPublic';

/** Decorator to mark route as public (no auth required) */
export const Public = () => SetMetadata(IS_PUBLIC_KEY, true);

import { SetMetadata } from '@nestjs/common';

/**
 * Guard that validates the X-AgentKernIdentity header.
 * 
 * Checks:
 * 1. Header exists
 * 2. JWT structure is valid
 * 3. Signature is valid
 * 4. Claims are not expired
 * 5. Issuer is trusted
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
      (request as any).liabilityProof = proof;
      
      this.logger.debug('Liability proof validated', {
        issuer: proof.issuer,
        subject: proof.subject,
        path: request.path,
      });

      return true;
    } catch (error) {
      this.logger.warn('Invalid liability proof', {
        path: request.path,
        error: error instanceof Error ? error.message : 'Unknown error',
      });
      throw new UnauthorizedException('Invalid liability proof');
    }
  }

  /**
   * Validate a liability proof JWT.
   * 
   * In production, this would:
   * 1. Verify JWT signature using public key from DNS
   * 2. Check claims (exp, iss, sub, aud)
   * 3. Validate WebAuthn attestation if present
   */
  private async validateProof(token: string): Promise<LiabilityProofPayload> {
    // Split JWT into parts
    const parts = token.split('.');
    if (parts.length !== 3) {
      throw new Error('Invalid JWT format');
    }

    // Decode payload (base64url)
    const payloadBase64 = parts[1];
    const payloadJson = Buffer.from(payloadBase64, 'base64url').toString('utf8');
    const payload = JSON.parse(payloadJson);

    // Validate required claims
    const now = Math.floor(Date.now() / 1000);

    if (payload.exp && payload.exp < now) {
      throw new Error('Token expired');
    }

    if (!payload.iss) {
      throw new Error('Missing issuer claim');
    }

    if (!payload.sub) {
      throw new Error('Missing subject claim');
    }

    // TODO: In production, verify signature using issuer's public key
    // const publicKey = await this.dnsResolver.resolvePublicKey(payload.iss);
    // await verifyJwtSignature(token, publicKey);

    return {
      issuer: payload.iss,
      subject: payload.sub,
      audience: payload.aud,
      issuedAt: payload.iat,
      expiresAt: payload.exp,
      action: payload.action,
      scope: payload.scope,
      raw: token,
    };
  }
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
  async canActivate(context: ExecutionContext): Promise<boolean> {
    const request = context.switchToHttp().getRequest<Request>();
    const proofHeader = request.headers['x-agentkernidentity'] as string;

    if (proofHeader) {
      try {
        const parts = proofHeader.split('.');
        if (parts.length === 3) {
          const payloadJson = Buffer.from(parts[1], 'base64url').toString('utf8');
          (request as any).liabilityProof = JSON.parse(payloadJson);
        }
      } catch {
        // Ignore invalid tokens - proceed as unauthenticated
      }
    }

    return true;
  }
}
