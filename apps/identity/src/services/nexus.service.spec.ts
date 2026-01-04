/**
 * Nexus Service Unit Tests
 * 
 * Tests protocol routing, agent registration, and message sending.
 */
import { Test, TestingModule } from '@nestjs/testing';
import { NexusService } from './nexus.service';

// Mock the bridge module
jest.mock('../../native-bridge', () => ({
  nexus_route: jest.fn(),
  nexus_list: jest.fn(),
  nexus_send: jest.fn(),
  nexus_register: jest.fn(),
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
      bridgeMock.nexus_list.mockReturnValue(JSON.stringify({ agents: [] }));
      await expect(service.onModuleInit()).resolves.not.toThrow();
    });
  });

  describe('routeMessage', () => {
    it('should route message to correct protocol', async () => {
      const mockRoute = {
        destination: 'agent-b',
        protocol: 'A2A',
        endpoint: 'https://agent-b.example.com/.well-known/agent.json',
      };
      bridgeMock.nexus_route.mockReturnValue(JSON.stringify(mockRoute));

      const result = await service.routeMessage('agent-a', 'agent-b', { type: 'request' });

      expect(result.protocol).toBe('A2A');
      expect(bridgeMock.nexus_route).toHaveBeenCalled();
    });

    it('should handle routing failure', async () => {
      bridgeMock.nexus_route.mockImplementation(() => {
        throw new Error('No route to destination');
      });

      await expect(
        service.routeMessage('agent-a', 'unknown', { type: 'request' }),
      ).rejects.toThrow('No route to destination');
    });
  });

  describe('listAgents', () => {
    it('should return list of registered agents', async () => {
      const mockAgents = {
        agents: [
          { id: 'agent-a', protocol: 'A2A', status: 'active' },
          { id: 'agent-b', protocol: 'MCP', status: 'active' },
        ],
        total: 2,
      };
      bridgeMock.nexus_list.mockReturnValue(JSON.stringify(mockAgents));

      const result = await service.listAgents();

      expect(result.agents).toHaveLength(2);
      expect(result.agents[0].id).toBe('agent-a');
    });

    it('should filter by protocol', async () => {
      const mockAgents = {
        agents: [{ id: 'agent-a', protocol: 'A2A', status: 'active' }],
        total: 1,
      };
      bridgeMock.nexus_list.mockReturnValue(JSON.stringify(mockAgents));

      const result = await service.listAgents({ protocol: 'A2A' });

      expect(result.agents).toHaveLength(1);
      expect(result.agents[0].protocol).toBe('A2A');
    });
  });

  describe('sendMessage', () => {
    it('should send message successfully', async () => {
      const mockResponse = {
        message_id: 'msg-123',
        status: 'delivered',
        timestamp: Date.now(),
      };
      bridgeMock.nexus_send.mockReturnValue(JSON.stringify(mockResponse));

      const result = await service.sendMessage('agent-a', 'agent-b', {
        type: 'request',
        payload: { action: 'hello' },
      });

      expect(result.message_id).toBe('msg-123');
      expect(result.status).toBe('delivered');
    });

    it('should handle delivery failure', async () => {
      bridgeMock.nexus_send.mockImplementation(() => {
        throw new Error('Delivery failed');
      });

      await expect(
        service.sendMessage('agent-a', 'offline-agent', { type: 'request' }),
      ).rejects.toThrow('Delivery failed');
    });
  });

  describe('registerAgent', () => {
    it('should register agent with A2A protocol', async () => {
      const mockRegistration = {
        agent_id: 'agent-new',
        protocol: 'A2A',
        endpoint: 'https://agent-new.example.com',
        registered_at: Date.now(),
      };
      bridgeMock.nexus_register.mockReturnValue(JSON.stringify(mockRegistration));

      const result = await service.registerAgent({
        id: 'agent-new',
        protocol: 'A2A',
        endpoint: 'https://agent-new.example.com',
      });

      expect(result.agent_id).toBe('agent-new');
    });

    it('should reject duplicate registration', async () => {
      bridgeMock.nexus_register.mockImplementation(() => {
        throw new Error('Agent already registered');
      });

      await expect(
        service.registerAgent({
          id: 'existing-agent',
          protocol: 'A2A',
          endpoint: 'https://existing.example.com',
        }),
      ).rejects.toThrow('Agent already registered');
    });

    it('should validate protocol type', async () => {
      await expect(
        service.registerAgent({
          id: 'agent-new',
          protocol: 'INVALID_PROTOCOL' as 'A2A',
          endpoint: 'https://agent-new.example.com',
        }),
      ).rejects.toThrow();
    });
  });
});
