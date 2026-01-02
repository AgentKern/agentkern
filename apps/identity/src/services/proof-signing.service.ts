/**
 * AgentKernIdentity - Proof Signing Service
 *
 * Creates signed Liability Proofs using ES256 (ECDSA P-256).
 * In production, signing happens on the client device via WebAuthn.
 * This service is for testing and demonstration purposes.
 */

import { Injectable, Logger } from '@nestjs/common';
import * as jose from 'jose';
import {
  LiabilityProof,
  LiabilityProofPayload,
  Principal,
  Agent,
  Intent,
  Constraints,
  createProofPayload,
} from '../domain/liability-proof.entity';

export interface CreateProofRequest {
  principal: Principal;
  agent: Agent;
  intent: Intent;
  constraints?: Constraints;
  expiresInSeconds?: number;
  privateKey: string; // PEM format - in production this never leaves device
}

@Injectable()
export class ProofSigningService {
  private readonly logger = new Logger(ProofSigningService.name);

  /**
   * Create and sign a Liability Proof
   *
   * NOTE: In production, signing happens on the CLIENT device via WebAuthn.
   * This method is for server-side testing and development only.
   */
  async createSignedProof(
    request: CreateProofRequest,
  ): Promise<LiabilityProof> {
    // Create the unsigned payload
    const payload = createProofPayload(
      request.principal,
      request.agent,
      request.intent,
      {
        constraints: request.constraints,
        expiresInSeconds: request.expiresInSeconds,
      },
    );

    this.logger.log(
      `Creating proof: ${payload.proofId} for principal: ${request.principal.id}`,
    );

    // Sign the payload
    const signature = await this.signPayload(payload, request.privateKey);

    return {
      version: 'v1',
      payload,
      signature,
    };
  }

  /**
   * Sign a payload using ES256 (ECDSA with P-256 curve)
   */
  private async signPayload(
    payload: LiabilityProofPayload,
    privateKeyPem: string,
  ): Promise<string> {
    try {
      // Import the private key
      const privateKey = await jose.importPKCS8(privateKeyPem, 'ES256');

      // Convert payload to bytes
      const payloadJson = JSON.stringify(payload);

      // Create a compact JWS
      const jws = await new jose.CompactSign(
        new TextEncoder().encode(payloadJson),
      )
        .setProtectedHeader({ alg: 'ES256' })
        .sign(privateKey);

      // Extract just the signature part
      const parts = jws.split('.');
      return parts[2]; // Return only the signature
    } catch (error) {
      this.logger.error(`Signing failed: ${error}`);
      throw new Error('Failed to sign proof');
    }
  }

  /**
   * Generate a new ES256 key pair for testing
   */
  async generateKeyPair(): Promise<{ publicKey: string; privateKey: string }> {
    const { publicKey, privateKey } = await jose.generateKeyPair('ES256');

    const publicKeyPem = await jose.exportSPKI(publicKey);
    const privateKeyPem = await jose.exportPKCS8(privateKey);

    return {
      publicKey: publicKeyPem,
      privateKey: privateKeyPem,
    };
  }
}
