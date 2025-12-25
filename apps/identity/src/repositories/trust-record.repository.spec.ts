/**
 * Trust Record Repository Tests
 * Using mocked TypeORM Repository
 */

import { Test, TestingModule } from '@nestjs/testing';
import { getRepositoryToken } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import { TrustRecordRepository } from './trust-record.repository';
import { TrustRecordEntity } from '../entities/trust-record.entity';

describe('TrustRecordRepository', () => {
  let repo: TrustRecordRepository;
  let mockTypeOrmRepo: Partial<Repository<TrustRecordEntity>>;

  const mockRecord: Partial<TrustRecordEntity> = {
    id: 'record-1',
    agentId: 'agent-1',
    principalId: 'principal-1',
    trustScore: 750,
    trusted: true,
    revoked: false,
    verificationCount: 10,
    failureCount: 0,
  };

  beforeEach(async () => {
    mockTypeOrmRepo = {
      create: jest.fn().mockReturnValue(mockRecord),
      save: jest.fn().mockResolvedValue(mockRecord),
      findOne: jest.fn().mockResolvedValue(mockRecord),
      find: jest.fn().mockResolvedValue([mockRecord]),
      createQueryBuilder: jest.fn().mockReturnValue({
        select: jest.fn().mockReturnThis(),
        addSelect: jest.fn().mockReturnThis(),
        getRawOne: jest.fn().mockResolvedValue({
          totalRecords: '100',
          trustedCount: '90',
          revokedCount: '5',
          avgScore: '750',
        }),
      }),
    };

    const module: TestingModule = await Test.createTestingModule({
      providers: [
        TrustRecordRepository,
        {
          provide: getRepositoryToken(TrustRecordEntity),
          useValue: mockTypeOrmRepo,
        },
      ],
    }).compile();

    repo = module.get<TrustRecordRepository>(TrustRecordRepository);
  });

  describe('findByAgentAndPrincipal', () => {
    it('should find a trust record by agent and principal', async () => {
      const result = await repo.findByAgentAndPrincipal('agent-1', 'principal-1');

      expect(mockTypeOrmRepo.findOne).toHaveBeenCalledWith({
        where: { agentId: 'agent-1', principalId: 'principal-1' },
      });
      expect(result).toEqual(mockRecord);
    });
  });

  describe('findByPrincipal', () => {
    it('should find all trust records for a principal', async () => {
      const result = await repo.findByPrincipal('principal-1');

      expect(mockTypeOrmRepo.find).toHaveBeenCalledWith({
        where: { principalId: 'principal-1' },
        order: { trustScore: 'DESC' },
      });
      expect(result).toEqual([mockRecord]);
    });
  });

  describe('upsert', () => {
    it('should update existing record', async () => {
      const result = await repo.upsert({ agentId: 'agent-1', principalId: 'principal-1', trustScore: 800 });

      expect(mockTypeOrmRepo.findOne).toHaveBeenCalled();
      expect(mockTypeOrmRepo.save).toHaveBeenCalled();
      expect(result).toEqual(mockRecord);
    });

    it('should create new record if not exists', async () => {
      (mockTypeOrmRepo.findOne as jest.Mock).mockResolvedValue(null);

      await repo.upsert({ agentId: 'agent-new', principalId: 'principal-new' });

      expect(mockTypeOrmRepo.create).toHaveBeenCalled();
      expect(mockTypeOrmRepo.save).toHaveBeenCalled();
    });
  });

  describe('recordVerificationSuccess', () => {
    it('should increment verification count', async () => {
      const result = await repo.recordVerificationSuccess('agent-1', 'principal-1');

      expect(mockTypeOrmRepo.findOne).toHaveBeenCalled();
      expect(mockTypeOrmRepo.save).toHaveBeenCalled();
    });

    it('should return null if record not found', async () => {
      (mockTypeOrmRepo.findOne as jest.Mock).mockResolvedValue(null);

      const result = await repo.recordVerificationSuccess('unknown', 'unknown');

      expect(result).toBeNull();
    });
  });

  describe('recordVerificationFailure', () => {
    it('should increment failure count', async () => {
      const result = await repo.recordVerificationFailure('agent-1', 'principal-1');

      expect(mockTypeOrmRepo.findOne).toHaveBeenCalled();
      expect(mockTypeOrmRepo.save).toHaveBeenCalled();
    });

    it('should return null if record not found', async () => {
      (mockTypeOrmRepo.findOne as jest.Mock).mockResolvedValue(null);

      const result = await repo.recordVerificationFailure('unknown', 'unknown');

      expect(result).toBeNull();
    });
  });

  describe('revoke', () => {
    it('should revoke trust record', async () => {
      const result = await repo.revoke('agent-1', 'principal-1');

      expect(mockTypeOrmRepo.findOne).toHaveBeenCalled();
      expect(mockTypeOrmRepo.save).toHaveBeenCalled();
    });

    it('should return null if record not found', async () => {
      (mockTypeOrmRepo.findOne as jest.Mock).mockResolvedValue(null);

      const result = await repo.revoke('unknown', 'unknown');

      expect(result).toBeNull();
    });
  });

  describe('reinstate', () => {
    it('should reinstate revoked trust record', async () => {
      const revokedRecord = { ...mockRecord, revoked: true };
      (mockTypeOrmRepo.findOne as jest.Mock).mockResolvedValue(revokedRecord);

      const result = await repo.reinstate('agent-1', 'principal-1');

      expect(mockTypeOrmRepo.save).toHaveBeenCalled();
    });

    it('should return null if record not found', async () => {
      (mockTypeOrmRepo.findOne as jest.Mock).mockResolvedValue(null);

      const result = await repo.reinstate('unknown', 'unknown');

      expect(result).toBeNull();
    });
  });

  describe('getStats', () => {
    it('should return repository statistics', async () => {
      const result = await repo.getStats();

      expect(result.totalRecords).toBe(100);
      expect(result.trustedCount).toBe(90);
      expect(result.revokedCount).toBe(5);
      expect(result.avgScore).toBe(750);
    });
  });
});
