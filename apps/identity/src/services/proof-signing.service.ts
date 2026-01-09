/**
 * AgentKernIdentity - Proof Signing Service
 *
 * Creates signed Liability Proofs using ES256 (ECDSA P-256).
 * In production, signing happens on the client device via WebAuthn.
 * This service is for testing and demonstration purposes.
 */

import { Injectable, Logger, OnModuleInit } from '@nestjs/common';
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
export class ProofSigningService implements OnModuleInit {
  private readonly logger = new Logger(ProofSigningService.name);
  private bridge: any;
  private bridgeLoaded = false;

  async onModuleInit() {
    try {
      // Dynamic import of the N-API bridge
      const bridgePath =
        process.env.BRIDGE_PATH ||
        '../../../../packages/foundation/bridge/index.node';
      
      // Check if file exists roughly
      const fs = require('fs');
      if (fs.existsSync(bridgePath)) {
          this.bridge = require(bridgePath);
          this.bridgeLoaded = true;
          this.logger.log('Native bridge loaded for PQC signing');
      } else {
        this.logger.warn(`Native bridge not found at ${bridgePath}. PQC signing unavailable.`);
      }
    } catch (error) {
      this.logger.warn(`Failed to load native bridge: ${error.message}`);
    }
  }

  /**
   * Create and sign a Liability Proof
   *
   * NOTE: In production, signing happens on the CLIENT device via WebAuthn.
   * This method is for server-side testing and development only.
   */
  async createSignedProof(
    request: CreateProofRequest & { algorithm?: 'ES256' | 'Hybrid-PQC' },
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
      `Creating proof: ${payload.proofId} for principal: ${request.principal.id} [${request.algorithm || 'ES256'}]`,
    );

    // Sign the payload
    const signature = await this.signPayload(payload, request.privateKey, request.algorithm);

    return {
      version: 'v1',
      payload,
      signature,
    };
  }

  /**
   * Sign a payload using ES256 or Hybrid-PQC
   */
  private async signPayload(
    payload: LiabilityProofPayload,
    privateKeyPem: string,
    algorithm: 'ES256' | 'Hybrid-PQC' = 'ES256',
  ): Promise<string> {
    try {
      // Import the private key
      const privateKey = await jose.importPKCS8(privateKeyPem, 'ES256');

      // Convert payload to bytes
      const payloadJson = JSON.stringify(payload);

      // Create a compact JWS (Classical Part)
      const jws = await new jose.CompactSign(
        new TextEncoder().encode(payloadJson),
      )
        .setProtectedHeader({ alg: 'ES256' })
        .sign(privateKey);

      // Extract just the signature part
      const parts = jws.split('.');
      let signature = parts[2];

      // If Hybrid, append PQC signature
      if (algorithm === 'Hybrid-PQC') {
         if (!this.bridgeLoaded) {
             throw new Error('Bridge not loaded, cannot sign Hybrid-PQC');
         }
         const pqcSig = this.bridge.cryptoSignHybrid(payloadJson, privateKeyPem);
         // Format: <ec_sig>~<pqc_sig> (using ~ as separator for simplicity in tests)
         signature = `${signature}~${pqcSig}`;
      }

      return signature;
    } catch (error) {
      this.logger.error(`Signing failed: ${error}`);
      throw new Error(`Failed to sign proof: ${error.message}`);
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
