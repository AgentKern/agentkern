/**
 * AgentKernIdentity - Trust Scoring Service
 *
 * Per MANIFESTO.md: "Agents have verifiable reputations built on their transaction history"
 * Per Market Research: No one has solved agent-to-agent trust properly
 *
 * This service provides:
 * - Agent reputation/trust scoring
 * - Transaction history tracking
 * - Agent-to-agent mutual verification
 * - W3C Verifiable Credentials issuance
 */

import { Injectable, Logger } from '@nestjs/common';
import { v4 as uuidv4 } from 'uuid';

// ============================================================================
// TYPES
// ============================================================================

export interface TrustScore {
  agentId: string;
  score: number; // 0-100
  level: TrustLevel;
  factors: TrustFactors;
  history: TrustEvent[];
  calculatedAt: Date;
}

export enum TrustLevel {
  UNTRUSTED = 'untrusted',      // 0-20
  LOW = 'low',                   // 21-40
  MEDIUM = 'medium',             // 41-60
  HIGH = 'high',                 // 61-80
  VERIFIED = 'verified',         // 81-100
}

export interface TrustFactors {
  transactionSuccess: number;    // % of successful transactions
  averageResponseTime: number;   // ms
  policyCompliance: number;      // % of policy-compliant actions
  peerEndorsements: number;      // count of positive endorsements
  accountAge: number;            // days since registration
  verifiedCredentials: number;   // count of verified credentials
}

export interface TrustEvent {
  id: string;
  type: TrustEventType;
  delta: number;                 // Change to trust score
  reason: string;
  timestamp: Date;
  relatedAgentId?: string;
  responseTimeMs?: number;       // Dec 2025: Track actual response times
}

export enum TrustEventType {
  TRANSACTION_SUCCESS = 'transaction_success',
  TRANSACTION_FAILURE = 'transaction_failure',
  POLICY_VIOLATION = 'policy_violation',
  PEER_ENDORSEMENT = 'peer_endorsement',
  PEER_REPORT = 'peer_report',
  CREDENTIAL_VERIFIED = 'credential_verified',
  REGISTRATION = 'registration',
}

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
export class TrustService {
  private readonly logger = new Logger(TrustService.name);

  // In-memory stores (production: database)
  private trustScores: Map<string, TrustScore> = new Map();
  private trustEvents: Map<string, TrustEvent[]> = new Map();

  // Configuration
  private readonly INITIAL_TRUST_SCORE = 50;
  private readonly MAX_TRUST_SCORE = 100;
  private readonly MIN_TRUST_SCORE = 0;

  // Weight factors for trust calculation
  private readonly WEIGHTS = {
    transactionSuccess: 0.30,
    policyCompliance: 0.25,
    peerEndorsements: 0.20,
    accountAge: 0.15,
    verifiedCredentials: 0.10,
  };

  // =========================================================================
  // TRUST SCORE OPERATIONS
  // =========================================================================

  /**
   * Get trust score for an agent.
   */
  async getTrustScore(agentId: string): Promise<TrustScore> {
    let score = this.trustScores.get(agentId);

    if (!score) {
      // Initialize new agent with default score
      score = this.initializeTrustScore(agentId);
    }

    return score;
  }

  /**
   * Initialize trust score for new agent.
   */
  private initializeTrustScore(agentId: string): TrustScore {
    const score: TrustScore = {
      agentId,
      score: this.INITIAL_TRUST_SCORE,
      level: TrustLevel.MEDIUM,
      factors: {
        transactionSuccess: 100, // No failures yet
        averageResponseTime: 0,
        policyCompliance: 100,   // No violations yet
        peerEndorsements: 0,
        accountAge: 0,
        verifiedCredentials: 0,
      },
      history: [],
      calculatedAt: new Date(),
    };

    // Record registration event
    this.recordEvent(agentId, {
      id: uuidv4(),
      type: TrustEventType.REGISTRATION,
      delta: 0,
      reason: 'Agent registered',
      timestamp: new Date(),
    });

    this.trustScores.set(agentId, score);
    this.logger.log(`Initialized trust score for agent: ${agentId}`);

    return score;
  }

  /**
   * Record a trust-affecting event.
   */
  async recordEvent(agentId: string, event: TrustEvent): Promise<TrustScore> {
    // Get or create events list
    let events = this.trustEvents.get(agentId);
    if (!events) {
      events = [];
      this.trustEvents.set(agentId, events);
    }

    events.push(event);

    // Keep only last 1000 events
    if (events.length > 1000) {
      events.shift();
    }

    // Recalculate trust score
    return this.recalculateTrustScore(agentId);
  }

  /**
   * Record successful transaction.
   */
  async recordTransactionSuccess(
    agentId: string,
    relatedAgentId?: string,
    responseTimeMs?: number,
  ): Promise<TrustScore> {
    return this.recordEvent(agentId, {
      id: uuidv4(),
      type: TrustEventType.TRANSACTION_SUCCESS,
      delta: 1,
      reason: 'Transaction completed successfully',
      timestamp: new Date(),
      relatedAgentId,
      responseTimeMs,
    });
  }

