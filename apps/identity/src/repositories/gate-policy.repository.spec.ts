import { Test, TestingModule } from '@nestjs/testing';
import { getRepositoryToken } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import { GatePolicyRepository, CreatePolicyData } from './gate-policy.repository';
import { GatePolicyEntity, PolicyAction } from '../entities/gate-policy.entity';

describe('GatePolicyRepository', () => {
  let repository: GatePolicyRepository;
  let typeOrmRepository: jest.Mocked<Repository<GatePolicyEntity>>;

  const mockPolicy: Partial<GatePolicyEntity> = {
    id: 'policy-123',
    name: 'Test Policy',
    description: 'A test security policy',
    rules: [
      { id: 'rule-1', condition: 'test', action: 'deny' as PolicyAction, priority: 100 },
    ],
    tags: ['test', 'security'],
    active: true,
    version: 1,
    createdBy: 'test-user',
    createdAt: new Date(),
    updatedAt: new Date(),
  };

  beforeEach(async () => {
    const mockTypeOrmRepository = {
      create: jest.fn((data) => ({ ...data, id: data.id || 'new-id' })),
      save: jest.fn((data) => Promise.resolve(data)),
      findOne: jest.fn(),
      find: jest.fn(),
      update: jest.fn(),
      delete: jest.fn(),
      count: jest.fn(),
      createQueryBuilder: jest.fn().mockReturnValue({
        where: jest.fn().mockReturnThis(),
        andWhere: jest.fn().mockReturnThis(),
        getMany: jest.fn().mockResolvedValue([]),
      }),
    };

    const module: TestingModule = await Test.createTestingModule({
      providers: [
        GatePolicyRepository,
        {
          provide: getRepositoryToken(GatePolicyEntity),
          useValue: mockTypeOrmRepository,
        },
      ],
    }).compile();

    repository = module.get<GatePolicyRepository>(GatePolicyRepository);
    typeOrmRepository = module.get(getRepositoryToken(GatePolicyEntity));
  });

  it('should be defined', () => {
    expect(repository).toBeDefined();
  });

  describe('create', () => {
    it('should create a new policy', async () => {
      const createData: CreatePolicyData = {
        name: 'New Policy',
        description: 'Test description',
        rules: [{ id: 'rule-1', condition: 'test', action: 'deny' as PolicyAction, priority: 100 }],
        tags: ['test'],
        createdBy: 'user-123',
      };

      typeOrmRepository.save.mockResolvedValue(mockPolicy as GatePolicyEntity);

      const result = await repository.create(createData);

      expect(result).toBeDefined();
      expect(typeOrmRepository.create).toHaveBeenCalled();
      expect(typeOrmRepository.save).toHaveBeenCalled();
    });

    it('should create policy with default empty tags', async () => {
      const createData: CreatePolicyData = {
        name: 'Policy Without Tags',
        rules: [],
      };

      typeOrmRepository.save.mockResolvedValue(mockPolicy as GatePolicyEntity);

      await repository.create(createData);

      expect(typeOrmRepository.create).toHaveBeenCalledWith(
        expect.objectContaining({ tags: [] }),
      );
    });
  });

  describe('findById', () => {
    it('should find policy by ID', async () => {
      typeOrmRepository.findOne.mockResolvedValue(mockPolicy as GatePolicyEntity);

      const result = await repository.findById('policy-123');

      expect(result).toEqual(mockPolicy);
      expect(typeOrmRepository.findOne).toHaveBeenCalledWith({
        where: { id: 'policy-123' },
      });
    });

    it('should return null when policy not found', async () => {
      typeOrmRepository.findOne.mockResolvedValue(null);

      const result = await repository.findById('nonexistent');

      expect(result).toBeNull();
    });
  });

  describe('findAll', () => {
    it('should return all policies when activeOnly is false', async () => {
      typeOrmRepository.find.mockResolvedValue([mockPolicy] as GatePolicyEntity[]);

      const result = await repository.findAll(false);

      expect(result).toHaveLength(1);
      expect(typeOrmRepository.find).toHaveBeenCalledWith({
        order: { createdAt: 'DESC' },
      });
    });

    it('should return only active policies when activeOnly is true', async () => {
      typeOrmRepository.find.mockResolvedValue([mockPolicy] as GatePolicyEntity[]);

      await repository.findAll(true);

      expect(typeOrmRepository.find).toHaveBeenCalledWith({
        where: { active: true },
        order: { createdAt: 'DESC' },
      });
    });
  });

  describe('findByTag', () => {
    it('should find policies by tag using query builder', async () => {
      const mockQueryBuilder = {
        where: jest.fn().mockReturnThis(),
        andWhere: jest.fn().mockReturnThis(),
        getMany: jest.fn().mockResolvedValue([mockPolicy]),
      };
      typeOrmRepository.createQueryBuilder.mockReturnValue(mockQueryBuilder as any);

      const result = await repository.findByTag('security');

      expect(result).toHaveLength(1);
      expect(typeOrmRepository.createQueryBuilder).toHaveBeenCalledWith('policy');
    });
  });

  describe('findByName', () => {
    it('should find policies by name pattern', async () => {
      typeOrmRepository.find.mockResolvedValue([mockPolicy] as GatePolicyEntity[]);

      const result = await repository.findByName('Test');

      expect(result).toHaveLength(1);
    });
  });

  describe('update', () => {
    it('should update policy and increment version', async () => {
      typeOrmRepository.findOne.mockResolvedValue({ ...mockPolicy, version: 1 } as GatePolicyEntity);
      typeOrmRepository.save.mockResolvedValue({ ...mockPolicy, version: 2 } as GatePolicyEntity);

      const result = await repository.update('policy-123', { name: 'Updated Name' });

      expect(result).toBeDefined();
      expect(result!.version).toBe(2);
    });

    it('should return null when policy not found', async () => {
      typeOrmRepository.findOne.mockResolvedValue(null);

      const result = await repository.update('nonexistent', { name: 'Test' });

      expect(result).toBeNull();
    });
  });

  describe('deactivate', () => {
    it('should deactivate policy and return true', async () => {
      typeOrmRepository.update.mockResolvedValue({ affected: 1 } as any);

      const result = await repository.deactivate('policy-123');

      expect(result).toBe(true);
      expect(typeOrmRepository.update).toHaveBeenCalledWith(
        { id: 'policy-123' },
        { active: false },
      );
    });

    it('should return false when policy not found', async () => {
      typeOrmRepository.update.mockResolvedValue({ affected: 0 } as any);

      const result = await repository.deactivate('nonexistent');

      expect(result).toBe(false);
    });
  });

  describe('activate', () => {
    it('should activate policy and return true', async () => {
      typeOrmRepository.update.mockResolvedValue({ affected: 1 } as any);

      const result = await repository.activate('policy-123');

      expect(result).toBe(true);
    });
  });

  describe('delete', () => {
    it('should delete policy and return true', async () => {
      typeOrmRepository.delete.mockResolvedValue({ affected: 1 } as any);

      const result = await repository.delete('policy-123');

      expect(result).toBe(true);
    });

    it('should return false when policy not found', async () => {
      typeOrmRepository.delete.mockResolvedValue({ affected: 0 } as any);

      const result = await repository.delete('nonexistent');

      expect(result).toBe(false);
    });
  });

  describe('getStats', () => {
    it('should return policy statistics', async () => {
      const policies = [
        { ...mockPolicy, active: true, rules: [{ id: '1' }, { id: '2' }] },
        { ...mockPolicy, id: 'policy-2', active: false, rules: [{ id: '3' }] },
      ];
      typeOrmRepository.find.mockResolvedValue(policies as any);

      const stats = await repository.getStats();

      expect(stats.totalPolicies).toBe(2);
      expect(stats.activePolicies).toBe(1);
      expect(stats.totalRules).toBe(3);
    });
  });

  describe('seedDefaults', () => {
    it('should not seed when policies exist', async () => {
      typeOrmRepository.count.mockResolvedValue(5);

      await repository.seedDefaults();

      expect(typeOrmRepository.save).not.toHaveBeenCalled();
    });

    it('should seed default policies when none exist', async () => {
      typeOrmRepository.count.mockResolvedValue(0);
      typeOrmRepository.save.mockResolvedValue(mockPolicy as GatePolicyEntity);

      await repository.seedDefaults();

      expect(typeOrmRepository.create).toHaveBeenCalled();
      expect(typeOrmRepository.save).toHaveBeenCalled();
    });
  });
});
