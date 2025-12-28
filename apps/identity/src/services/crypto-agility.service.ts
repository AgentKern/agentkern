/**
 * AgentKernIdentity - Crypto-Agility Service
 *
 * Provides algorithm-agnostic cryptographic operations to prepare for
 * quantum-safe cryptography migration. Supports hybrid signatures
 * (classical + post-quantum) for gradual transition.
 *
 * Supported Algorithm Families:
 * - Classical: ES256 (ECDSA P-256), Ed25519
 * - Post-Quantum (future): CRYSTALS-Dilithium, SPHINCS+
 * - Hybrid: Classical + PQC combined signatures
 */

import { Injectable, Logger, OnModuleInit } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';
import * as jose from 'jose';

// Cryptographic algorithm identifiers
export enum CryptoAlgorithm {
  // Classical algorithms (quantum-vulnerable)
  ES256 = 'ES256', // ECDSA with P-256
  ES384 = 'ES384', // ECDSA with P-384
  ES512 = 'ES512', // ECDSA with P-521
  Ed25519 = 'EdDSA', // Edwards-curve Digital Signature Algorithm

  // Post-quantum algorithms (future support)
  DILITHIUM2 = 'DILITHIUM2', // CRYSTALS-Dilithium (NIST Level 2)
  DILITHIUM3 = 'DILITHIUM3', // CRYSTALS-Dilithium (NIST Level 3)
  DILITHIUM5 = 'DILITHIUM5', // CRYSTALS-Dilithium (NIST Level 5)
  SPHINCS_SHA256_128F = 'SPHINCS+-SHA256-128f', // SPHINCS+ (fast)
  SPHINCS_SHA256_256F = 'SPHINCS+-SHA256-256f', // SPHINCS+ (high security)

  // Hybrid modes
  HYBRID_ES256_DILITHIUM2 = 'HYBRID-ES256-DILITHIUM2',
  HYBRID_Ed25519_DILITHIUM3 = 'HYBRID-Ed25519-DILITHIUM3',
}

export interface CryptoKeyPair {
  algorithm: CryptoAlgorithm;
  publicKey: string; // PEM format
  privateKey: string; // PEM format
  keyId: string;
  createdAt: string;
  expiresAt?: string;
}

export interface SignatureResult {
  signature: string;
  algorithm: CryptoAlgorithm;
  keyId: string;
  timestamp: string;
}

export interface HybridSignature {
  classical: SignatureResult;
  postQuantum?: SignatureResult;
  combined: string; // Combined signature for header
}

export interface VerificationResult {
  valid: boolean;
  algorithm: CryptoAlgorithm;
  keyId?: string;
  error?: string;
}

export interface CryptoProvider {
  algorithm: CryptoAlgorithm;
  sign(payload: Uint8Array, privateKey: string): Promise<Uint8Array>;
  verify(payload: Uint8Array, signature: Uint8Array, publicKey: string): Promise<boolean>;
  generateKeyPair(): Promise<CryptoKeyPair>;
}

@Injectable()
export class CryptoAgilityService implements OnModuleInit {
  private readonly logger = new Logger(CryptoAgilityService.name);

  // Registered crypto providers
  private providers: Map<CryptoAlgorithm, CryptoProvider> = new Map();

  // Default algorithms (can be changed via config)
  private defaultSigningAlgorithm: CryptoAlgorithm = CryptoAlgorithm.ES256;
  private hybridModeEnabled = false;
  private pqcAvailable = false;

  constructor(private readonly configService: ConfigService) {}

  async onModuleInit(): Promise<void> {
    // Register built-in providers
    await this.registerBuiltInProviders();

    // Check for PQC library availability
    await this.checkPQCAvailability();

    // Load configuration
    this.loadConfiguration();

    this.logger.log(`🔐 Crypto-Agility Service initialized`);
    this.logger.log(`   Default algorithm: ${this.defaultSigningAlgorithm}`);
    this.logger.log(`   Hybrid mode: ${this.hybridModeEnabled ? 'enabled' : 'disabled'}`);
    this.logger.log(`   PQC available: ${this.pqcAvailable ? 'yes' : 'no'}`);
  }

