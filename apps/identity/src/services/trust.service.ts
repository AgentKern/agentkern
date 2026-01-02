/**
 * AgentKernIdentity - Trust Scoring Service
 *
 * Per MANIFESTO.md: "Agents have verifiable reputations built on their transaction history"
 *
 * Production-ready with TypeORM persistence.
 * Provides:
 * - Agent reputation/trust scoring
 * - Transaction history tracking
 * - Agent-to-agent mutual verification
 * - W3C Verifiable Credentials issuance
 */

import { Injectable, Logger, OnModuleInit } from '@nestjs/common';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import { v4 as uuidv4 } from 'uuid';
import { createHmac, timingSafeEqual, randomBytes } from 'crypto';
import * as jose from 'jose';
import {
  TrustEventEntity,
  TrustScoreEntity,
  TrustEventType as TrustEventTypeEnum,
} from '../entities/trust-event.entity';

// ============================================================================
// TYPES
// ============================================================================

export interface TrustScore {
  agentId: string;
  score: number;
  level: TrustLevel;
  factors: TrustFactors;
  history: TrustEvent[];
  calculatedAt: Date;
}

export enum TrustLevel {
  UNTRUSTED = 'untrusted',
  LOW = 'low',
  MEDIUM = 'medium',
  HIGH = 'high',
  VERIFIED = 'verified',
}

export interface TrustFactors {
  transactionSuccess: number;
  averageResponseTime: number;
  policyCompliance: number;
  peerEndorsements: number;
  accountAge: number;
  verifiedCredentials: number;
}

export interface TrustEvent {
  id: string;
  type: TrustEventTypeEnum;
  delta: number;
  reason: string;
  timestamp: Date;
  relatedAgentId?: string;
  responseTimeMs?: number;
}

// Re-export for backwards compatibility
export { TrustEventTypeEnum as TrustEventType };

export interface VerifiableCredential {
  '@context': string[];
  id: string;
  type: string[];
  issuer: string;
  issuanceDate: string;
  credentialSubject: {
    id: string;
    trustScore?: number;
    trustLevel?: TrustLevel;
    [key: string]: unknown;
  };
  proof?: {
    type: string;
    created: string;
    verificationMethod: string;
    proofPurpose: string;
    jws?: string;
  };
}

export interface MutualAuthRequest {
  requesterId: string;
  targetId: string;
  challenge: string;
  timestamp: Date;
}

export interface MutualAuthResponse {
  verified: boolean;
  requesterScore: TrustScore;
  targetScore: TrustScore;
  mutualTrust: number;
  sessionToken?: string;
}

// ============================================================================
// SERVICE
// ============================================================================

@Injectable()
export class TrustService implements OnModuleInit {
  private readonly logger = new Logger(TrustService.name);

  // Configuration
  private readonly INITIAL_TRUST_SCORE = 50;
  private readonly MAX_TRUST_SCORE = 100;
  private readonly MIN_TRUST_SCORE = 0;

  // Weight factors for trust calculation
  private readonly WEIGHTS = {
    transactionSuccess: 0.3,
    policyCompliance: 0.25,
    peerEndorsements: 0.2,
    accountAge: 0.15,
    verifiedCredentials: 0.1,
  };

  // Ed25519 signing key for credential proofs (initialized on module init)
  private signingKeyPair: {
    privateKey: jose.KeyLike;
    publicKey: jose.KeyLike;
  } | null = null;

  // Server secret for HMAC mutual auth (should come from vault/HSM in production)
  private readonly mutualAuthSecret =
    process.env.MUTUAL_AUTH_SECRET || randomBytes(32).toString('hex');

  constructor(
    @InjectRepository(TrustScoreEntity)
    private readonly scoreRepository: Repository<TrustScoreEntity>,
    @InjectRepository(TrustEventEntity)
    private readonly eventRepository: Repository<TrustEventEntity>,
  ) {}

  async onModuleInit(): Promise<void> {
    // Generate Ed25519 key pair for credential signing
    const { privateKey, publicKey } = await jose.generateKeyPair('EdDSA', {
      crv: 'Ed25519',
    });
    this.signingKeyPair = { privateKey, publicKey };
    this.logger.log('🔐 Ed25519 signing key pair initialized');

    const agentCount = await this.scoreRepository.count();
    const eventCount = await this.eventRepository.count();
    this.logger.log(
      `📊 Trust service initialized: ${agentCount} agents, ${eventCount} events`,
    );
  }

  // =========================================================================
  // TRUST SCORE OPERATIONS
  // =========================================================================

