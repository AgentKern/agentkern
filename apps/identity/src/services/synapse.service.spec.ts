import { Test, TestingModule } from '@nestjs/testing';
import { SynapseService, AgentState, StateUpdateResult } from './synapse.service';

// Mock fs module
jest.mock('fs', () => ({
  existsSync: jest.fn().mockReturnValue(false),
}));

describe('SynapseService', () => {
  let service: SynapseService;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [SynapseService],
    }).compile();

    service = module.get<SynapseService>(SynapseService);
  });

  describe('initialization', () => {
    it('should stay in degraded mode if bridge files missing', async () => {
       (require('fs').existsSync as jest.Mock).mockReturnValue(false);
       const logSpy = jest.spyOn((service as any).logger, 'warn').mockImplementation();
       const errorSpy = jest.spyOn((service as any).logger, 'error').mockImplementation();
       
       // Force init
       await service.onModuleInit();

       expect((service as any).bridgeLoaded).toBe(false);
       // Should verify it logged warning or error
       expect(logSpy).toHaveBeenCalledWith(expect.stringContaining('degraded mode'));
    });

    it('should verify bridge is operational if loaded', async () => {
       const mockBridge = { synapseGetState: jest.fn() };
       (service as any).bridge = mockBridge;
       
       mockBridge.synapseGetState.mockResolvedValue(JSON.stringify({ version: 1 }));
       
       // Call private method
       await (service as any).verifyBridge();
       
       expect(mockBridge.synapseGetState).toHaveBeenCalledWith('test-verify');
    });

    it('should throw if bridge verification fails', async () => {
       const mockBridge = { synapseGetState: jest.fn() };
       (service as any).bridge = mockBridge;
       
       mockBridge.synapseGetState.mockResolvedValue(null);
       
       await expect((service as any).verifyBridge()).rejects.toThrow('Bridge verification failed');
    });

    it('should throw if bridge verification returns invalid json', async () => {
       const mockBridge = { synapseGetState: jest.fn() };
       (service as any).bridge = mockBridge;
       
       mockBridge.synapseGetState.mockResolvedValue('invalid-json');
       
       await expect((service as any).verifyBridge()).rejects.toThrow();
    });
  });

  describe('isOperational', () => {
    it('should return false when bridge is not loaded', () => {
      expect(service.isOperational()).toBe(false);
    });
  });

  describe('getState (degraded mode)', () => {
    it('should return null when bridge is not loaded', async () => {
      // Ensure bridge not loaded
      (service as any).bridgeLoaded = false;
      const logSpy = jest.spyOn((service as any).logger, 'warn').mockImplementation();
      
      const result = await service.getState('agent-123');
      
      expect(result).toBeNull();
      expect(logSpy).toHaveBeenCalledWith(expect.stringContaining('degraded mode'));
    });
  });

  describe('updateState (degraded mode)', () => {
    it('should return error when bridge is not loaded', async () => {
      const result = await service.updateState('agent-123', { key: 'value' });
      
      expect(result.success).toBe(false);
      expect(result.error).toBe('Bridge not loaded');
    });
  });

  describe('deleteKeys (degraded mode)', () => {
    it('should return error when bridge is not loaded', async () => {
      const result = await service.deleteKeys('agent-123', ['key1', 'key2']);
      
      expect(result.success).toBe(false);
      expect(result.error).toBe('Bridge not loaded');
    });
  });

  describe('storeMemory (degraded mode)', () => {
    it('should return error when bridge is not loaded', async () => {
      const result = await service.storeMemory('agent-123', 'test memory');
      
      expect(result.error).toBe('Bridge not loaded');
      expect(result.id).toBeUndefined();
    });
  });

  describe('queryMemory (degraded mode)', () => {
    it('should return empty array when bridge is not loaded', async () => {
      const result = await service.queryMemory('test query', 5);
      
      expect(result).toEqual([]);
    });
  });
});