  /**
   * Sign payload with the default algorithm (or hybrid if enabled)
   */
  async sign(
    payload: Uint8Array,
    privateKey: string,
    keyId: string,
  ): Promise<SignatureResult | HybridSignature> {
    if (this.hybridModeEnabled && this.pqcAvailable) {
      return this.hybridSign(payload, privateKey, keyId);
    }

    return this.singleSign(payload, privateKey, keyId, this.defaultSigningAlgorithm);
  }

  /**
   * Sign with a specific algorithm
   */
  async signWithAlgorithm(
    payload: Uint8Array,
    privateKey: string,
    keyId: string,
    algorithm: CryptoAlgorithm,
  ): Promise<SignatureResult> {
    return this.singleSign(payload, privateKey, keyId, algorithm);
  }

  /**
   * Verify a signature (auto-detects algorithm if possible)
   */
  async verify(
    payload: Uint8Array,
    signature: string,
    publicKey: string,
    algorithm?: CryptoAlgorithm,
  ): Promise<VerificationResult> {
    const algo = algorithm || this.defaultSigningAlgorithm;
    const provider = this.providers.get(algo);

    if (!provider) {
      return {
        valid: false,
        algorithm: algo,
        error: `Unsupported algorithm: ${algo}`,
      };
    }

    try {
      const signatureBytes = Buffer.from(signature, 'base64url');
      const valid = await provider.verify(payload, signatureBytes, publicKey);

      return {
        valid,
        algorithm: algo,
      };
    } catch (error) {
      return {
        valid: false,
        algorithm: algo,
        error: error instanceof Error ? error.message : 'Verification failed',
      };
    }
  }

  /**
   * Verify a hybrid signature (both classical and PQC must pass)
   */
  async verifyHybrid(
    payload: Uint8Array,
    hybridSignature: HybridSignature,
    classicalPublicKey: string,
    pqcPublicKey?: string,
  ): Promise<VerificationResult> {
    // Verify classical signature
    const classicalResult = await this.verify(
      payload,
      hybridSignature.classical.signature,
      classicalPublicKey,
      hybridSignature.classical.algorithm,
    );

    if (!classicalResult.valid) {
      return {
        valid: false,
        algorithm: hybridSignature.classical.algorithm,
        error: 'Classical signature verification failed',
      };
    }

    // If PQC signature present, verify it too
    if (hybridSignature.postQuantum && pqcPublicKey) {
      const pqcResult = await this.verify(
        payload,
        hybridSignature.postQuantum.signature,
        pqcPublicKey,
        hybridSignature.postQuantum.algorithm,
      );

      if (!pqcResult.valid) {
        return {
          valid: false,
          algorithm: hybridSignature.postQuantum.algorithm,
          error: 'Post-quantum signature verification failed',
        };
      }
    }

    return {
      valid: true,
      algorithm: hybridSignature.classical.algorithm,
    };
  }

  /**
   * Generate a new key pair for the specified algorithm
   */
  async generateKeyPair(algorithm?: CryptoAlgorithm): Promise<CryptoKeyPair> {
    const algo = algorithm || this.defaultSigningAlgorithm;
    const provider = this.providers.get(algo);

    if (!provider) {
      throw new Error(`Unsupported algorithm: ${algo}`);
    }

    return provider.generateKeyPair();
  }

  /**
   * Register a custom crypto provider
   */
  registerProvider(provider: CryptoProvider): void {
    this.providers.set(provider.algorithm, provider);
    this.logger.log(`Registered crypto provider: ${provider.algorithm}`);
  }

  /**
   * Get available algorithms
   */
  getAvailableAlgorithms(): CryptoAlgorithm[] {
    return Array.from(this.providers.keys());
  }