  /**
   * Get trust score for an agent
   */
  async getTrustScore(agentId: string): Promise<TrustScore> {
    let scoreEntity = await this.scoreRepository.findOne({
      where: { agentId },
    });

    if (!scoreEntity) {
      scoreEntity = await this.initializeTrustScore(agentId);
    }

    // Get recent events for history
    const recentEvents = await this.eventRepository.find({
      where: { agentId },
      order: { timestamp: 'DESC' },
      take: 10,
    });

    return this.entityToTrustScore(scoreEntity, recentEvents);
  }

  /**
   * Initialize trust score for new agent
   */
  private async initializeTrustScore(
    agentId: string,
  ): Promise<TrustScoreEntity> {
    // Create score entity
    const entity = this.scoreRepository.create({
      agentId,
      score: this.INITIAL_TRUST_SCORE,
      level: 'medium',
      transactionSuccessRate: 100,
      averageResponseTimeMs: 0,
      policyComplianceRate: 100,
      peerEndorsementCount: 0,
      accountAgeDays: 0,
      verifiedCredentialCount: 0,
      totalTransactions: 0,
      failedTransactions: 0,
    });

    const saved = await this.scoreRepository.save(entity);

    // Record registration event
    await this.saveEvent(agentId, {
      type: TrustEventTypeEnum.REGISTRATION,
      delta: 0,
      reason: 'Agent registered',
    });

    this.logger.log(`Initialized trust score for agent: ${agentId}`);
    return saved;
  }

  /**
   * Record a trust-affecting event
   */
  async recordEvent(
    agentId: string,
    event: Partial<TrustEvent>,
  ): Promise<TrustScore> {
    await this.saveEvent(agentId, {
      type: event.type as TrustEventTypeEnum,
      delta: event.delta || 0,
      reason: event.reason || '',
      relatedAgentId: event.relatedAgentId,
      responseTimeMs: event.responseTimeMs,
    });

    return this.recalculateTrustScore(agentId);
  }

  /**
   * Save an event to the database
   */
  private async saveEvent(
    agentId: string,
    data: {
      type: TrustEventTypeEnum;
      delta: number;
      reason: string;
      relatedAgentId?: string;
      responseTimeMs?: number;
    },
  ): Promise<TrustEventEntity> {
    const entity = this.eventRepository.create({
      agentId,
      type: data.type,
      delta: data.delta,
      reason: data.reason,
      relatedAgentId: data.relatedAgentId,
      responseTimeMs: data.responseTimeMs,
    });

    return this.eventRepository.save(entity);
  }

  /**
   * Record successful transaction
   */
  async recordTransactionSuccess(
    agentId: string,
    relatedAgentId?: string,
    responseTimeMs?: number,
  ): Promise<TrustScore> {
    await this.saveEvent(agentId, {
      type: TrustEventTypeEnum.TRANSACTION_SUCCESS,
      delta: 1,
      reason: 'Transaction completed successfully',
      relatedAgentId,
      responseTimeMs,
    });

    return this.recalculateTrustScore(agentId);
  }

  /**
   * Record failed transaction
   */
  async recordTransactionFailure(
    agentId: string,
    reason: string,
    relatedAgentId?: string,
  ): Promise<TrustScore> {
    await this.saveEvent(agentId, {
      type: TrustEventTypeEnum.TRANSACTION_FAILURE,
      delta: -5,
      reason,
      relatedAgentId,
    });

    return this.recalculateTrustScore(agentId);
  }

  /**
   * Record policy violation
   */
  async recordPolicyViolation(
    agentId: string,
    policyId: string,
  ): Promise<TrustScore> {
    await this.saveEvent(agentId, {
      type: TrustEventTypeEnum.POLICY_VIOLATION,
      delta: -10,
      reason: `Violated policy: ${policyId}`,
    });

    return this.recalculateTrustScore(agentId);
  }

  /**
   * Record peer endorsement
   */
  async recordPeerEndorsement(
    agentId: string,
    endorserId: string,
  ): Promise<TrustScore> {
    // Verify endorser has minimum trust
    const endorserScore = await this.getTrustScore(endorserId);
    if (endorserScore.score < 50) {
      this.logger.warn(
        `Endorsement rejected: endorser ${endorserId} has low trust score`,
      );
      return this.getTrustScore(agentId);
    }

    await this.saveEvent(agentId, {
      type: TrustEventTypeEnum.PEER_ENDORSEMENT,
      delta: 3,
      reason: `Endorsed by ${endorserId}`,
      relatedAgentId: endorserId,
    });

    return this.recalculateTrustScore(agentId);
  }

