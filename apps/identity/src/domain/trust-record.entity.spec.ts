/**
 * AgentKernIdentity - Trust Record Entity Tests
 *
 * Unit tests for trust scoring and resolution functions.
 */

import {
  calculateTrustScore,
  isTrusted,
  calculateTTL,
  createTrustRecord,
  createTrustResolution,
} from './trust-record.entity';

describe('TrustRecordEntity', () => {
  describe('calculateTrustScore', () => {
    it('should return base score for new agent', () => {
      const score = calculateTrustScore(0, 0, 0, 0);
      expect(score).toBe(500);
    });

    it('should increase score with successful verifications', () => {
      const score = calculateTrustScore(50, 0, 0, 0);
      expect(score).toBe(600); // 500 + 50*2
    });

    it('should decrease score with failed verifications', () => {
      const score = calculateTrustScore(0, 5, 0, 0);
      expect(score).toBe(450); // 500 - 5*10
    });

    it('should add age bonus capped at 100', () => {
      const youngScore = calculateTrustScore(0, 0, 0, 30);
      expect(youngScore).toBe(530); // 500 + 30

      const oldScore = calculateTrustScore(0, 0, 0, 200);
      expect(oldScore).toBe(600); // 500 + 100 (capped)
    });

    it('should decrease score for revocations', () => {
      const score = calculateTrustScore(0, 0, 0, 0, 2);
      expect(score).toBe(400); // 500 - 2*50
    });

    it('should clamp score between 0 and 1000', () => {
      const lowScore = calculateTrustScore(0, 100, 0, 0);
      expect(lowScore).toBe(0);

      const highScore = calculateTrustScore(500, 0, 0, 100);
      expect(highScore).toBe(1000);
    });
  });

  describe('isTrusted', () => {
    it('should return true for score >= 500', () => {
      expect(isTrusted(500)).toBe(true);
      expect(isTrusted(800)).toBe(true);
    });

    it('should return false for score < 500', () => {
      expect(isTrusted(499)).toBe(false);
      expect(isTrusted(0)).toBe(false);
    });

    it('should respect custom threshold', () => {
      expect(isTrusted(600, 700)).toBe(false);
      expect(isTrusted(800, 700)).toBe(true);
    });
  });

  describe('calculateTTL', () => {
    it('should return 0 for revoked', () => {
      expect(calculateTTL(900, true)).toBe(0);
    });

    it('should return 1 hour for high score', () => {
      expect(calculateTTL(850, false)).toBe(3600);
    });

    it('should return 15 minutes for medium score', () => {
      expect(calculateTTL(600, false)).toBe(900);
    });

    it('should return 5 minutes for low score', () => {
      expect(calculateTTL(300, false)).toBe(300);
    });
  });

  describe('createTrustRecord', () => {
    it('should create a record with default values', () => {
      const record = createTrustRecord('agent-1', 'user-1');

      expect(record.id).toBeDefined();
      expect(record.agentId).toBe('agent-1');
      expect(record.principalId).toBe('user-1');
      expect(record.trustScore).toBe(500);
      expect(record.trusted).toBe(true);
      expect(record.revoked).toBe(false);
      expect(record.verificationCount).toBe(0);
      expect(record.failureCount).toBe(0);
    });

    it('should include metadata when provided', () => {
      const record = createTrustRecord('agent-1', 'user-1', {
        agentName: 'Test Agent',
        agentVersion: '1.0.0',
      });

      expect(record.metadata?.agentName).toBe('Test Agent');
      expect(record.metadata?.agentVersion).toBe('1.0.0');
    });
  });

  describe('createTrustResolution', () => {
    it('should create resolution from record', () => {
      const record = createTrustRecord('agent-1', 'user-1');
      const resolution = createTrustResolution(record);

      expect(resolution.version).toBe('1.0');
      expect(resolution.agentId).toBe('agent-1');
      expect(resolution.principalId).toBe('user-1');
      expect(resolution.trusted).toBe(true);
      expect(resolution.trustScore).toBe(500);
      expect(resolution.revoked).toBe(false);
      expect(resolution.ttl).toBe(900); // 15 min for score 500
    });

    it('should mark revoked record as not trusted', () => {
      const record = createTrustRecord('agent-1', 'user-1');
      record.revoked = true;

      const resolution = createTrustResolution(record);

      expect(resolution.trusted).toBe(false);
      expect(resolution.ttl).toBe(0);
    });
  });
});