  /**
   * Get algorithm recommendation based on security requirements
   */
  getRecommendedAlgorithm(options: {
    quantumSafe?: boolean;
    highPerformance?: boolean;
    hybrid?: boolean;
  }): CryptoAlgorithm {
    if (options.quantumSafe && this.pqcAvailable) {
      return options.highPerformance
        ? CryptoAlgorithm.DILITHIUM2
        : CryptoAlgorithm.DILITHIUM3;
    }

    if (options.hybrid && this.pqcAvailable) {
      return CryptoAlgorithm.HYBRID_ES256_DILITHIUM2;
    }

    return options.highPerformance
      ? CryptoAlgorithm.Ed25519
      : CryptoAlgorithm.ES256;
  }

  /**
   * Enable or disable hybrid mode
   */
  setHybridMode(enabled: boolean): void {
    if (enabled && !this.pqcAvailable) {
      this.logger.warn('Cannot enable hybrid mode: PQC libraries not available');
      return;
    }
    this.hybridModeEnabled = enabled;
    this.logger.log(`Hybrid mode ${enabled ? 'enabled' : 'disabled'}`);
  }

  // ============ Private Methods ============

  private async singleSign(
    payload: Uint8Array,
    privateKey: string,
    keyId: string,
    algorithm: CryptoAlgorithm,
  ): Promise<SignatureResult> {
    const provider = this.providers.get(algorithm);

    if (!provider) {
      throw new Error(`Unsupported algorithm: ${algorithm}`);
    }

    const signatureBytes = await provider.sign(payload, privateKey);

    return {
      signature: Buffer.from(signatureBytes).toString('base64url'),
      algorithm,
      keyId,
      timestamp: new Date().toISOString(),
    };
  }

  private async hybridSign(
    payload: Uint8Array,
    privateKey: string,
    keyId: string,
  ): Promise<HybridSignature> {
    // Sign with classical algorithm
    const classical = await this.singleSign(
      payload,
      privateKey,
      keyId,
      CryptoAlgorithm.ES256,
    );

    // Sign with PQC algorithm if available
    let postQuantum: SignatureResult | undefined;
    if (this.providers.has(CryptoAlgorithm.DILITHIUM2)) {
      // Note: In real implementation, PQC would use a separate key
      postQuantum = await this.singleSign(
        payload,
        privateKey, // Would be different PQC key
        keyId,
        CryptoAlgorithm.DILITHIUM2,
      );
    }

    // Combine signatures
    const combined = postQuantum
      ? `${classical.signature}.${postQuantum.signature}`
      : classical.signature;

    return {
      classical,
      postQuantum,
      combined,
    };
  }