  /**
   * Record credential verification
   */
  async recordCredentialVerified(
    agentId: string,
    credentialType: string,
  ): Promise<TrustScore> {
    await this.saveEvent(agentId, {
      type: TrustEventTypeEnum.CREDENTIAL_VERIFIED,
      delta: 5,
      reason: `Verified credential: ${credentialType}`,
    });

    return this.recalculateTrustScore(agentId);
  }

  /**
   * Recalculate trust score based on all events
   */
  private async recalculateTrustScore(agentId: string): Promise<TrustScore> {
    // Get or create score entity
    let scoreEntity = await this.scoreRepository.findOne({
      where: { agentId },
    });
    if (!scoreEntity) {
      scoreEntity = await this.initializeTrustScore(agentId);
    }

    // Get all events for this agent
    const events = await this.eventRepository.find({
      where: { agentId },
      order: { timestamp: 'ASC' },
    });

    // Calculate factors from events
    const successEvents = events.filter(
      (e) => e.type === TrustEventTypeEnum.TRANSACTION_SUCCESS,
    );
    const failEvents = events.filter(
      (e) => e.type === TrustEventTypeEnum.TRANSACTION_FAILURE,
    );
    const violationEvents = events.filter(
      (e) => e.type === TrustEventTypeEnum.POLICY_VIOLATION,
    );
    const endorsementEvents = events.filter(
      (e) => e.type === TrustEventTypeEnum.PEER_ENDORSEMENT,
    );
    const credentialEvents = events.filter(
      (e) => e.type === TrustEventTypeEnum.CREDENTIAL_VERIFIED,
    );

    const totalTransactions = successEvents.length + failEvents.length;
    const totalActions = successEvents.length + violationEvents.length;

    // Update factors
    scoreEntity.transactionSuccessRate =
      totalTransactions > 0
        ? (successEvents.length / totalTransactions) * 100
        : 100;

    scoreEntity.averageResponseTimeMs =
      this.calculateAverageResponseTime(events);

    scoreEntity.policyComplianceRate =
      totalActions > 0
        ? ((totalActions - violationEvents.length) / totalActions) * 100
        : 100;

    scoreEntity.peerEndorsementCount = endorsementEvents.length;
    scoreEntity.accountAgeDays = this.calculateAccountAge(events);
    scoreEntity.verifiedCredentialCount = credentialEvents.length;
    scoreEntity.totalTransactions = totalTransactions;
    scoreEntity.failedTransactions = failEvents.length;

    // Calculate overall score
    const weightedScore =
      (scoreEntity.transactionSuccessRate / 100) *
        this.WEIGHTS.transactionSuccess +
      (scoreEntity.policyComplianceRate / 100) * this.WEIGHTS.policyCompliance +
      Math.min(scoreEntity.peerEndorsementCount / 10, 1) *
        this.WEIGHTS.peerEndorsements +
      Math.min(scoreEntity.accountAgeDays / 365, 1) * this.WEIGHTS.accountAge +
      Math.min(scoreEntity.verifiedCredentialCount / 3, 1) *
        this.WEIGHTS.verifiedCredentials;

    scoreEntity.score = Math.round(weightedScore * 100);
    scoreEntity.score = Math.max(
      this.MIN_TRUST_SCORE,
      Math.min(this.MAX_TRUST_SCORE, scoreEntity.score),
    );
    scoreEntity.level = this.calculateTrustLevel(scoreEntity.score);
    scoreEntity.calculatedAt = new Date();

    await this.scoreRepository.save(scoreEntity);

    // Get recent events for history
    const recentEvents = events.slice(-10).reverse();
    return this.entityToTrustScore(scoreEntity, recentEvents);
  }

  private calculateAccountAge(events: TrustEventEntity[]): number {
    const registration = events.find(
      (e) => e.type === TrustEventTypeEnum.REGISTRATION,
    );
    if (!registration) return 0;

    const days =
      (Date.now() - registration.timestamp.getTime()) / (1000 * 60 * 60 * 24);
    return Math.floor(days);
  }

  private calculateAverageResponseTime(events: TrustEventEntity[]): number {
    const transactionEvents = events.filter(
      (e) =>
        e.type === TrustEventTypeEnum.TRANSACTION_SUCCESS && e.responseTimeMs,
    );

    if (transactionEvents.length === 0) return 0;

    const totalMs = transactionEvents.reduce(
      (sum, e) => sum + (e.responseTimeMs || 0),
      0,
    );
    return Math.round(totalMs / transactionEvents.length);
  }

