/**
 * DNS Resolution Service Tests
 * Updated to work with async TypeORM-based service
 */

import { Test, TestingModule } from '@nestjs/testing';
import { getRepositoryToken } from '@nestjs/typeorm';
import { DnsResolutionService } from './dns-resolution.service';
import { AuditLoggerService } from './audit-logger.service';
import { TrustRecordEntity } from '../entities/trust-record.entity';

describe('DnsResolutionService', () => {
  let service: DnsResolutionService;
  let mockRepository: any;

  // Sample trust record for testing
  const createMockRecord = (agentId: string, principalId: string) => ({
    id: `${agentId}-${principalId}`,
    agentId,
    principalId,
    trusted: true,
    revoked: false,
    trustScore: 500,
    verificationCount: 0,
    failureCount: 0,
    registeredAt: new Date(),
    lastVerifiedAt: new Date(),
  });

  beforeEach(async () => {
    // Create a stateful mock repository
    const records = new Map<string, any>();
    
    mockRepository = {
      find: jest.fn().mockImplementation(({ where }) => {
        if (where?.principalId) {
          return Promise.resolve(
            Array.from(records.values()).filter(r => r.principalId === where.principalId)
          );
        }
        return Promise.resolve(Array.from(records.values()));
      }),
      findOne: jest.fn().mockImplementation(({ where }) => {
        const key = `${where.agentId}-${where.principalId}`;
        return Promise.resolve(records.get(key) || null);
      }),
      save: jest.fn().mockImplementation(entity => {
        const key = `${entity.agentId}-${entity.principalId}`;
        records.set(key, { ...entity, id: key });
        return Promise.resolve(records.get(key));
      }),
      create: jest.fn().mockImplementation(entity => entity),
      delete: jest.fn().mockResolvedValue({ affected: 1 }),
    };

    const module: TestingModule = await Test.createTestingModule({
      providers: [
        DnsResolutionService,
        AuditLoggerService,
        {
          provide: getRepositoryToken(TrustRecordEntity),
          useValue: mockRepository,
        },
      ],
    }).compile();

    service = module.get<DnsResolutionService>(DnsResolutionService);
  });

  describe('registerTrust', () => {
    it('should register new trust relationship', async () => {
      const result = await service.registerTrust('agent-1', 'principal-1', {
        agentName: 'Test Agent',
        agentVersion: '1.0.0',
      });
      
      expect(result.agentId).toBe('agent-1');
      expect(result.principalId).toBe('principal-1');
      expect(result.trusted).toBe(true);
      expect(result.revoked).toBe(false);
    });

    it('should create record with default values', async () => {
      const result = await service.registerTrust('agent-2', 'principal-2');
      expect(result).toBeDefined();
      expect(result.trustScore).toBe(500);
    });
  });

  describe('resolve', () => {
    it('should resolve existing trust', async () => {
      await service.registerTrust('agent-1', 'principal-1');
      const resolution = await service.resolve({ agentId: 'agent-1', principalId: 'principal-1' });
      
      expect(resolution.trusted).toBe(true);
      expect(resolution.trustScore).toBeGreaterThan(0);
    });

    it('should create and return trust for unknown agent', async () => {
      const resolution = await service.resolve({ agentId: 'unknown-agent', principalId: 'principal-1' });
      expect(resolution.trusted).toBe(true); // New records start trusted
      expect(resolution.trustScore).toBe(500); // Default score
    });
  });

  describe('revokeTrust', () => {
    it('should revoke existing trust', async () => {
      await service.registerTrust('agent-1', 'principal-1');
      const result = await service.revokeTrust('agent-1', 'principal-1', 'Compromised');
      
      expect(result).toBeDefined();
      expect(result?.revoked).toBe(true);
      expect(result?.trusted).toBe(false);
    });

    it('should return null for non-existent trust', async () => {
      const result = await service.revokeTrust('fake', 'fake', 'reason');
      expect(result).toBeNull();
    });
  });

  describe('reinstateTrust', () => {
    it('should reinstate revoked trust', async () => {
      await service.registerTrust('agent-1', 'principal-1');
      await service.revokeTrust('agent-1', 'principal-1', 'Temporary');
      const result = await service.reinstateTrust('agent-1', 'principal-1');
      
      expect(result).toBeDefined();
      expect(result?.revoked).toBe(false);
    });
  });

  describe('recordVerificationSuccess', () => {
    it('should increment verification count', async () => {
      await service.registerTrust('agent-1', 'principal-1');
      const result = await service.recordVerificationSuccess('agent-1', 'principal-1');
      
      expect(result?.verificationCount).toBe(1);
    });
  });

  describe('recordVerificationFailure', () => {
    it('should increment failure count', async () => {
      await service.registerTrust('agent-1', 'principal-1');
      const result = await service.recordVerificationFailure('agent-1', 'principal-1');
      
      expect(result?.failureCount).toBe(1);
    });
  });

  describe('getTrustRecordsForPrincipal', () => {
    it('should return all records for principal', async () => {
      await service.registerTrust('agent-1', 'principal-1');
      await service.registerTrust('agent-2', 'principal-1');
      await service.registerTrust('agent-3', 'principal-2');
      
      const records = await service.getTrustRecordsForPrincipal('principal-1');
      expect(records.length).toBe(2);
    });
  });

  describe('resolveBatch', () => {
    it('should resolve multiple queries', async () => {
      await service.registerTrust('agent-1', 'principal-1');
      await service.registerTrust('agent-2', 'principal-1');
      
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

  describe('getTrustRecord', () => {
    it('should return specific trust record', async () => {
      await service.registerTrust('agent-1', 'principal-1');
      const record = await service.getTrustRecord('agent-1', 'principal-1');
      
      expect(record).toBeDefined();
      expect(record?.agentId).toBe('agent-1');
    });

    it('should return null for non-existent record', async () => {
      const record = await service.getTrustRecord('fake', 'fake');
      expect(record).toBeNull();
    });
  });
});
