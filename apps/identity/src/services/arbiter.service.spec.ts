/**
 * Arbiter Service Unit Tests
 * 
 * Tests kill switch, audit statistics, and chaos stats.
 */
import { Test, TestingModule } from '@nestjs/testing';
import { ArbiterService } from './arbiter.service';

// Mock the bridge module
jest.mock('../../native-bridge', () => ({
  arbiterKillSwitchActivate: jest.fn(),
  arbiterKillSwitchStatus: jest.fn(),
  arbiterKillSwitchDeactivate: jest.fn(),
  arbiterQueryAudit: jest.fn(),
  arbiterChaosStats: jest.fn(),
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
      bridgeMock.arbiterKillSwitchStatus.mockResolvedValue(
        JSON.stringify({ active: false, terminated_count: 0 }),
      );
      await expect(service.onModuleInit()).resolves.not.toThrow();
    });
  });

  describe('activateKillSwitch', () => {
    it('should activate kill switch with reason', async () => {
      const mockResult = {
        id: 'kill-123',
        timestamp: new Date().toISOString(),
        target_id: 'global',
        target_type: 'Global',
        reason: 'Security breach detected',
        termination_type: 'Graceful',
        success: true,
      };
      bridgeMock.arbiterKillSwitchActivate.mockResolvedValue(JSON.stringify(mockResult));

      const result = await service.activateKillSwitch('Security breach detected');

      if (!('error' in result)) {
        expect(result.success).toBe(true);
        expect(result.reason).toBe('Security breach detected');
      }
    });

    it('should activate kill switch for specific agent', async () => {
      const mockResult = {
        id: 'kill-456',
        target_id: 'agent-123',
        target_type: 'Agent',
        reason: 'Agent compromised',
        success: true,
      };
      bridgeMock.arbiterKillSwitchActivate.mockResolvedValue(JSON.stringify(mockResult));

      const result = await service.activateKillSwitch('Agent compromised', 'agent-123');

      if (!('error' in result)) {
        expect(result.target_id).toBe('agent-123');
      }
    });
  });

  describe('getKillSwitchStatus', () => {
    it('should return inactive status', async () => {
      bridgeMock.arbiterKillSwitchStatus.mockResolvedValue(
        JSON.stringify({ active: false, terminated_count: 0 }),
      );

      const result = await service.getKillSwitchStatus();

      expect(result.active).toBe(false);
      expect(result.terminated_count).toBe(0);
    });

    it('should return active status', async () => {
      bridgeMock.arbiterKillSwitchStatus.mockResolvedValue(
        JSON.stringify({ active: true, terminated_count: 5 }),
      );

      const result = await service.getKillSwitchStatus();

      expect(result.active).toBe(true);
      expect(result.terminated_count).toBe(5);
    });
  });

  describe('deactivateKillSwitch', () => {
    it('should deactivate kill switch', async () => {
      bridgeMock.arbiterKillSwitchDeactivate.mockResolvedValue(
        JSON.stringify({ active: false }),
      );

      const result = await service.deactivateKillSwitch();

      expect(result.active).toBe(false);
    });
  });

  describe('getAuditStatistics', () => {
    it('should return audit statistics', async () => {
      const mockStats = {
        total_records: 100,
        approved_count: 80,
        denied_count: 10,
        review_count: 5,
        logged_count: 5,
        high_risk_count: 3,
        avg_risk_score: 0.25,
      };
      bridgeMock.arbiterQueryAudit.mockResolvedValue(JSON.stringify(mockStats));

      const result = await service.getAuditStatistics();

      expect(result?.total_records).toBe(100);
      expect(result?.approved_count).toBe(80);
    });

    it('should accept limit parameter', async () => {
      const mockStats = {
        total_records: 50,
        approved_count: 40,
        denied_count: 5,
        review_count: 3,
        logged_count: 2,
        high_risk_count: 1,
        avg_risk_score: 0.15,
      };
      bridgeMock.arbiterQueryAudit.mockResolvedValue(JSON.stringify(mockStats));

      const result = await service.getAuditStatistics(50);

      expect(result?.total_records).toBe(50);
    });
  });

  describe('getChaosStats', () => {
    it('should return chaos statistics', () => {
      const mockStats = {
        total_ops: 100,
        latency_injections: 20,
        error_injections: 10,
      };
      bridgeMock.arbiterChaosStats.mockReturnValue(JSON.stringify(mockStats));

      const result = service.getChaosStats();

      expect(result.total_ops).toBe(100);
      expect(result.latency_injections).toBe(20);
      expect(result.error_injections).toBe(10);
    });
  });
});
