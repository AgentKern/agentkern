/**
 * AgentKernIdentity - Proof Verification Service
 *
 * Verifies Liability Proofs using real cryptographic verification.
 * Production-ready with TypeORM persistence for verification keys.
 */

import { Injectable, Logger, OnModuleInit } from '@nestjs/common';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import * as jose from 'jose';
import {
  LiabilityProof,
  LiabilityProofPayload,
  parseProofHeader,
} from '../domain/liability-proof.entity';
import { VerificationKeyEntity } from '../entities/verification-key.entity';

export interface VerificationResult {
  valid: boolean;
  proofId?: string;
  principalId?: string;
  agentId?: string;
  intent?: {
    action: string;
    target: string;
  };
  liabilityAcceptedBy?: string;
  errors?: string[];
}

export interface PublicKeyInfo {
  principalId: string;
  credentialId: string;
  publicKey: string;
  algorithm: string;
}

@Injectable()
export class ProofVerificationService implements OnModuleInit {
  private readonly logger = new Logger(ProofVerificationService.name);

  constructor(
    @InjectRepository(VerificationKeyEntity)
    private readonly keyRepository: Repository<VerificationKeyEntity>,
  ) {}

  async onModuleInit(): Promise<void> {
    const count = await this.keyRepository.count({ where: { active: true } });
    this.logger.log(
      `🔑 Proof verification initialized with ${count} active keys`,
    );
  }

  /**
   * Verify a Liability Proof from X-AgentKernIdentity header
   */
  async verifyProof(header: string): Promise<VerificationResult> {
    const errors: string[] = [];

    // Step 1: Parse the header
    const proof = parseProofHeader(header);
    if (!proof) {
      return { valid: false, errors: ['Invalid proof format'] };
    }

    this.logger.log(`Verifying proof: ${proof.payload.proofId}`);

    // Step 2: Check expiration
    const now = new Date();
    const expiresAt = new Date(proof.payload.expiresAt);
    if (expiresAt < now) {
      errors.push('Proof has expired');
    }

    // Step 3: Check issuedAt is not in the future
    const issuedAt = new Date(proof.payload.issuedAt);
    if (issuedAt > now) {
      errors.push('Proof issuedAt is in the future');
    }

    // Step 4: Verify signature
    const signatureValid = await this.verifySignature(proof);
    if (!signatureValid) {
      errors.push('Invalid signature');
    }

    // Step 5: Validate constraints
    const constraintErrors = this.validateConstraints(proof.payload);
    errors.push(...constraintErrors);

    if (errors.length > 0) {
      return {
        valid: false,
        proofId: proof.payload.proofId,
        errors,
      };
    }

    return {
      valid: true,
      proofId: proof.payload.proofId,
      principalId: proof.payload.principal.id,
      agentId: proof.payload.agent.id,
      intent: {
        action: proof.payload.intent.action,
        target: `${proof.payload.intent.target.service}${proof.payload.intent.target.endpoint}`,
      },
      liabilityAcceptedBy: proof.payload.liability.acceptedBy,
    };
  }

  /**
   * Register a public key for verification (persisted to database)
   */
  async registerPublicKey(
    keyInfo: PublicKeyInfo,
  ): Promise<VerificationKeyEntity> {
    // Check if key already exists
    const existing = await this.keyRepository.findOne({
      where: {
        principalId: keyInfo.principalId,
        credentialId: keyInfo.credentialId,
      },
    });

    if (existing) {
      // Update existing key
      existing.publicKey = keyInfo.publicKey;
      existing.algorithm = keyInfo.algorithm;
      existing.active = true;
      existing.updatedAt = new Date();
      const updated = await this.keyRepository.save(existing);
      this.logger.log(
        `Updated public key for: ${keyInfo.principalId}:${keyInfo.credentialId}`,
      );
      return updated;
    }

    // Create new key
    const entity = this.keyRepository.create({
      principalId: keyInfo.principalId,
      credentialId: keyInfo.credentialId,
      publicKey: keyInfo.publicKey,
      algorithm: keyInfo.algorithm,
      format: 'pem',
      active: true,
    });

    const saved = await this.keyRepository.save(entity);
    this.logger.log(
      `Registered public key for: ${keyInfo.principalId}:${keyInfo.credentialId}`,
    );
    return saved;
  }

  /**
   * Revoke a public key
   */
  async revokeKey(principalId: string, credentialId: string): Promise<boolean> {
    const result = await this.keyRepository.update(
      { principalId, credentialId },
      { active: false },
    );
    return (result.affected ?? 0) > 0;
  }

  /**
   * Get all active keys for a principal
   */
  async getActiveKeys(principalId: string): Promise<VerificationKeyEntity[]> {
    return this.keyRepository.find({
      where: { principalId, active: true },
      order: { createdAt: 'DESC' },
    });
  }

  /**
   * Verify the cryptographic signature using ES256 (ECDSA P-256)
   */
  private async verifySignature(proof: LiabilityProof): Promise<boolean> {
    try {
      const { principal } = proof.payload;

      // Look up key from database
      const keyEntity = await this.keyRepository.findOne({
        where: {
          principalId: principal.id,
          credentialId: principal.credentialId,
          active: true,
        },
      });

      if (!keyEntity) {
        this.logger.warn(
          `Public key not found for: ${principal.id}:${principal.credentialId}`,
        );
        return false;
      }

      // Check if key is expired
      if (keyEntity.expiresAt && keyEntity.expiresAt < new Date()) {
        this.logger.warn(
          `Public key expired for: ${principal.id}:${principal.credentialId}`,
        );
        return false;
      }

      // Import the public key
      const publicKey = await jose.importSPKI(
        keyEntity.publicKey,
        keyEntity.algorithm as 'ES256',
      );

      // Reconstruct the signed payload
      const payloadJson = JSON.stringify(proof.payload);

      // Verify the signature
      const isValid = await jose
        .compactVerify(
          `eyJhbGciOiJFUzI1NiJ9.${Buffer.from(payloadJson).toString('base64url')}.${proof.signature}`,
          publicKey,
        )
        .then(() => true)
        .catch(() => false);

      if (isValid) {
        // Update usage statistics
        await this.keyRepository.update(
          { id: keyEntity.id },
          {
            lastUsedAt: new Date(),
            usageCount: () => 'usageCount + 1',
          },
        );
      }

      return isValid;
    } catch (error) {
      this.logger.error(`Signature verification failed: ${error}`);
      return false;
    }
  }

  /**
   * Validate proof constraints
   */
  private validateConstraints(payload: LiabilityProofPayload): string[] {
    const errors: string[] = [];
    const constraints = payload.constraints;

    if (!constraints) return errors;

    // Check time-based constraints
    if (constraints.validHours) {
      const currentHour = new Date().getUTCHours();
      if (
        currentHour < constraints.validHours.start ||
        currentHour > constraints.validHours.end
      ) {
        errors.push(
          `Action not allowed outside valid hours (${constraints.validHours.start}-${constraints.validHours.end} UTC)`,
        );
      }
    }

    // GeoFence constraint logged for now (would require IP geolocation service)
    if (constraints.geoFence && constraints.geoFence.length > 0) {
      this.logger.debug(
        `GeoFence constraint: ${constraints.geoFence.join(', ')}`,
      );
    }

    return errors;
  }
}