  private async registerBuiltInProviders(): Promise<void> {
    // ES256 Provider (ECDSA with P-256)
    this.registerProvider({
      algorithm: CryptoAlgorithm.ES256,
      async sign(payload: Uint8Array, privateKeyPem: string): Promise<Uint8Array> {
        const privateKey = await jose.importPKCS8(privateKeyPem, 'ES256');
        const jws = await new jose.CompactSign(payload)
          .setProtectedHeader({ alg: 'ES256' })
          .sign(privateKey);
        const signaturePart = jws.split('.')[2];
        return Buffer.from(signaturePart, 'base64url');
      },
      async verify(
        payload: Uint8Array,
        signature: Uint8Array,
        publicKeyPem: string,
      ): Promise<boolean> {
        try {
          const publicKey = await jose.importSPKI(publicKeyPem, 'ES256');
          const payloadB64 = Buffer.from(payload).toString('base64url');
          const signatureB64 = Buffer.from(signature).toString('base64url');
          await jose.compactVerify(
            `eyJhbGciOiJFUzI1NiJ9.${payloadB64}.${signatureB64}`,
            publicKey,
          );
          return true;
        } catch {
          return false;
        }
      },
      async generateKeyPair(): Promise<CryptoKeyPair> {
        const { publicKey, privateKey } = await jose.generateKeyPair('ES256');
        return {
          algorithm: CryptoAlgorithm.ES256,
          publicKey: await jose.exportSPKI(publicKey),
          privateKey: await jose.exportPKCS8(privateKey),
          keyId: crypto.randomUUID(),
          createdAt: new Date().toISOString(),
        };
      },
    });

    // Ed25519 Provider
    this.registerProvider({
      algorithm: CryptoAlgorithm.Ed25519,
      async sign(payload: Uint8Array, privateKeyPem: string): Promise<Uint8Array> {
        const privateKey = await jose.importPKCS8(privateKeyPem, 'EdDSA');
        const jws = await new jose.CompactSign(payload)
          .setProtectedHeader({ alg: 'EdDSA' })
          .sign(privateKey);
        const signaturePart = jws.split('.')[2];
        return Buffer.from(signaturePart, 'base64url');
      },
      async verify(
        payload: Uint8Array,
        signature: Uint8Array,
        publicKeyPem: string,
      ): Promise<boolean> {
        try {
          const publicKey = await jose.importSPKI(publicKeyPem, 'EdDSA');
          const payloadB64 = Buffer.from(payload).toString('base64url');
          const signatureB64 = Buffer.from(signature).toString('base64url');
          await jose.compactVerify(
            `eyJhbGciOiJFZERTQSJ9.${payloadB64}.${signatureB64}`,
            publicKey,
          );
          return true;
        } catch {
          return false;
        }
      },
      async generateKeyPair(): Promise<CryptoKeyPair> {
        const { publicKey, privateKey } = await jose.generateKeyPair('EdDSA');
        return {
          algorithm: CryptoAlgorithm.Ed25519,
          publicKey: await jose.exportSPKI(publicKey),
          privateKey: await jose.exportPKCS8(privateKey),
          keyId: crypto.randomUUID(),
          createdAt: new Date().toISOString(),
        };
      },
    });

    // CRYSTALS-Dilithium2 Provider (Placeholder for PQC)
    this.registerProvider({
      algorithm: CryptoAlgorithm.DILITHIUM2,
      async sign(payload: Uint8Array): Promise<Uint8Array> {
        // In reality, this would use liboqs-node
        // We simulate the larger PQC signature size (approx 2.4KB for Dilithium2)
        const header = Buffer.from('DILITHIUM2-SIG:');
        const padding = crypto.getRandomValues(new Uint8Array(2400));
        return Buffer.concat([header, payload, padding]);
      },
      async verify(payload: Uint8Array, signature: Uint8Array): Promise<boolean> {
        const header = Buffer.from('DILITHIUM2-SIG:');
        if (signature.length < header.length + payload.length) return false;
        const msgPart = signature.slice(header.length, header.length + payload.length);
        return Buffer.compare(payload, msgPart) === 0;
      },
      async generateKeyPair(): Promise<CryptoKeyPair> {
        return {
          algorithm: CryptoAlgorithm.DILITHIUM2,
          publicKey: `PQC-PUB-${crypto.randomUUID()}`,
          privateKey: `PQC-PRIV-${crypto.randomUUID()}`,
          keyId: crypto.randomUUID(),
          createdAt: new Date().toISOString(),
        };
      },
    });
  }

  private async checkPQCAvailability(): Promise<void> {
    try {
      // In a real environment, we'd check for liboqs-node
      // For this implementation, we enable the built-in PQC simulation
      this.pqcAvailable = true;
      this.logger.log('✨ Post-Quantum Cryptography (PQC) simulation enabled');
    } catch {
      this.pqcAvailable = false;
    }
  }

  private loadConfiguration(): void {
    const algorithm = this.configService.get<string>('CRYPTO_ALGORITHM');
    if (algorithm && Object.values(CryptoAlgorithm).includes(algorithm as CryptoAlgorithm)) {
      this.defaultSigningAlgorithm = algorithm as CryptoAlgorithm;
    }

    const hybridMode = this.configService.get<string>('CRYPTO_HYBRID_MODE');
    if (hybridMode === 'true' || hybridMode === '1') {
      this.hybridModeEnabled = true;
    }
  }
}
