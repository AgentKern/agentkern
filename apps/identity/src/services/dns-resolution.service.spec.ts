/**
 * DNS Resolution Service Tests
 */

import { Test, TestingModule } from '@nestjs/testing';
import { DnsResolutionService } from './dns-resolution.service';
import { AuditLoggerService } from './audit-logger.service';

describe('DnsResolutionService', () => {
  let service: DnsResolutionService;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [DnsResolutionService, AuditLoggerService],
    }).compile();

    service = module.get<DnsResolutionService>(DnsResolutionService);
  });

  describe('registerTrust', () => {
    it('should register new trust relationship', () => {
      const result = service.registerTrust('agent-1', 'principal-1', {
        agentName: 'Test Agent',
        agentVersion: '1.0.0',
      });
      
      expect(result.agentId).toBe('agent-1');
      expect(result.principalId).toBe('principal-1');
      expect(result.trusted).toBe(true);
      expect(result.revoked).toBe(false);
    });

    it('should return existing record if already exists', () => {
      service.registerTrust('agent-1', 'principal-1');
      const result = service.registerTrust('agent-1', 'principal-1');
      expect(result).toBeDefined();
    });
  });

  describe('resolve', () => {
    it('should resolve existing trust', async () => {
      service.registerTrust('agent-1', 'principal-1');
      const resolution = await service.resolve({ agentId: 'agent-1', principalId: 'principal-1' });
      
      expect(resolution.trusted).toBe(true);
      expect(resolution.trustScore).toBeGreaterThan(0);
    });

    it('should return low trust for unknown agent', async () => {
      const resolution = await service.resolve({ agentId: 'unknown-agent', principalId: 'principal-1' });
      expect(resolution.trusted).toBe(true); // New records start trusted
      expect(resolution.trustScore).toBe(500); // Default score
    });
  });

  describe('revokeTrust', () => {
    it('should revoke existing trust', async () => {
      service.registerTrust('agent-1', 'principal-1');
      const result = service.revokeTrust('agent-1', 'principal-1', 'Compromised');
      
      expect(result).toBeDefined();
      expect(result?.revoked).toBe(true);
      expect(result?.trusted).toBe(false);
    });

    it('should return null for non-existent trust', () => {
      const result = service.revokeTrust('fake', 'fake', 'reason');
      expect(result).toBeNull();
    });
  });

  describe('reinstateTrust', () => {
    it('should reinstate revoked trust', () => {
      service.registerTrust('agent-1', 'principal-1');
      service.revokeTrust('agent-1', 'principal-1', 'Temporary');
      const result = service.reinstateTrust('agent-1', 'principal-1');
      
      expect(result).toBeDefined();
      expect(result?.revoked).toBe(false);
    });
  });

  describe('recordVerificationSuccess', () => {
    it('should increment verification count', () => {
      service.registerTrust('agent-1', 'principal-1');
      service.recordVerificationSuccess('agent-1', 'principal-1');
      
      const record = service.getTrustRecord('agent-1', 'principal-1');
      expect(record?.verificationCount).toBe(1);
    });
  });

  describe('recordVerificationFailure', () => {
    it('should increment failure count', () => {
      service.registerTrust('agent-1', 'principal-1');
      service.recordVerificationFailure('agent-1', 'principal-1');
      
      const record = service.getTrustRecord('agent-1', 'principal-1');
      expect(record?.failureCount).toBe(1);
    });
  });

  describe('getTrustRecordsForPrincipal', () => {
    it('should return all records for principal', () => {
      service.registerTrust('agent-1', 'principal-1');
      service.registerTrust('agent-2', 'principal-1');
      service.registerTrust('agent-3', 'principal-2');
      
      const records = service.getTrustRecordsForPrincipal('principal-1');
      expect(records.length).toBe(2);
    });
  });

  describe('resolveBatch', () => {
    it('should resolve multiple queries', async () => {
      service.registerTrust('agent-1', 'principal-1');
      service.registerTrust('agent-2', 'principal-1');
      
      const results = await service.resolveBatch([
        { agentId: 'agent-1', principalId: 'principal-1' },
        { agentId: 'agent-2', principalId: 'principal-1' },
        { agentId: 'unknown', principalId: 'principal-1' },
      ]);
      
      expect(results.length).toBe(3);
      expect(results[0].trusted).toBe(true);
      expect(results[1].trusted).toBe(true);
    });
  });
});