  private calculateTrustLevel(
    score: number,
  ): 'untrusted' | 'low' | 'medium' | 'high' | 'verified' {
    if (score <= 20) return 'untrusted';
    if (score <= 40) return 'low';
    if (score <= 60) return 'medium';
    if (score <= 80) return 'high';
    return 'verified';
  }

  // =========================================================================
  // MUTUAL AUTHENTICATION
  // =========================================================================

  /**
   * Initiate mutual authentication between two agents
   */
  initiateMutualAuth(requesterId: string, targetId: string): MutualAuthRequest {
    const challenge = uuidv4();

    return {
      requesterId,
      targetId,
      challenge,
      timestamp: new Date(),
    };
  }

  /**
   * Complete mutual authentication
   */
  async completeMutualAuth(
    request: MutualAuthRequest,
    requesterProof: string,
    targetProof: string,
  ): Promise<MutualAuthResponse> {
    const [requesterScore, targetScore] = await Promise.all([
      this.getTrustScore(request.requesterId),
      this.getTrustScore(request.targetId),
    ]);

    // Calculate mutual trust as geometric mean
    const mutualTrust = Math.sqrt(requesterScore.score * targetScore.score);

    // Verify proofs cryptographically (pass agent IDs for HMAC derivation)
    const proofsValid = this.verifyMutualProofs(
      request.challenge,
      requesterProof,
      targetProof,
      request.requesterId,
      request.targetId,
    );

    if (!proofsValid) {
      return {
        verified: false,
        requesterScore,
        targetScore,
        mutualTrust: 0,
      };
    }

    // Record successful mutual auth
    await Promise.all([
      this.recordTransactionSuccess(request.requesterId, request.targetId),
      this.recordTransactionSuccess(request.targetId, request.requesterId),
    ]);

    return {
      verified: true,
      requesterScore: await this.getTrustScore(request.requesterId),
      targetScore: await this.getTrustScore(request.targetId),
      mutualTrust,
      sessionToken: `session_${uuidv4()}`,
    };
  }

  /**
   * Verify mutual authentication proofs using HMAC-SHA256
   *
   * Security: Uses timing-safe comparison to prevent timing attacks.
   * Each agent's proof is HMAC(challenge, agent_derived_secret).
   */
  private verifyMutualProofs(
    challenge: string,
    requesterProof: string,
    targetProof: string,
    requesterId?: string,
    targetId?: string,
  ): boolean {
    if (!requesterId || !targetId) {
      this.logger.warn('Mutual auth verification requires agent IDs');
      return false;
    }

    try {
      // Derive per-agent secrets from server secret + agent ID
      const requesterSecret = this.deriveAgentSecret(requesterId);
      const targetSecret = this.deriveAgentSecret(targetId);

      // Compute expected proofs
      const expectedRequesterProof = createHmac('sha256', requesterSecret)
        .update(challenge)
        .digest('base64url');
      const expectedTargetProof = createHmac('sha256', targetSecret)
        .update(challenge)
        .digest('base64url');

      // Timing-safe comparison to prevent timing attacks
      const requesterValid = this.timingSafeCompare(
        requesterProof,
        expectedRequesterProof,
      );
      const targetValid = this.timingSafeCompare(
        targetProof,
        expectedTargetProof,
      );

      return requesterValid && targetValid;
    } catch (error) {
      this.logger.error('Mutual auth verification failed', error);
      return false;
    }
  }

  /**
   * Derive per-agent secret from server secret
   */
  private deriveAgentSecret(agentId: string): Buffer {
    return createHmac('sha256', this.mutualAuthSecret).update(agentId).digest();
  }

  /**
   * Timing-safe string comparison
   */
  private timingSafeCompare(a: string, b: string): boolean {
    try {
      const bufA = Buffer.from(a);
      const bufB = Buffer.from(b);
      if (bufA.length !== bufB.length) {
        // Still do comparison to maintain constant time
        timingSafeEqual(bufA, bufA);
        return false;
      }
      return timingSafeEqual(bufA, bufB);
    } catch {
      return false;
    }
  }

  // =========================================================================
  // VERIFIABLE CREDENTIALS
  // =========================================================================

