/**
 * Arbiter Service Unit Tests
 * 
 * Tests kill switch, chaos injection, and audit logging.
 */
import { Test, TestingModule } from '@nestjs/testing';
import { ArbiterService } from './arbiter.service';

// Mock the bridge module
jest.mock('../../native-bridge', () => ({
  arbiter_kill_switch: jest.fn(),
  arbiter_chaos_inject: jest.fn(),
  arbiter_audit_log: jest.fn(),
  arbiter_get_status: jest.fn(),
}));

describe('ArbiterService', () => {
  let service: ArbiterService;
  let bridgeMock: Record<string, jest.Mock>;

  beforeEach(async () => {
    jest.resetModules();
    
    const module: TestingModule = await Test.createTestingModule({
      providers: [ArbiterService],
    }).compile();

    service = module.get<ArbiterService>(ArbiterService);
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
      bridgeMock.arbiter_get_status.mockReturnValue(
        JSON.stringify({ kill_switch: false, chaos_enabled: false }),
      );
      await expect(service.onModuleInit()).resolves.not.toThrow();
    });
  });

  describe('activateKillSwitch', () => {
    it('should activate global kill switch', async () => {
      const mockResult = {
        activated: true,
        reason: 'Security breach detected',
        activated_by: 'admin-001',
        timestamp: Date.now(),
      };
      bridgeMock.arbiter_kill_switch.mockReturnValue(JSON.stringify(mockResult));

      const result = await service.activateKillSwitch('Security breach detected', 'admin-001');

      expect(result.activated).toBe(true);
      expect(bridgeMock.arbiter_kill_switch).toHaveBeenCalledWith(
        expect.any(String),
        'Security breach detected',
        'admin-001',
      );
    });

    it('should activate agent-specific kill switch', async () => {
      const mockResult = {
        activated: true,
        agent_id: 'agent-123',
        reason: 'Agent compromised',
        timestamp: Date.now(),
      };
      bridgeMock.arbiter_kill_switch.mockReturnValue(JSON.stringify(mockResult));

      const result = await service.activateKillSwitch(
        'Agent compromised',
        'admin-001',
        'agent-123',
      );

      expect(result.agent_id).toBe('agent-123');
    });

    it('should require reason for kill switch', async () => {
      await expect(service.activateKillSwitch('', 'admin-001')).rejects.toThrow();
    });

    it('should require activator identity', async () => {
      await expect(service.activateKillSwitch('Reason', '')).rejects.toThrow();
    });
  });

  describe('deactivateKillSwitch', () => {
    it('should deactivate kill switch with authorization', async () => {
      const mockResult = {
        deactivated: true,
        deactivated_by: 'admin-001',
        timestamp: Date.now(),
      };
      bridgeMock.arbiter_kill_switch.mockReturnValue(JSON.stringify(mockResult));

      const result = await service.deactivateKillSwitch('admin-001');

      expect(result.deactivated).toBe(true);
    });
  });

  describe('injectChaos', () => {
    it('should inject latency chaos', async () => {
      const mockResult = {
        chaos_id: 'chaos-123',
        type: 'latency',
        target: 'treasury',
        duration_ms: 5000,
        started_at: Date.now(),
      };
      bridgeMock.arbiter_chaos_inject.mockReturnValue(JSON.stringify(mockResult));

      const result = await service.injectChaos({
        type: 'latency',
        target: 'treasury',
        duration: 5000,
      });

      expect(result.chaos_id).toBe('chaos-123');
      expect(result.type).toBe('latency');
    });

    it('should inject error chaos', async () => {
      const mockResult = {
        chaos_id: 'chaos-456',
        type: 'error',
        target: 'nexus',
        error_rate: 0.5,
        started_at: Date.now(),
      };
      bridgeMock.arbiter_chaos_inject.mockReturnValue(JSON.stringify(mockResult));

      const result = await service.injectChaos({
        type: 'error',
        target: 'nexus',
        errorRate: 0.5,
      });

      expect(result.type).toBe('error');
      expect(result.error_rate).toBe(0.5);
    });

    it('should reject invalid chaos type', async () => {
      await expect(
        service.injectChaos({
          type: 'invalid' as 'latency',
          target: 'treasury',
        }),
      ).rejects.toThrow();
    });

    it('should reject production chaos without flag', async () => {
      // In production mode, chaos should be disabled by default
      process.env.NODE_ENV = 'production';
      
      await expect(
        service.injectChaos({
          type: 'latency',
          target: 'treasury',
        }),
      ).rejects.toThrow();

      process.env.NODE_ENV = 'test';
    });
  });

  describe('getAuditLog', () => {
    it('should retrieve audit log entries', async () => {
      const mockLog = {
        entries: [
          {
            id: 'audit-1',
            action: 'kill_switch_activated',
            actor: 'admin-001',
            timestamp: Date.now(),
          },
          {
            id: 'audit-2',
            action: 'chaos_injected',
            actor: 'admin-001',
            timestamp: Date.now(),
          },
        ],
        total: 2,
      };
      bridgeMock.arbiter_audit_log.mockReturnValue(JSON.stringify(mockLog));

      const result = await service.getAuditLog({ limit: 10 });

      expect(result.entries).toHaveLength(2);
      expect(result.entries[0].action).toBe('kill_switch_activated');
    });

    it('should filter by agent ID', async () => {
      const mockLog = {
        entries: [
          {
            id: 'audit-1',
            action: 'agent_suspended',
            agent_id: 'agent-123',
            timestamp: Date.now(),
          },
        ],
        total: 1,
      };
      bridgeMock.arbiter_audit_log.mockReturnValue(JSON.stringify(mockLog));

      const result = await service.getAuditLog({ agentId: 'agent-123' });

      expect(result.entries[0].agent_id).toBe('agent-123');
    });
  });

  describe('getStatus', () => {
    it('should return current arbiter status', async () => {
      const mockStatus = {
        kill_switch: false,
        chaos_enabled: false,
        active_chaos_experiments: 0,
        pending_audit_entries: 0,
      };
      bridgeMock.arbiter_get_status.mockReturnValue(JSON.stringify(mockStatus));

      const result = await service.getStatus();

      expect(result.kill_switch).toBe(false);
      expect(result.chaos_enabled).toBe(false);
    });

    it('should reflect kill switch state', async () => {
      const mockStatus = {
        kill_switch: true,
        kill_switch_reason: 'Security breach',
        chaos_enabled: false,
      };
      bridgeMock.arbiter_get_status.mockReturnValue(JSON.stringify(mockStatus));

      const result = await service.getStatus();

      expect(result.kill_switch).toBe(true);
      expect(result.kill_switch_reason).toBe('Security breach');
    });
  });
});
