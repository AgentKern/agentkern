/**
 * Audit Logger Service Tests
 */

import { Test, TestingModule } from '@nestjs/testing';
import { AuditLoggerService, AuditEventType } from './audit-logger.service';

describe('AuditLoggerService', () => {
  let service: AuditLoggerService;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [AuditLoggerService],
    }).compile();

    service = module.get<AuditLoggerService>(AuditLoggerService);
  });

  describe('log', () => {
    it('should create an audit event with ID and timestamp', () => {
      const event = service.log({
        type: AuditEventType.PROOF_VERIFICATION_SUCCESS,
        success: true,
      });
      
      expect(event.id).toBeDefined();
      expect(event.timestamp).toBeDefined();
    });
  });

  describe('logVerificationSuccess', () => {
    it('should log successful verification', () => {
      const event = service.logVerificationSuccess(
        'proof-123',
        'principal-1',
        'agent-1',
        'transfer',
        'bank.api/transfer',
      );
      
      expect(event.type).toBe(AuditEventType.PROOF_VERIFICATION_SUCCESS);
      expect(event.success).toBe(true);
      expect(event.proofId).toBe('proof-123');
      expect(event.principalId).toBe('principal-1');
      expect(event.agentId).toBe('agent-1');
    });
  });

  describe('logVerificationFailure', () => {
    it('should log failed verification', () => {
      const event = service.logVerificationFailure(
        'proof-456',
        'Invalid signature',
      );
      
      expect(event.type).toBe(AuditEventType.PROOF_VERIFICATION_FAILURE);
      expect(event.success).toBe(false);
      expect(event.errorMessage).toBe('Invalid signature');
    });
  });

  describe('logSecurityEvent', () => {
    it('should log security event', () => {
      const event = service.logSecurityEvent(
        AuditEventType.RATE_LIMIT_EXCEEDED,
        'Rate limit exceeded',
        { endpoint: '/api/v1/proof' },
      );
      
      expect(event.type).toBe(AuditEventType.RATE_LIMIT_EXCEEDED);
      expect(event.metadata?.endpoint).toBe('/api/v1/proof');
    });
  });

  describe('getAuditTrailForPrincipal', () => {
    it('should return events for principal', () => {
      service.logVerificationSuccess('p1', 'principal-a', 'agent-1', 'test', 'target');
      service.logVerificationSuccess('p2', 'principal-a', 'agent-2', 'test', 'target');
      service.logVerificationSuccess('p3', 'principal-b', 'agent-1', 'test', 'target');
      
      const trail = service.getAuditTrailForPrincipal('principal-a');
      expect(trail.length).toBe(2);
    });
  });

  describe('getAuditTrailForProof', () => {
    it('should return events for proof', () => {
      service.logVerificationSuccess('proof-abc', 'p1', 'a1', 'test', 'target');
      service.logVerificationFailure('proof-abc', 'Error');
      
      const trail = service.getAuditTrailForProof('proof-abc');
      expect(trail.length).toBe(2);
    });
  });

  describe('getSecurityEvents', () => {
    it('should return only security events', () => {
      service.logVerificationSuccess('p1', 'p1', 'a1', 'test', 'target');
      service.logSecurityEvent(AuditEventType.RATE_LIMIT_EXCEEDED, 'Rate limit');
      service.logSecurityEvent(AuditEventType.SUSPICIOUS_ACTIVITY, 'Suspicious');
      
      const secEvents = service.getSecurityEvents();
      expect(secEvents.length).toBe(2);
    });
  });

  describe('getRecentEvents', () => {
    it('should return recent events up to limit', () => {
      for (let i = 0; i < 10; i++) {
        service.log({
          type: AuditEventType.PROOF_VERIFICATION_SUCCESS,
          success: true,
        });
      }
      
      const recent = service.getRecentEvents(5);
      expect(recent.length).toBe(5);
    });
  });

  describe('exportAuditLog', () => {
    it('should filter by type', () => {
      service.logVerificationSuccess('p1', 'p1', 'a1', 'test', 'target');
      service.logVerificationFailure('p2', 'Error');
      
      const exported = service.exportAuditLog({
        types: [AuditEventType.PROOF_VERIFICATION_SUCCESS],
      });
      
      expect(exported.every(e => e.type === AuditEventType.PROOF_VERIFICATION_SUCCESS)).toBe(true);
    });

    it('should filter by principalId', () => {
      service.logVerificationSuccess('p1', 'principal-x', 'a1', 'test', 'target');
      service.logVerificationSuccess('p2', 'principal-y', 'a1', 'test', 'target');
      
      const exported = service.exportAuditLog({
        principalId: 'principal-x',
      });
      
      expect(exported.every(e => e.principalId === 'principal-x')).toBe(true);
    });
  });
});