  /**
   * Record failed transaction.
   */
  async recordTransactionFailure(
    agentId: string,
    reason: string,
    relatedAgentId?: string,
  ): Promise<TrustScore> {
    return this.recordEvent(agentId, {
      id: uuidv4(),
      type: TrustEventType.TRANSACTION_FAILURE,
      delta: -5,
      reason,
      timestamp: new Date(),
      relatedAgentId,
    });
  }

  /**
   * Record policy violation.
   */
  async recordPolicyViolation(
    agentId: string,
    policyId: string,
  ): Promise<TrustScore> {
    return this.recordEvent(agentId, {
      id: uuidv4(),
      type: TrustEventType.POLICY_VIOLATION,
      delta: -10,
      reason: `Violated policy: ${policyId}`,
      timestamp: new Date(),
    });
  }

  /**
   * Record peer endorsement.
   */
  async recordPeerEndorsement(
    agentId: string,
    endorserId: string,
  ): Promise<TrustScore> {
    // Check endorser has sufficient trust to endorse
    const endorserScore = await this.getTrustScore(endorserId);
    if (endorserScore.score < 60) {
      this.logger.warn(`Endorser ${endorserId} has insufficient trust score`);
      return this.getTrustScore(agentId);
    }

    return this.recordEvent(agentId, {
      id: uuidv4(),
      type: TrustEventType.PEER_ENDORSEMENT,
      delta: 3,
      reason: `Endorsed by ${endorserId}`,
      timestamp: new Date(),
      relatedAgentId: endorserId,
    });
  }

  /**
   * Recalculate trust score based on all events.
   */
  private recalculateTrustScore(agentId: string): TrustScore {
    const events = this.trustEvents.get(agentId) || [];
    let score = this.trustScores.get(agentId);

    if (!score) {
      score = this.initializeTrustScore(agentId);
    }

    // Calculate factors from events
    const successEvents = events.filter(
      (e) => e.type === TrustEventType.TRANSACTION_SUCCESS,
    );
    const failEvents = events.filter(
      (e) => e.type === TrustEventType.TRANSACTION_FAILURE,
    );
    const violationEvents = events.filter(
      (e) => e.type === TrustEventType.POLICY_VIOLATION,
    );
    const endorsementEvents = events.filter(
      (e) => e.type === TrustEventType.PEER_ENDORSEMENT,
    );
    const credentialEvents = events.filter(
      (e) => e.type === TrustEventType.CREDENTIAL_VERIFIED,
    );

    const totalTransactions = successEvents.length + failEvents.length;
    const totalActions = successEvents.length + violationEvents.length;

    // Update factors
    score.factors = {
      transactionSuccess:
        totalTransactions > 0
          ? (successEvents.length / totalTransactions) * 100
          : 100,
      averageResponseTime: this.calculateAverageResponseTime(events),
      policyCompliance:
        totalActions > 0
          ? ((totalActions - violationEvents.length) / totalActions) * 100
          : 100,
      peerEndorsements: endorsementEvents.length,
      accountAge: this.calculateAccountAge(events),
      verifiedCredentials: credentialEvents.length,
    };

    // Calculate overall score
    const weightedScore =
      (score.factors.transactionSuccess / 100) * this.WEIGHTS.transactionSuccess +
      (score.factors.policyCompliance / 100) * this.WEIGHTS.policyCompliance +
      Math.min(score.factors.peerEndorsements / 10, 1) * this.WEIGHTS.peerEndorsements +
      Math.min(score.factors.accountAge / 365, 1) * this.WEIGHTS.accountAge +
      Math.min(score.factors.verifiedCredentials / 3, 1) * this.WEIGHTS.verifiedCredentials;

    score.score = Math.round(weightedScore * 100);
    score.score = Math.max(this.MIN_TRUST_SCORE, Math.min(this.MAX_TRUST_SCORE, score.score));
    score.level = this.calculateTrustLevel(score.score);
    score.history = events.slice(-10); // Last 10 events
    score.calculatedAt = new Date();

    this.trustScores.set(agentId, score);
    return score;
  }

  private calculateAccountAge(events: TrustEvent[]): number {
    const registration = events.find(
      (e) => e.type === TrustEventType.REGISTRATION,
    );
    if (!registration) return 0;

    const now = new Date();
    const diffMs = now.getTime() - registration.timestamp.getTime();
    return Math.floor(diffMs / (1000 * 60 * 60 * 24)); // Days
  }