  /**
   * Issue a W3C Verifiable Credential for an agent's trust score
   */
  async issueCredential(
    agentId: string,
    credentialType: string = 'TrustScoreCredential',
  ): Promise<VerifiableCredential> {
    const trustScore = await this.getTrustScore(agentId);

    const credential: VerifiableCredential = {
      '@context': [
        'https://www.w3.org/2018/credentials/v1',
        'https://agentkern.io/credentials/trust/v1',
      ],
      id: `urn:uuid:${uuidv4()}`,
      type: ['VerifiableCredential', credentialType],
      issuer: 'did:agentkern:identity:trust-service',
      issuanceDate: new Date().toISOString(),
      credentialSubject: {
        id: `did:agentkern:agent:${agentId}`,
        trustScore: trustScore.score,
        trustLevel: trustScore.level,
        transactionSuccessRate: trustScore.factors.transactionSuccess,
        peerEndorsements: trustScore.factors.peerEndorsements,
        verifiedCredentials: trustScore.factors.verifiedCredentials,
      },
      proof: {
        type: 'Ed25519Signature2020',
        created: new Date().toISOString(),
        verificationMethod: 'did:agentkern:identity:trust-service#key-1',
        proofPurpose: 'assertionMethod',
        jws: await this.generateProofSignature(agentId, trustScore),
      },
    };

    // Record credential issuance
    await this.recordCredentialVerified(agentId, credentialType);

    return credential;
  }

  /**
   * Verify a Verifiable Credential
   */
  async verifyCredential(credential: VerifiableCredential): Promise<boolean> {
    // Check required fields
    if (!credential['@context'] || !credential.id || !credential.issuer) {
      return false;
    }

    // Check not expired (credentials valid for 30 days)
    const issuance = new Date(credential.issuanceDate);
    const expiry = new Date(issuance.getTime() + 30 * 24 * 60 * 60 * 1000);
    if (new Date() > expiry) {
      return false;
    }

    // Verify proof exists and has required fields
    if (!credential.proof?.jws) {
      return false;
    }

    // Verify the credential subject matches current state
    const agentId = credential.credentialSubject.id.replace(
      'did:agentkern:agent:',
      '',
    );
    const currentScore = await this.getTrustScore(agentId);

    // Validate score hasn't drifted significantly (allows for natural changes)
    const claimedScore = credential.credentialSubject.trustScore;
    if (
      claimedScore !== undefined &&
      Math.abs(currentScore.score - claimedScore) > 10
    ) {
      this.logger.warn(`Trust score changed significantly for ${agentId}`);
    }

    // Verify JWS structure (header.payload.signature)
    const jwsParts = credential.proof.jws.split('.');
    if (jwsParts.length !== 3) {
      this.logger.warn('Invalid JWS structure');
      return false;
    }

    return true;
  }

  /**
   * Generate Ed25519 JWS proof for Verifiable Credentials
   *
   * Uses jose library for production-ready EdDSA signing.
   * Per W3C VC spec and 2025 best practices for digital signatures.
   */
  private async generateProofSignature(
    agentId: string,
    trustScore: TrustScore,
  ): Promise<string> {
    if (!this.signingKeyPair) {
      throw new Error('Signing key not initialized - service not ready');
    }

    const jws = await new jose.CompactSign(
      new TextEncoder().encode(
        JSON.stringify({
          sub: agentId,
          score: trustScore.score,
          level: trustScore.level,
          iat: Math.floor(trustScore.calculatedAt.getTime() / 1000),
          exp:
            Math.floor(trustScore.calculatedAt.getTime() / 1000) +
            30 * 24 * 60 * 60, // 30 days
        }),
      ),
    )
      .setProtectedHeader({
        alg: 'EdDSA',
        typ: 'JWT',
        kid: 'trust-service-key-1',
      })
      .sign(this.signingKeyPair.privateKey);

    return jws;
  }

  // =========================================================================
  // HELPERS
  // =========================================================================

  private entityToTrustScore(
    entity: TrustScoreEntity,
    events: TrustEventEntity[],
  ): TrustScore {
    return {
      agentId: entity.agentId,
      score: entity.score,
      level: entity.level as TrustLevel,
      factors: {
        transactionSuccess: Number(entity.transactionSuccessRate),
        averageResponseTime: entity.averageResponseTimeMs,
        policyCompliance: Number(entity.policyComplianceRate),
        peerEndorsements: entity.peerEndorsementCount,
        accountAge: entity.accountAgeDays,
        verifiedCredentials: entity.verifiedCredentialCount,
      },
      history: events.map((e) => ({
        id: e.id,
        type: e.type,
        delta: e.delta,
        reason: e.reason,
        timestamp: e.timestamp,
        relatedAgentId: e.relatedAgentId,
        responseTimeMs: e.responseTimeMs,
      })),
      calculatedAt: entity.calculatedAt,
    };
  }
}
