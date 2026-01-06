import { Test, TestingModule } from '@nestjs/testing';
import { getRepositoryToken } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import { DnsResolutionService } from './dns-resolution.service';
import { AuditLoggerService, AuditEventType } from './audit-logger.service';
import { TrustRecordEntity } from '../entities/trust-record.entity';

describe('DnsResolutionService', () => {
  let service: DnsResolutionService;
  let trustRepository: jest.Mocked<Repository<TrustRecordEntity>>;
  let auditLogger: jest.Mocked<AuditLoggerService>;

  const mockTrustRecord = {
    id: '123e4567-e89b-12d3-a456-426614174000',
    agentId: 'agent-123',
    principalId: 'principal-456',
    trustScore: 500,
    trusted: true,
    revoked: false,
    verificationCount: 10,
    failureCount: 1,
    registeredAt: new Date('2025-01-01'),
    lastVerifiedAt: new Date('2025-12-01'),
    expiresAt: new Date('2026-12-31'),
    createdAt: new Date(),
    updatedAt: new Date(),
  };

  beforeEach(async () => {
    const mockTrustRepository = {
      findOne: jest.fn(),
      find: jest.fn(),
      save: jest.fn(),
      create: jest.fn((data) => data),
    };

    const mockAuditLogger = {
      log: jest.fn().mockResolvedValue(undefined),
      logSecurityEvent: jest.fn().mockResolvedValue(undefined),
    };

    const module: TestingModule = await Test.createTestingModule({
      providers: [
        DnsResolutionService,
        {
          provide: getRepositoryToken(TrustRecordEntity),
          useValue: mockTrustRepository,
        },
        {
          provide: AuditLoggerService,
          useValue: mockAuditLogger,
        },
      ],
    }).compile();

    service = module.get<DnsResolutionService>(DnsResolutionService);
    trustRepository = module.get(getRepositoryToken(TrustRecordEntity));
    auditLogger = module.get(AuditLoggerService);
  });

  it('should be defined', () => {
    expect(service).toBeDefined();
  });

  describe('resolve', () => {
    it('should create a new trust record if none exists', async () => {
      trustRepository.findOne.mockResolvedValue(null);
      trustRepository.save.mockResolvedValue(mockTrustRecord as TrustRecordEntity);

      const result = await service.resolve({
        agentId: 'agent-123',
        principalId: 'principal-456',
      });

      expect(result).toBeDefined();
      expect(result.agentId).toBe('agent-123');
      expect(trustRepository.save).toHaveBeenCalled();
    });

    it('should return cached resolution on cache hit', async () => {
      // First call - cache miss
      trustRepository.findOne.mockResolvedValue(mockTrustRecord as TrustRecordEntity);
      const firstResult = await service.resolve({
        agentId: 'agent-123',
        principalId: 'principal-456',
      });

      // Reset mock call count
      trustRepository.findOne.mockClear();

      // Second call - should hit cache
      const secondResult = await service.resolve({
        agentId: 'agent-123',
        principalId: 'principal-456',
      });

      expect(secondResult).toEqual(firstResult);
      // Should not hit database on cached query
      expect(trustRepository.findOne).not.toHaveBeenCalled();
    });

    it('should return existing trust record if found', async () => {
      trustRepository.findOne.mockResolvedValue(mockTrustRecord as TrustRecordEntity);

      const result = await service.resolve({
        agentId: 'agent-123',
        principalId: 'principal-456',
      });

      expect(result.trusted).toBe(true);
      expect(result.trustScore).toBe(mockTrustRecord.trustScore);
    });
  });

  describe('resolveBatch', () => {
    it('should resolve multiple queries in parallel', async () => {
      trustRepository.findOne.mockResolvedValue(mockTrustRecord as TrustRecordEntity);

      const queries = [
        { agentId: 'agent-1', principalId: 'principal-1' },
        { agentId: 'agent-2', principalId: 'principal-2' },
        { agentId: 'agent-3', principalId: 'principal-3' },
      ];

      const results = await service.resolveBatch(queries);

      expect(results).toHaveLength(3);
      expect(results[0]).toBeDefined();
    });
  });

  describe('registerTrust', () => {
    it('should create a new trust record with metadata', async () => {
      trustRepository.save.mockResolvedValue(mockTrustRecord as TrustRecordEntity);

      const result = await service.registerTrust('agent-new', 'principal-new', {
        agentName: 'TestAgent',
        agentVersion: '1.0.0',
        principalDevice: 'device-123',
      });

      expect(result).toBeDefined();
      expect(trustRepository.save).toHaveBeenCalled();
    });

    it('should invalidate cache after registration', async () => {
      // Populate cache
      trustRepository.findOne.mockResolvedValue(mockTrustRecord as TrustRecordEntity);
      await service.resolve({ agentId: 'agent-123', principalId: 'principal-456' });

      // Register new trust (should invalidate cache)
      trustRepository.save.mockResolvedValue(mockTrustRecord as TrustRecordEntity);
      await service.registerTrust('agent-123', 'principal-456');

      // Next resolve should hit database
      trustRepository.findOne.mockClear();
      await service.resolve({ agentId: 'agent-123', principalId: 'principal-456' });

      expect(trustRepository.findOne).toHaveBeenCalled();
    });
  });

  describe('recordVerificationSuccess', () => {
    it('should increment verification count and recalculate score', async () => {
      const recordCopy = { ...mockTrustRecord };
      trustRepository.findOne.mockResolvedValue(recordCopy as TrustRecordEntity);
      trustRepository.save.mockResolvedValue(recordCopy as TrustRecordEntity);

      const result = await service.recordVerificationSuccess('agent-123', 'principal-456');

      expect(result).toBeDefined();
      expect(recordCopy.verificationCount).toBe(11); // Incremented
      expect(trustRepository.save).toHaveBeenCalled();
    });

    it('should return null if record does not exist', async () => {
      trustRepository.findOne.mockResolvedValue(null);

      const result = await service.recordVerificationSuccess('nonexistent', 'user');

      expect(result).toBeNull();
    });
  });

  describe('recordVerificationFailure', () => {
    it('should increment failure count and log security event', async () => {
      const recordCopy = { ...mockTrustRecord };
      trustRepository.findOne.mockResolvedValue(recordCopy as TrustRecordEntity);
      trustRepository.save.mockResolvedValue(recordCopy as TrustRecordEntity);

      const result = await service.recordVerificationFailure('agent-123', 'principal-456');

      expect(result).toBeDefined();
      expect(recordCopy.failureCount).toBe(2); // Incremented
      expect(auditLogger.logSecurityEvent).toHaveBeenCalledWith(
        AuditEventType.PROOF_VERIFICATION_FAILURE,
        expect.any(String),
        expect.any(Object),
      );
    });

    it('should return null if record does not exist', async () => {
      trustRepository.findOne.mockResolvedValue(null);

      const result = await service.recordVerificationFailure('nonexistent', 'user');

      expect(result).toBeNull();
    });
  });

  describe('revokeTrust', () => {
    it('should set revoked flag and decrease trust score', async () => {
      const recordCopy = { ...mockTrustRecord, trustScore: 500 };
      trustRepository.findOne.mockResolvedValue(recordCopy as TrustRecordEntity);
      trustRepository.save.mockResolvedValue(recordCopy as TrustRecordEntity);

      const result = await service.revokeTrust('agent-123', 'principal-456', 'Policy violation');

      expect(result).toBeDefined();
      expect(recordCopy.revoked).toBe(true);
      expect(recordCopy.trusted).toBe(false);
      expect(recordCopy.trustScore).toBe(300); // 500 - 200
      expect(auditLogger.log).toHaveBeenCalledWith(
        expect.objectContaining({
          type: AuditEventType.KEY_REVOKED,
        }),
      );
    });

    it('should not go below zero trust score', async () => {
      const recordCopy = { ...mockTrustRecord, trustScore: 100 };
      trustRepository.findOne.mockResolvedValue(recordCopy as TrustRecordEntity);
      trustRepository.save.mockResolvedValue(recordCopy as TrustRecordEntity);

      await service.revokeTrust('agent-123', 'principal-456', 'Zero test');

      expect(recordCopy.trustScore).toBe(0); // Max(0, 100-200)
    });

    it('should return null if record does not exist', async () => {
      trustRepository.findOne.mockResolvedValue(null);

      const result = await service.revokeTrust('nonexistent', 'user', 'test');

      expect(result).toBeNull();
    });
  });

  describe('reinstateTrust', () => {
    it('should clear revoked flag and recalculate trusted status', async () => {
      const recordCopy = { ...mockTrustRecord, revoked: true, trusted: false };
      trustRepository.findOne.mockResolvedValue(recordCopy as TrustRecordEntity);
      trustRepository.save.mockResolvedValue(recordCopy as TrustRecordEntity);

      const result = await service.reinstateTrust('agent-123', 'principal-456');

      expect(result).toBeDefined();
      expect(recordCopy.revoked).toBe(false);
    });

    it('should return null if record does not exist', async () => {
      trustRepository.findOne.mockResolvedValue(null);

      const result = await service.reinstateTrust('nonexistent', 'user');

      expect(result).toBeNull();
    });
  });

  describe('getTrustRecordsForPrincipal', () => {
    it('should return all trust records for a principal', async () => {
      trustRepository.find.mockResolvedValue([
        mockTrustRecord,
        { ...mockTrustRecord, agentId: 'agent-789' },
      ] as TrustRecordEntity[]);

      const results = await service.getTrustRecordsForPrincipal('principal-456');

      expect(results).toHaveLength(2);
      expect(trustRepository.find).toHaveBeenCalledWith({
        where: { principalId: 'principal-456' },
      });
    });
  });

  describe('getTrustRecord', () => {
    it('should return a specific trust record', async () => {
      trustRepository.findOne.mockResolvedValue(mockTrustRecord as TrustRecordEntity);

      const result = await service.getTrustRecord('agent-123', 'principal-456');

      expect(result).toEqual(mockTrustRecord);
    });

    it('should return null if record does not exist', async () => {
      trustRepository.findOne.mockResolvedValue(null);

      const result = await service.getTrustRecord('nonexistent', 'user');

      expect(result).toBeNull();
    });
  });
});
