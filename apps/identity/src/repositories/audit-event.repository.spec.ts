/**
 * Audit Event Repository Tests
 * Using mocked TypeORM Repository
 */

import { Test, TestingModule } from '@nestjs/testing';
import { getRepositoryToken } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import { AuditEventRepository } from './audit-event.repository';
import { AuditEventEntity, AuditEventTypeEnum } from '../entities/audit-event.entity';

describe('AuditEventRepository', () => {
  let repo: AuditEventRepository;
  let mockTypeOrmRepo: Partial<Repository<AuditEventEntity>>;

  const mockEvent: Partial<AuditEventEntity> = {
    id: 'event-1',
    type: AuditEventTypeEnum.PROOF_VERIFICATION_SUCCESS,
    timestamp: new Date(),
    principalId: 'principal-1',
    agentId: 'agent-1',
    success: true,
  };

  beforeEach(async () => {
    mockTypeOrmRepo = {
      create: jest.fn().mockReturnValue(mockEvent),
      save: jest.fn().mockResolvedValue(mockEvent),
      find: jest.fn().mockResolvedValue([mockEvent]),
      createQueryBuilder: jest.fn().mockReturnValue({
        where: jest.fn().mockReturnThis(),
        andWhere: jest.fn().mockReturnThis(),
        select: jest.fn().mockReturnThis(),
        getRawOne: jest.fn().mockResolvedValue({
          totalVerifications: '10',
          successfulVerifications: '8',
          failedVerifications: '2',
          revocations: '1',
          securityAlerts: '0',
        }),
      }),
    };

    const module: TestingModule = await Test.createTestingModule({
      providers: [
        AuditEventRepository,
        {
          provide: getRepositoryToken(AuditEventEntity),
          useValue: mockTypeOrmRepo,
        },
      ],
    }).compile();

    repo = module.get<AuditEventRepository>(AuditEventRepository);
  });

  describe('log', () => {
    it('should create and save an audit event', async () => {
      const data = { type: AuditEventTypeEnum.PROOF_VERIFICATION_SUCCESS, success: true };
      const result = await repo.log(data);

      expect(mockTypeOrmRepo.create).toHaveBeenCalledWith(data);
      expect(mockTypeOrmRepo.save).toHaveBeenCalled();
      expect(result).toEqual(mockEvent);
    });
  });

  describe('getRecentEvents', () => {
    it('should return recent events with default limit', async () => {
      const result = await repo.getRecentEvents();

      expect(mockTypeOrmRepo.find).toHaveBeenCalledWith({
        order: { timestamp: 'DESC' },
        take: 100,
      });
      expect(result).toEqual([mockEvent]);
    });

    it('should return recent events with custom limit', async () => {
      await repo.getRecentEvents(50);

      expect(mockTypeOrmRepo.find).toHaveBeenCalledWith({
        order: { timestamp: 'DESC' },
        take: 50,
      });
    });
  });

  describe('getByPrincipal', () => {
    it('should return events for a principal', async () => {
      const result = await repo.getByPrincipal('principal-1');

      expect(mockTypeOrmRepo.find).toHaveBeenCalledWith({
        where: { principalId: 'principal-1' },
        order: { timestamp: 'DESC' },
        take: 100,
      });
      expect(result).toEqual([mockEvent]);
    });

    it('should accept custom limit', async () => {
      await repo.getByPrincipal('principal-1', 25);

      expect(mockTypeOrmRepo.find).toHaveBeenCalledWith({
        where: { principalId: 'principal-1' },
        order: { timestamp: 'DESC' },
        take: 25,
      });
    });
  });

  describe('getByAgent', () => {
    it('should return events for an agent', async () => {
      const result = await repo.getByAgent('agent-1');

      expect(mockTypeOrmRepo.find).toHaveBeenCalledWith({
        where: { agentId: 'agent-1' },
        order: { timestamp: 'DESC' },
        take: 100,
      });
      expect(result).toEqual([mockEvent]);
    });
  });

  describe('getByDateRange', () => {
    it('should return events within date range', async () => {
      const startDate = new Date('2025-01-01');
      const endDate = new Date('2025-12-31');

      await repo.getByDateRange(startDate, endDate);

      expect(mockTypeOrmRepo.find).toHaveBeenCalled();
    });

    it('should filter by event types if provided', async () => {
      const startDate = new Date('2025-01-01');
      const endDate = new Date('2025-12-31');
      const types = [AuditEventTypeEnum.PROOF_VERIFICATION_SUCCESS];

      await repo.getByDateRange(startDate, endDate, types);

      expect(mockTypeOrmRepo.find).toHaveBeenCalled();
    });
  });

  describe('getSecurityEvents', () => {
    it('should return security events', async () => {
      const result = await repo.getSecurityEvents();

      expect(mockTypeOrmRepo.find).toHaveBeenCalled();
      expect(result).toEqual([mockEvent]);
    });

    it('should filter by date if provided', async () => {
      const since = new Date('2025-01-01');
      await repo.getSecurityEvents(since);

      expect(mockTypeOrmRepo.find).toHaveBeenCalled();
    });
  });

  describe('getComplianceData', () => {
    it('should return compliance statistics', async () => {
      const startDate = new Date('2025-01-01');
      const endDate = new Date('2025-12-31');

      const result = await repo.getComplianceData(startDate, endDate);

      expect(result.totalVerifications).toBe(10);
      expect(result.successfulVerifications).toBe(8);
      expect(result.failedVerifications).toBe(2);
      expect(result.revocations).toBe(1);
      expect(result.securityAlerts).toBe(0);
    });

    it('should filter by agentId if provided', async () => {
      const startDate = new Date('2025-01-01');
      const endDate = new Date('2025-12-31');

      await repo.getComplianceData(startDate, endDate, 'agent-1');

      const qb = mockTypeOrmRepo.createQueryBuilder!('ae');
      expect(qb.andWhere).toBeDefined();
    });

    it('should filter by principalId if provided', async () => {
      const startDate = new Date('2025-01-01');
      const endDate = new Date('2025-12-31');

      await repo.getComplianceData(startDate, endDate, undefined, 'principal-1');

      expect(mockTypeOrmRepo.createQueryBuilder).toHaveBeenCalled();
    });
  });
});
