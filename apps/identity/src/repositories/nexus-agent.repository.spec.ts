import { Test, TestingModule } from '@nestjs/testing';
import { getRepositoryToken } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import { NexusAgentRepository, RegisterAgentData } from './nexus-agent.repository';
import { NexusAgentEntity } from '../entities/nexus-agent.entity';

describe('NexusAgentRepository', () => {
  let repository: NexusAgentRepository;
  let typeOrmRepository: jest.Mocked<Repository<NexusAgentEntity>>;

  const mockAgent: Partial<NexusAgentEntity> = {
    id: 'agent-123',
    name: 'Test Agent',
    description: 'A test agent',
    url: 'https://example.com/agent',
    version: '1.0.0',
    capabilities: [{ id: 'cap-1', name: 'Test Capability' }],
    skills: [{ id: 'skill-1', name: 'Test Skill', tags: ['test'] }],
    protocols: ['agentkern'],
    active: true,
    registeredAt: new Date(),
    lastSeenAt: new Date(),
  };

  beforeEach(async () => {
    const mockTypeOrmRepository = {
      create: jest.fn((data) => ({ ...data })),
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
        NexusAgentRepository,
        {
          provide: getRepositoryToken(NexusAgentEntity),
          useValue: mockTypeOrmRepository,
        },
      ],
    }).compile();

    repository = module.get<NexusAgentRepository>(NexusAgentRepository);
    typeOrmRepository = module.get(getRepositoryToken(NexusAgentEntity));
  });

  it('should be defined', () => {
    expect(repository).toBeDefined();
  });

  describe('register', () => {
    it('should create a new agent', async () => {
      const registerData: RegisterAgentData = {
        name: 'New Agent',
        url: 'https://example.com/new-agent',
        description: 'Test description',
      };

      typeOrmRepository.findOne.mockResolvedValue(null);
      typeOrmRepository.save.mockResolvedValue(mockAgent as NexusAgentEntity);

      const result = await repository.register(registerData);

      expect(result).toBeDefined();
      expect(typeOrmRepository.create).toHaveBeenCalled();
      expect(typeOrmRepository.save).toHaveBeenCalled();
    });

    it('should update existing agent when ID matches', async () => {
      const registerData: RegisterAgentData = {
        id: 'agent-123',
        name: 'Updated Agent',
        url: 'https://example.com/updated-agent',
      };

      typeOrmRepository.findOne.mockResolvedValue(mockAgent as NexusAgentEntity);
      typeOrmRepository.save.mockResolvedValue({ ...mockAgent, name: 'Updated Agent' } as NexusAgentEntity);

      const result = await repository.register(registerData);

      expect(result.name).toBe('Updated Agent');
    });

    it('should use default values for optional fields', async () => {
      const registerData: RegisterAgentData = {
        name: 'Minimal Agent',
        url: 'https://example.com/minimal',
      };

      typeOrmRepository.findOne.mockResolvedValue(null);
      typeOrmRepository.save.mockResolvedValue(mockAgent as NexusAgentEntity);

      await repository.register(registerData);

      expect(typeOrmRepository.create).toHaveBeenCalledWith(
        expect.objectContaining({
          version: '1.0.0',
          capabilities: [],
          skills: [],
          protocols: ['agentkern'],
        }),
      );
    });
  });

  describe('findById', () => {
    it('should find active agent by ID', async () => {
      typeOrmRepository.findOne.mockResolvedValue(mockAgent as NexusAgentEntity);

      const result = await repository.findById('agent-123');

      expect(result).toEqual(mockAgent);
      expect(typeOrmRepository.findOne).toHaveBeenCalledWith({
        where: { id: 'agent-123', active: true },
      });
    });

    it('should return null when agent not found', async () => {
      typeOrmRepository.findOne.mockResolvedValue(null);

      const result = await repository.findById('nonexistent');

      expect(result).toBeNull();
    });
  });

  describe('findAll', () => {
    it('should return all active agents', async () => {
      typeOrmRepository.find.mockResolvedValue([mockAgent] as NexusAgentEntity[]);

      const result = await repository.findAll();

      expect(result).toHaveLength(1);
      expect(typeOrmRepository.find).toHaveBeenCalledWith({
        where: { active: true },
        order: { registeredAt: 'DESC' },
      });
    });
  });

  describe('findBySkill', () => {
    it('should find agents by skill using JSONB query', async () => {
      const mockQueryBuilder = {
        where: jest.fn().mockReturnThis(),
        andWhere: jest.fn().mockReturnThis(),
        getMany: jest.fn().mockResolvedValue([mockAgent]),
      };
      typeOrmRepository.createQueryBuilder.mockReturnValue(mockQueryBuilder as any);

      const result = await repository.findBySkill('Test Skill');

      expect(result).toHaveLength(1);
      expect(typeOrmRepository.createQueryBuilder).toHaveBeenCalledWith('agent');
    });
  });

  describe('findByName', () => {
    it('should find active agents by name pattern', async () => {
      typeOrmRepository.find.mockResolvedValue([mockAgent] as NexusAgentEntity[]);

      const result = await repository.findByName('Test');

      expect(result).toHaveLength(1);
    });
  });

  describe('unregister', () => {
    it('should soft-delete agent and return true', async () => {
      typeOrmRepository.update.mockResolvedValue({ affected: 1 } as any);

      const result = await repository.unregister('agent-123');

      expect(result).toBe(true);
      expect(typeOrmRepository.update).toHaveBeenCalledWith(
        { id: 'agent-123' },
        { active: false },
      );
    });

    it('should return false when agent not found', async () => {
      typeOrmRepository.update.mockResolvedValue({ affected: 0 } as any);

      const result = await repository.unregister('nonexistent');

      expect(result).toBe(false);
    });
  });

  describe('delete', () => {
    it('should hard-delete agent and return true', async () => {
      typeOrmRepository.delete.mockResolvedValue({ affected: 1 } as any);

      const result = await repository.delete('agent-123');

      expect(result).toBe(true);
    });

    it('should return false when agent not found', async () => {
      typeOrmRepository.delete.mockResolvedValue({ affected: 0 } as any);

      const result = await repository.delete('nonexistent');

      expect(result).toBe(false);
    });
  });

  describe('getStats', () => {
    it('should return registry statistics', async () => {
      typeOrmRepository.count
        .mockResolvedValueOnce(5)  // active count
        .mockResolvedValueOnce(8); // total count

      const stats = await repository.getStats();

      expect(stats.activeAgents).toBe(5);
      expect(stats.totalAgents).toBe(8);
      expect(stats.inactiveAgents).toBe(3);
    });
  });

  describe('exists', () => {
    it('should return true when agent exists', async () => {
      typeOrmRepository.count.mockResolvedValue(1);

      const result = await repository.exists('agent-123');

      expect(result).toBe(true);
    });

    it('should return false when agent does not exist', async () => {
      typeOrmRepository.count.mockResolvedValue(0);

      const result = await repository.exists('nonexistent');

      expect(result).toBe(false);
    });
  });
});
