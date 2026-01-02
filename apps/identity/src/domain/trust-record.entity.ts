/**
 * AgentKernIdentity - Trust Record Entity
 *
 * Represents an agent's trust status in the Intent DNS system.
 * Used for caching and resolving trust lookups.
 */

export class TrustRecord {
  id: string;
  agentId: string;
  principalId: string;
  trustScore: number;
  trusted: boolean;
  revoked: boolean;
  registeredAt: string | Date;
  lastVerifiedAt: string | Date;
  expiresAt: string | Date;
  verificationCount: number;
  failureCount: number;
  metadata?: {
    agentName?: string;
    agentVersion?: string;
    principalDevice?: string;
  };
}

export interface TrustQuery {
  agentId: string;
  principalId: string;
}

export interface TrustResolution {
  version: string;
  agentId: string;
  principalId: string;
  trusted: boolean;
  trustScore: number;
  expiresAt: string;
  revoked: boolean;
  cachedAt: string;
  ttl: number;
}

/**
 * Calculate trust score based on verification history
 */
export function calculateTrustScore(
  verificationCount: number,
  failureCount: number,
  daysSinceLastVerification: number,
  daysActive: number,
  revocationCount: number = 0,
): number {
  const baseScore = 500;

  const score =
    baseScore +
    verificationCount * 2 -
    failureCount * 10 -
    daysSinceLastVerification * 1 +
    Math.min(daysActive, 100) - // Age bonus capped at 100
    revocationCount * 50;

  // Clamp between 0 and 1000
  return Math.max(0, Math.min(1000, Math.round(score)));
}

/**
 * Determine if an agent should be trusted based on score
 */
export function isTrusted(score: number, threshold: number = 500): boolean {
  return score >= threshold;
}

/**
 * Calculate TTL based on trust score
 */
export function calculateTTL(score: number, revoked: boolean): number {
  if (revoked) return 0; // No caching for revoked

  if (score >= 800) return 3600; // 1 hour
  if (score >= 500) return 900; // 15 minutes
  return 300; // 5 minutes
}

/**
 * Create a new trust record with default values
 */
export function createTrustRecord(
  agentId: string,
  principalId: string,
  metadata?: {
    agentName?: string;
    agentVersion?: string;
    principalDevice?: string;
  },
): TrustRecord {
  const now = new Date();
  const expiresAt = new Date(Date.now() + 365 * 24 * 60 * 60 * 1000); // 1 year

  return {
    id: crypto.randomUUID(),
    agentId,
    principalId,
    trustScore: 500, // Base score for new agents
    trusted: true,
    revoked: false,
    registeredAt: now,
    lastVerifiedAt: now,
    expiresAt,
    verificationCount: 0,
    failureCount: 0,
    metadata,
  };
}

/**
 * Create a trust resolution response
 */
export function createTrustResolution(record: TrustRecord): TrustResolution {
  const ttl = calculateTTL(record.trustScore, record.revoked);

  return {
    version: '1.0',
    agentId: record.agentId,
    principalId: record.principalId,
    trusted: record.trusted && !record.revoked,
    trustScore: record.trustScore,
    expiresAt:
      typeof record.expiresAt === 'string'
        ? record.expiresAt
        : record.expiresAt.toISOString(),
    revoked: record.revoked,
    cachedAt: new Date().toISOString(),
    ttl,
  };
}