describe('SynapseService (with mock bridge)', () => {
  let service: SynapseService;
  let mockBridge: {
    synapseGetState: jest.Mock;
    synapseUpdateState: jest.Mock;
    synapseStoreMemory: jest.Mock;
    synapseQueryMemory: jest.Mock;
  };

  beforeEach(async () => {
    mockBridge = {
      synapseGetState: jest.fn(),
      synapseUpdateState: jest.fn(),
      synapseStoreMemory: jest.fn(),
      synapseQueryMemory: jest.fn(),
    };

    const module: TestingModule = await Test.createTestingModule({
      providers: [SynapseService],
    }).compile();

    service = module.get<SynapseService>(SynapseService);

    // Manually inject bridge and set loaded flag using reflection
    (service as any).bridge = mockBridge;
    (service as any).bridgeLoaded = true;
  });

  describe('getState', () => {
    it('should return parsed agent state from bridge', async () => {
      const mockState: AgentState = {
        agent_id: 'agent-123',
        state: { key: 'value' },
        version: 1,
      };
      mockBridge.synapseGetState.mockResolvedValue(JSON.stringify(mockState));

      const result = await service.getState('agent-123');

      expect(result).toEqual(mockState);
      expect(mockBridge.synapseGetState).toHaveBeenCalledWith('agent-123');
    });

    it('should return null on bridge error', async () => {
      mockBridge.synapseGetState.mockRejectedValue(new Error('Bridge error'));

      const result = await service.getState('agent-123');

      expect(result).toBeNull();
    });
  });

  describe('updateState', () => {
    it('should return success result from bridge', async () => {
      mockBridge.synapseUpdateState.mockResolvedValue(
        JSON.stringify({ version: 2 }),
      );

      const result = await service.updateState('agent-123', { key: 'value' });

      expect(result.success).toBe(true);
      expect(result.version).toBe(2);
    });

    it('should return error when bridge returns error', async () => {
      mockBridge.synapseUpdateState.mockResolvedValue(
        JSON.stringify({ error: 'Conflict' }),
      );

      const result = await service.updateState('agent-123', { key: 'value' });

      expect(result.success).toBe(false);
      expect(result.error).toBe('Conflict');
    });

    it('should handle bridge exception', async () => {
      mockBridge.synapseUpdateState.mockRejectedValue(new Error('Network error'));

      const result = await service.updateState('agent-123', { key: 'value' });

      expect(result.success).toBe(false);
      expect(result.error).toContain('Network error');
    });
  });

  describe('deleteKeys', () => {
    it('should call updateState with null values for each key', async () => {
      mockBridge.synapseUpdateState.mockResolvedValue(
        JSON.stringify({ version: 3 }),
      );

      const result = await service.deleteKeys('agent-123', ['key1', 'key2']);

      expect(result.success).toBe(true);
      expect(mockBridge.synapseUpdateState).toHaveBeenCalledWith(
        'agent-123',
        JSON.stringify({ key1: null, key2: null }),
      );
    });
  });

  describe('storeMemory', () => {
    it('should return memory ID on success', async () => {
      mockBridge.synapseStoreMemory.mockResolvedValue(
        JSON.stringify({ id: 'memory-123' }),
      );

      const result = await service.storeMemory('agent-123', 'test memory');

      expect(result.id).toBe('memory-123');
      expect(result.error).toBeUndefined();
    });

    it('should return error when bridge returns error', async () => {
      mockBridge.synapseStoreMemory.mockResolvedValue(
        JSON.stringify({ error: 'Storage full' }),
      );

      const result = await service.storeMemory('agent-123', 'test memory');

      expect(result.error).toBe('Storage full');
    });

    it('should return error when no ID returned', async () => {
      mockBridge.synapseStoreMemory.mockResolvedValue(JSON.stringify({}));

      const result = await service.storeMemory('agent-123', 'test memory');

      expect(result.error).toBe('Memory storage failed: no ID returned');
    });
  });

  describe('queryMemory', () => {
    it('should return similarity results', async () => {
      const mockResults = [
        { node_id: 'node-1', score: 0.95 },
        { node_id: 'node-2', score: 0.85 },
      ];
      mockBridge.synapseQueryMemory.mockResolvedValue(JSON.stringify(mockResults));

      const result = await service.queryMemory('test query', 5);

      expect(result).toEqual(mockResults);
      expect(mockBridge.synapseQueryMemory).toHaveBeenCalledWith('test query', 5);
    });

    it('should return empty array on bridge error', async () => {
      mockBridge.synapseQueryMemory.mockRejectedValue(new Error('Query failed'));

      const result = await service.queryMemory('test query');

      expect(result).toEqual([]);
    });

    it('should return empty array when bridge returns error object', async () => {
      mockBridge.synapseQueryMemory.mockResolvedValue(
        JSON.stringify({ error: 'Invalid query' }),
      );

      const result = await service.queryMemory('test query');

      expect(result).toEqual([]);
    });
  });
});
