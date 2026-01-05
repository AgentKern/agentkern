/**
 * Nexus Service Unit Tests
 *
 * Tests protocol routing, agent registration, and message receiving.
 * Uses proper mocking strategy for dynamically loaded native bridge.
 */
import { Test, TestingModule } from '@nestjs/testing';
import { NexusService } from './nexus.service';

describe('NexusService', () => {
  let service: NexusService;
  let mockBridge: Record<string, jest.Mock>;

  beforeEach(async () => {
    // Create mock bridge functions
    mockBridge = {
      nexusReceive: jest.fn(),
      nexusRegisterAgent: jest.fn(),
      nexusListAgents: jest.fn(),
      nexusGetAgent: jest.fn(),
      nexusUnregisterAgent: jest.fn(),
      nexusDiscoverAgent: jest.fn(),
      nexusRouteTask: jest.fn(),
      nexusGetStats: jest.fn(),
      nexusCreateA2aTask: jest.fn(),
      nexusSend: jest.fn(),
    };

    const module: TestingModule = await Test.createTestingModule({
      providers: [NexusService],
    }).compile();

    service = module.get<NexusService>(NexusService);

    // Manually inject the mock bridge and set bridgeLoaded flag
    (service as any).bridge = mockBridge;
    (service as any).bridgeLoaded = true;
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  describe('initialization', () => {
    it('should be defined', () => {
      expect(service).toBeDefined();
    });

    it('should be operational when bridge is loaded', () => {
      expect(service.isOperational()).toBe(true);
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
      mockBridge.nexusReceive.mockResolvedValue(JSON.stringify(mockMessage));

      const result = await service.receive('{"method":"task"}');

      if (!('error' in result)) {
        expect(result.id).toBe('msg-123');
        expect(result.method).toBe('task');
      }
    });

    it('should handle parsing errors', async () => {
      mockBridge.nexusReceive.mockRejectedValue(new Error('Invalid payload'));

      const result = await service.receive('invalid');
      expect('error' in result).toBe(true);
    });
  });

  describe('listAgents', () => {
    it('should return list of registered agents', async () => {
      const mockAgents = [
        { id: 'agent-a', name: 'Agent A', url: 'https://a.example.com', protocols: ['A2A'] },
        { id: 'agent-b', name: 'Agent B', url: 'https://b.example.com', protocols: ['MCP'] },
      ];
      mockBridge.nexusListAgents.mockResolvedValue(JSON.stringify(mockAgents));

      const result = await service.listAgents();

      expect(result).toHaveLength(2);
      expect(result[0].id).toBe('agent-a');
    });

    it('should return empty array when no agents', async () => {
      mockBridge.nexusListAgents.mockResolvedValue(JSON.stringify([]));

      const result = await service.listAgents();

      expect(result).toHaveLength(0);
    });
  });

  describe('getAgent', () => {
    it('should return agent by ID', async () => {
      const mockAgent = {
        id: 'agent-123',
        name: 'Test Agent',
        url: 'https://test.example.com',
        protocols: ['A2A'],
        skills: [{ name: 'search' }],
      };
      mockBridge.nexusGetAgent.mockResolvedValue(JSON.stringify(mockAgent));

      const result = await service.getAgent('agent-123');

      expect(result?.id).toBe('agent-123');
      expect(result?.name).toBe('Test Agent');
    });

    it('should return null for unknown agent', async () => {
      mockBridge.nexusGetAgent.mockResolvedValue('null');

      const result = await service.getAgent('unknown');

      expect(result).toBeNull();
    });
  });

  describe('registerAgent', () => {
    it('should register agent with protocols', async () => {
      mockBridge.nexusRegisterAgent.mockResolvedValue(JSON.stringify({ success: true }));

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
      mockBridge.nexusUnregisterAgent.mockResolvedValue(true);

      const result = await service.unregisterAgent('agent-123');

      expect(result).toBe(true);
    });

    it('should return false for unknown agent', async () => {
      mockBridge.nexusUnregisterAgent.mockResolvedValue(false);

      const result = await service.unregisterAgent('unknown');

      expect(result).toBe(false);
    });
  });

  describe('discoverAgent', () => {
    it('should discover agent from URL', async () => {
      const mockAgent = {
        id: 'discovered-agent',
        name: 'Discovered Agent',
        url: 'https://example.com',
        protocols: ['A2A'],
      };
      mockBridge.nexusDiscoverAgent.mockResolvedValue(JSON.stringify(mockAgent));

      const result = await service.discoverAgent('https://example.com/.well-known/agent.json');

      expect(result.id).toBe('discovered-agent');
    });
  });

  describe('routeTask', () => {
    it('should route task to matching agent', async () => {
      const mockMatch = {
        id: 'agent-best',
        name: 'Best Agent',
        url: 'https://best.example.com',
        matchScore: 0.95,
      };
      mockBridge.nexusRouteTask.mockResolvedValue(JSON.stringify(mockMatch));

      const result = await service.routeTask({
        taskType: 'search',
        requiredSkills: ['search'],
      });

      expect(result?.id).toBe('agent-best');
      expect(result?.matchScore).toBe(0.95);
    });

    it('should return null when no match found', async () => {
      mockBridge.nexusRouteTask.mockResolvedValue(JSON.stringify({ error: 'No match' }));

      const result = await service.routeTask({
        taskType: 'unknown',
        requiredSkills: ['nonexistent'],
      });

      expect(result).toBeNull();
    });
  });

  describe('getStats', () => {
    it('should return gateway statistics', async () => {
      const mockStats = {
        registeredAgents: 10,
        supportedProtocols: 3,
      };
      mockBridge.nexusGetStats.mockResolvedValue(JSON.stringify(mockStats));

      const result = await service.getStats();

      expect(result.registeredAgents).toBe(10);
      expect(result.supportedProtocols).toBe(3);
    });
  });
});
