/**
 * Entity Tests - Full Coverage
 */

import { AuditEventEntity, AuditEventTypeEnum } from './audit-event.entity';
import { PolicyEntity, PolicyActionEnum, PolicyRule } from './policy.entity';
import { TrustRecordEntity } from './trust-record.entity';

describe('Entities', () => {
  describe('AuditEventEntity', () => {
    it('should create an audit event', () => {
      const event = new AuditEventEntity();
      event.id = 'event-1';
      event.type = AuditEventTypeEnum.PROOF_VERIFICATION_SUCCESS;
      event.timestamp = new Date();
      event.proofId = 'proof-1';
      event.principalId = 'principal-1';
      event.agentId = 'agent-1';
      event.success = true;
      event.ipAddress = '127.0.0.1';
      event.userAgent = 'test-agent';
      event.metadata = { action: 'test' };

      expect(event.id).toBe('event-1');
      expect(event.type).toBe(AuditEventTypeEnum.PROOF_VERIFICATION_SUCCESS);
      expect(event.success).toBe(true);
    });

    it('should create failed audit event', () => {
      const event = new AuditEventEntity();
      event.id = 'event-2';
      event.type = AuditEventTypeEnum.PROOF_VERIFICATION_FAILURE;
      event.success = false;
      event.errorMessage = 'Invalid signature';

      expect(event.success).toBe(false);
      expect(event.errorMessage).toBe('Invalid signature');
    });

    it('should support all audit event types', () => {
      expect(AuditEventTypeEnum.PROOF_VERIFICATION_SUCCESS).toBe('proof.verification.success');
      expect(AuditEventTypeEnum.PROOF_VERIFICATION_FAILURE).toBe('proof.verification.failure');
      expect(AuditEventTypeEnum.KEY_REGISTERED).toBe('key.registered');
      expect(AuditEventTypeEnum.KEY_REVOKED).toBe('key.revoked');
      expect(AuditEventTypeEnum.RATE_LIMIT_EXCEEDED).toBe('security.rate_limit_exceeded');
      expect(AuditEventTypeEnum.SECURITY_ALERT).toBe('security.alert');
    });

    it('should set action and target fields', () => {
      const event = new AuditEventEntity();
      event.action = 'transfer';
      event.target = '/api/bank/transfer';
      
      expect(event.action).toBe('transfer');
      expect(event.target).toBe('/api/bank/transfer');
    });
  });

  describe('PolicyEntity', () => {
    it('should create a policy', () => {
      const policy = new PolicyEntity();
      policy.id = 'policy-1';
      policy.name = 'Test Policy';
      policy.description = 'A test policy';
      policy.active = true;
      policy.createdAt = new Date();
      policy.updatedAt = new Date();

      expect(policy.id).toBe('policy-1');
      expect(policy.name).toBe('Test Policy');
      expect(policy.active).toBe(true);
    });

    it('should store policy rules', () => {
      const policy = new PolicyEntity();
      policy.id = 'policy-2';
      policy.name = 'With Rules';
      policy.description = 'Policy with rules';
      policy.rules = [
        {
          name: 'Rate Limit Rule',
          condition: 'amount > 10000',
          action: PolicyActionEnum.RATE_LIMIT,
          rateLimit: 100,
        },
      ];

      expect(policy.rules[0].name).toBe('Rate Limit Rule');
      expect(policy.rules[0].action).toBe(PolicyActionEnum.RATE_LIMIT);
      expect(policy.rules[0].rateLimit).toBe(100);
    });

    it('should store target agents and principals', () => {
      const policy = new PolicyEntity();
      policy.targetAgents = ['agent-1', 'agent-2'];
      policy.targetPrincipals = ['principal-1'];

      expect(policy.targetAgents).toContain('agent-1');
      expect(policy.targetPrincipals).toContain('principal-1');
    });

    it('should support all policy actions', () => {
      expect(PolicyActionEnum.ALLOW).toBe('ALLOW');
      expect(PolicyActionEnum.DENY).toBe('DENY');
      expect(PolicyActionEnum.RATE_LIMIT).toBe('RATE_LIMIT');
      expect(PolicyActionEnum.REQUIRE_CONFIRMATION).toBe('REQUIRE_CONFIRMATION');
    });
  });

  describe('TrustRecordEntity', () => {
    it('should create a trust record', () => {
      const record = new TrustRecordEntity();
      record.id = 'record-1';
      record.agentId = 'agent-1';
      record.principalId = 'principal-1';
      record.trustScore = 750;
      record.trusted = true;
      record.revoked = false;
      record.verificationCount = 10;
      record.failureCount = 0;
      record.registeredAt = new Date();
      record.lastVerifiedAt = new Date();

      expect(record.id).toBe('record-1');
      expect(record.trustScore).toBe(750);
      expect(record.trusted).toBe(true);
    });

    it('should handle revoked trust record', () => {
      const record = new TrustRecordEntity();
      record.id = 'record-2';
      record.agentId = 'agent-2';
      record.principalId = 'principal-2';
      record.trustScore = 0;
      record.trusted = false;
      record.revoked = true;

      expect(record.revoked).toBe(true);
      expect(record.trusted).toBe(false);
    });

    it('should store metadata', () => {
      const record = new TrustRecordEntity();
      record.metadata = {
        agentName: 'Test Agent',
        agentVersion: '1.0.0',
        principalDevice: 'iPhone 15',
      };

      expect(record.metadata.agentName).toBe('Test Agent');
      expect(record.metadata.agentVersion).toBe('1.0.0');
    });

    it('should set expiration date', () => {
      const record = new TrustRecordEntity();
      const future = new Date(Date.now() + 86400000);
      record.expiresAt = future;

      expect(record.expiresAt).toEqual(future);
    });
  });
});