  /**
   * Calculate average response time from transaction events.
   * Dec 2025: Fixed TODO - now tracks actual response times.
   */
  private calculateAverageResponseTime(events: TrustEvent[]): number {
    const eventsWithTiming = events.filter(
      (e) =>
        (e.type === TrustEventType.TRANSACTION_SUCCESS ||
          e.type === TrustEventType.TRANSACTION_FAILURE) &&
        e.responseTimeMs !== undefined,
    );

    if (eventsWithTiming.length === 0) return 0;

    const totalMs = eventsWithTiming.reduce(
      (sum, e) => sum + (e.responseTimeMs || 0),
      0,
    );

    return Math.round(totalMs / eventsWithTiming.length);
  }

  private calculateTrustLevel(score: number): TrustLevel {
    if (score <= 20) return TrustLevel.UNTRUSTED;
    if (score <= 40) return TrustLevel.LOW;
    if (score <= 60) return TrustLevel.MEDIUM;
    if (score <= 80) return TrustLevel.HIGH;
    return TrustLevel.VERIFIED;
  }

  // =========================================================================
  // AGENT-TO-AGENT MUTUAL AUTHENTICATION
  // =========================================================================

  /**
   * Initiate mutual authentication between two agents.
   */
  async initiateMutualAuth(
    requesterId: string,
    targetId: string,
  ): Promise<MutualAuthRequest> {
    const challenge = uuidv4();

    const request: MutualAuthRequest = {
      requesterId,
      targetId,
      challenge,
      timestamp: new Date(),
    };

    this.logger.log(
      `Mutual auth initiated: ${requesterId} -> ${targetId}`,
    );

    return request;
  }

  /**
   * Complete mutual authentication.
   */
  async completeMutualAuth(
    request: MutualAuthRequest,
    challengeResponse: string,
  ): Promise<MutualAuthResponse> {
    // Verify challenge response (simplified - real impl uses crypto)
    const expectedResponse = Buffer.from(request.challenge).toString('base64');
    const verified = challengeResponse === expectedResponse;

    if (!verified) {
      return {
        verified: false,
        requesterScore: await this.getTrustScore(request.requesterId),
        targetScore: await this.getTrustScore(request.targetId),
        mutualTrust: 0,
      };
    }

    const requesterScore = await this.getTrustScore(request.requesterId);
    const targetScore = await this.getTrustScore(request.targetId);

    // Mutual trust is the geometric mean of both scores
    const mutualTrust = Math.sqrt(requesterScore.score * targetScore.score);

    // Generate session token for authenticated session
    const sessionToken = uuidv4();

    this.logger.log(
      `Mutual auth completed: ${request.requesterId} <-> ${request.targetId}, trust: ${mutualTrust}`,
    );

    return {
      verified: true,
      requesterScore,
      targetScore,
      mutualTrust: Math.round(mutualTrust),
      sessionToken,
    };
  }

  // =========================================================================
  // VERIFIABLE CREDENTIALS
  // =========================================================================

  /**
   * Issue a W3C Verifiable Credential for an agent's trust score.
   */
  async issueCredential(
    agentId: string,
    credentialType: string = 'TrustScoreCredential',
  ): Promise<VerifiableCredential> {
    const score = await this.getTrustScore(agentId);

    const credential: VerifiableCredential = {
      '@context': [
        'https://www.w3.org/2018/credentials/v1',
        'https://agentkern.io/credentials/v1',
      ],
      id: `urn:uuid:${uuidv4()}`,
      type: ['VerifiableCredential', credentialType],
      issuer: 'did:web:agentkern.io',
      issuanceDate: new Date().toISOString(),
      credentialSubject: {
        id: `did:agentkern:${agentId}`,
        trustScore: score.score,
        trustLevel: score.level,
        transactionSuccessRate: score.factors.transactionSuccess,
        policyComplianceRate: score.factors.policyCompliance,
        peerEndorsements: score.factors.peerEndorsements,
      },
      // Proof would be added by proof-signing.service.ts
    };

    this.logger.log(`Issued credential for agent: ${agentId}`);

    // Record credential issuance
    await this.recordEvent(agentId, {
      id: uuidv4(),
      type: TrustEventType.CREDENTIAL_VERIFIED,
      delta: 2,
      reason: `Credential issued: ${credentialType}`,
      timestamp: new Date(),
    });

    return credential;
  }

  /**
   * Verify a Verifiable Credential.
   */
  async verifyCredential(credential: VerifiableCredential): Promise<boolean> {
    // Basic validation
    if (!credential['@context'] || !credential.type || !credential.issuer) {
      return false;
    }

    // Check issuer is AgentKern
    if (credential.issuer !== 'did:web:agentkern.io') {
      this.logger.warn(`Unknown credential issuer: ${credential.issuer}`);
      return false;
    }

    // Check not expired (credentials valid for 30 days)
    const issuance = new Date(credential.issuanceDate);
    const expiry = new Date(issuance.getTime() + 30 * 24 * 60 * 60 * 1000);
    if (new Date() > expiry) {
      return false;
    }

    // In production: verify cryptographic proof
    // For now: trust if format is valid
    return true;
  }
}
