/**
 * Nexus Service Unit Tests
 * 
 * Tests protocol routing, agent registration, and message receiving.
 */
import { Test, TestingModule } from '@nestjs/testing';
import { NexusService } from './nexus.service';

// Mock the bridge module
jest.mock('../../native-bridge', () => ({
  nexusReceive: jest.fn(),
  nexusRegisterAgent: jest.fn(),
  nexusListAgents: jest.fn(),
  nexusGetAgent: jest.fn(),
  nexusUnregisterAgent: jest.fn(),
  nexusDiscoverAgent: jest.fn(),
  nexusRouteTask: jest.fn(),
  nexusGetStats: jest.fn(),
}));

describe('NexusService', () => {
  let service: NexusService;
  let bridgeMock: Record<string, jest.Mock>;

  beforeEach(async () => {
    jest.resetModules();
    
    const module: TestingModule = await Test.createTestingModule({
      providers: [NexusService],
    }).compile();

    service = module.get<NexusService>(NexusService);
    bridgeMock = jest.requireMock('../../native-bridge');
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  describe('initialization', () => {
    it('should be defined', () => {
      expect(service).toBeDefined();
    });

    it('should verify bridge on module init', async () => {
      bridgeMock.nexusListAgents.mockResolvedValue(JSON.stringify([]));
      await expect(service.onModuleInit()).resolves.not.toThrow();
    });
  });

  describe('receive', () => {
    it('should receive and translate message', async () => {
      const mockMessage = {
        id: 'msg-123',
        method: 'task',
        sender: 'agent-a',
        recipient: 'agent-b',
        params: { action: 'hello' },
      };
      bridgeMock.nexusReceive.mockResolvedValue(JSON.stringify(mockMessage));

      const result = await service.receive('{"method":"task"}');

      if (!('error' in result)) {
        expect(result.id).toBe('msg-123');
        expect(result.method).toBe('task');
      }
    });

    it('should handle parsing errors', async () => {
      bridgeMock.nexusReceive.mockImplementation(() => {
        throw new Error('Invalid payload');
      });

      const result = await service.receive('invalid');
      expect('error' in result).toBe(true);
    });
  });

  describe('listAgents', () => {
    it('should return list of registered agents', async () => {
      const mockAgents = [
        { id: 'agent-a', name: 'Agent A', protocols: ['A2A'] },
        { id: 'agent-b', name: 'Agent B', protocols: ['MCP'] },
      ];
      bridgeMock.nexusListAgents.mockResolvedValue(JSON.stringify(mockAgents));

      const result = await service.listAgents();

      expect(result).toHaveLength(2);
      expect(result[0].id).toBe('agent-a');
    });

    it('should return empty array when no agents', async () => {
      bridgeMock.nexusListAgents.mockResolvedValue(JSON.stringify([]));

      const result = await service.listAgents();

      expect(result).toHaveLength(0);
    });
  });

  describe('getAgent', () => {
    it('should return agent by ID', async () => {
      const mockAgent = {
        id: 'agent-123',
        name: 'Test Agent',
        protocols: ['A2A'],
        skills: [{ name: 'search' }],
      };
      bridgeMock.nexusGetAgent.mockResolvedValue(JSON.stringify(mockAgent));

      const result = await service.getAgent('agent-123');

      expect(result?.id).toBe('agent-123');
      expect(result?.name).toBe('Test Agent');
    });

    it('should return null for unknown agent', async () => {
      bridgeMock.nexusGetAgent.mockResolvedValue(JSON.stringify({ error: 'Not found' }));

      const result = await service.getAgent('unknown');

      expect(result).toBeNull();
    });
  });

  describe('registerAgent', () => {
    it('should register agent with protocols', async () => {
      const mockAgent = {
        id: 'agent-new-123',
        name: 'New Agent',
        protocols: ['A2A'],
        skills: [],
        capabilities: [],
        is_local: true,
      };
      bridgeMock.nexusRegisterAgent.mockResolvedValue(JSON.stringify(mockAgent));

      const result = await service.registerAgent({
        name: 'New Agent',
        url: 'https://new-agent.example.com',
        protocols: ['A2A'],
        skills: [],
        capabilities: [],
      });

      expect(result.id).toBeDefined();
      expect(result.name).toBe('New Agent');
    });
  });

  describe('unregisterAgent', () => {
    it('should unregister agent by ID', async () => {
      bridgeMock.nexusUnregisterAgent.mockResolvedValue(true);

      const result = await service.unregisterAgent('agent-123');

      expect(result).toBe(true);
    });

    it('should return false for unknown agent', async () => {
      bridgeMock.nexusUnregisterAgent.mockResolvedValue(false);

      const result = await service.unregisterAgent('unknown');

      expect(result).toBe(false);
    });
  });

  describe('discoverAgent', () => {
    it('should discover agent from URL', async () => {
      const mockAgent = {
        id: 'discovered-agent',
        name: 'Discovered Agent',
        protocols: ['A2A'],
      };
      bridgeMock.nexusDiscoverAgent.mockResolvedValue(JSON.stringify(mockAgent));

      const result = await service.discoverAgent('https://example.com/.well-known/agent.json');

      expect(result.id).toBe('discovered-agent');
    });
  });

  describe('routeTask', () => {
    it('should route task to matching agent', async () => {
      const mockMatch = {
        id: 'agent-best',
        name: 'Best Agent',
        matchScore: 0.95,
      };
      bridgeMock.nexusRouteTask.mockResolvedValue(JSON.stringify(mockMatch));

      const result = await service.routeTask({
        taskType: 'search',
        requiredSkills: ['search'],
      });

      expect(result?.id).toBe('agent-best');
      expect(result?.matchScore).toBe(0.95);
    });

    it('should return null when no match found', async () => {
      bridgeMock.nexusRouteTask.mockResolvedValue(JSON.stringify({ error: 'No match' }));

      const result = await service.routeTask({
        taskType: 'unknown',
        requiredSkills: ['nonexistent'],
      });

      expect(result).toBeNull();
    });
  });
});
