/**
 * Agent Sandbox Service - In-Flight Request & Kill Switch Tests
 *
 * These tests validate the epistemic claims made in the service documentation:
 * - Kill switch blocks new requests immediately
 * - In-flight requests are NOT interrupted (by design)
 * - Reputation thresholds work as documented
 */

import { Test, TestingModule } from '@nestjs/testing';
import { getRepositoryToken } from '@nestjs/typeorm';
import { ConfigService } from '@nestjs/config';
import {
  AgentSandboxService,
  SandboxActionRequest,
} from './agent-sandbox.service';
import { AuditLoggerService } from './audit-logger.service';
import { GateService } from './gate.service';
import { TrustService } from './trust.service';
import { AgentRecordEntity } from '../entities/agent-record.entity';
import { SystemConfigEntity } from '../entities/system-config.entity';
import { AgentStatus } from '../domain/agent.entity';

describe('AgentSandboxService - Kill Switch & In-Flight Behavior', () => {
  let service: AgentSandboxService;

  const mockAgentRepo = {
    find: jest.fn().mockResolvedValue([]),
    findOne: jest.fn().mockResolvedValue(null),
    save: jest.fn().mockImplementation((entity) => Promise.resolve(entity)),
    create: jest
      .fn()
      .mockImplementation((entity) => entity as AgentRecordEntity),
  };

  const mockConfigRepo = {
    findOne: jest.fn().mockResolvedValue(null),
    upsert: jest.fn().mockResolvedValue(undefined),
  };

  const mockAuditLogger = {
    logSecurityEvent: jest.fn().mockResolvedValue(undefined),
  };

  const mockConfigService = {
    get: jest.fn().mockReturnValue(undefined),
  };

  const mockGateService = {
    verify: jest.fn().mockResolvedValue({ allowed: true, riskScore: 0 }),
    guardPrompt: jest
      .fn()
      .mockReturnValue(JSON.stringify({ safe: true, threat_level: 'None' })),
  };

  const mockTrustService = {
    recordTransactionSuccess: jest
      .fn()
      .mockResolvedValue({ score: 50, level: 'medium' }),
    recordTransactionFailure: jest
      .fn()
      .mockResolvedValue({ score: 45, level: 'medium' }),
    recordPolicyViolation: jest
      .fn()
      .mockResolvedValue({ score: 30, level: 'low' }),
    getTrustScore: jest.fn().mockResolvedValue({ score: 50, level: 'medium' }),
  };

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [
        AgentSandboxService,
        {
          provide: getRepositoryToken(AgentRecordEntity),
          useValue: mockAgentRepo,
        },
        {
          provide: getRepositoryToken(SystemConfigEntity),
          useValue: mockConfigRepo,
        },
        { provide: AuditLoggerService, useValue: mockAuditLogger },
        { provide: ConfigService, useValue: mockConfigService },
        { provide: GateService, useValue: mockGateService },
        { provide: TrustService, useValue: mockTrustService },
      ],
    }).compile();

    service = module.get<AgentSandboxService>(AgentSandboxService);
    // Variables used for type checking but not directly accessed in tests
    void module.get(getRepositoryToken(AgentRecordEntity));
    void module.get(getRepositoryToken(SystemConfigEntity));

    // Initialize service
    await service.onModuleInit();
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  describe('In-Flight Request Tracking', () => {
    it('should track in-flight requests when action is approved', async () => {
      // Register an agent
      await service.registerAgent('test-agent', 'Test', '1.0.0');

      // Initial state: no in-flight requests
      expect(service.getInFlightStats().total).toBe(0);

      // Check action (should be approved)
      const request: SandboxActionRequest = {
        agentId: 'test-agent',
        action: 'test_action',
        target: { service: 'test', endpoint: '/test', method: 'POST' },
      };
      const result = await service.checkAction(request);

      expect(result.allowed).toBe(true);
      expect(service.getInFlightStats().total).toBe(1);
      expect(service.getInFlightStats().byAgent['test-agent']).toBe(1);
    });

    it('should decrement in-flight on recordSuccess', async () => {
      await service.registerAgent('test-agent', 'Test', '1.0.0');

      // Approve action (increments in-flight)
      await service.checkAction({
        agentId: 'test-agent',
        action: 'test',
        target: { service: 'test', endpoint: '/test', method: 'POST' },
      });
      expect(service.getInFlightStats().total).toBe(1);

      // Record success (decrements in-flight)
      await service.recordSuccess('test-agent');
      expect(service.getInFlightStats().total).toBe(0);
    });

    it('should decrement in-flight on recordFailure', async () => {
      await service.registerAgent('test-agent', 'Test', '1.0.0');

      await service.checkAction({
        agentId: 'test-agent',
        action: 'test',
        target: { service: 'test', endpoint: '/test', method: 'POST' },
      });
      expect(service.getInFlightStats().total).toBe(1);

      // Record failure (decrements in-flight)
      await service.recordFailure('test-agent', 'Test failure');
      expect(service.getInFlightStats().total).toBe(0);
    });
  });

  describe('Kill Switch Behavior', () => {
    it('should immediately block NEW requests when kill switch is activated', async () => {
      await service.registerAgent('test-agent', 'Test', '1.0.0');

      // Before kill switch: action allowed
      const beforeResult = await service.checkAction({
        agentId: 'test-agent',
        action: 'before_kill',
        target: { service: 'test', endpoint: '/test', method: 'POST' },
      });
      expect(beforeResult.allowed).toBe(true);

      // Activate kill switch
      await service.activateGlobalKillSwitch('Test kill switch');

      // After kill switch: action blocked
      const afterResult = await service.checkAction({
        agentId: 'test-agent',
        action: 'after_kill',
        target: { service: 'test', endpoint: '/test', method: 'POST' },
      });
      expect(afterResult.allowed).toBe(false);
      expect(afterResult.reason).toContain('Global kill switch');
    });

    it('should persist kill switch state to database', async () => {
      await service.activateGlobalKillSwitch('Persistence test');

      expect(mockConfigRepo.upsert).toHaveBeenCalledWith(
        expect.objectContaining({
          key: 'global_kill_switch',
          value: 'true',
        }),
        ['key'],
      );
    });

    it('should log in-flight request count when kill switch activated', async () => {
      await service.registerAgent('test-agent', 'Test', '1.0.0');

      // Create some in-flight requests
      await service.checkAction({
        agentId: 'test-agent',
        action: 'in_flight_1',
        target: { service: 'test', endpoint: '/test', method: 'POST' },
      });
      await service.checkAction({
        agentId: 'test-agent',
        action: 'in_flight_2',
        target: { service: 'test', endpoint: '/test', method: 'POST' },
      });

      expect(service.getInFlightStats().total).toBe(2);

      // Activate kill switch
      await service.activateGlobalKillSwitch('In-flight test');

      // Verify audit log includes in-flight count
      expect(mockAuditLogger.logSecurityEvent).toHaveBeenCalledWith(
        expect.anything(),
        expect.anything(),
        expect.objectContaining({
          inFlightRequests: 2,
          inFlightByAgent: { 'test-agent': 2 },
        }),
      );
    });

    it('should NOT cancel in-flight requests (by design)', async () => {
      await service.registerAgent('test-agent', 'Test', '1.0.0');

      // Start an in-flight request
      await service.checkAction({
        agentId: 'test-agent',
        action: 'in_flight',
        target: { service: 'test', endpoint: '/test', method: 'POST' },
      });

      // Activate kill switch while request is in-flight
      await service.activateGlobalKillSwitch('Mid-flight test');

      // In-flight request should still be tracked (not cancelled)
      expect(service.getInFlightStats().total).toBe(1);

      // Completing the in-flight request should still work
      await service.recordSuccess('test-agent');
      expect(service.getInFlightStats().total).toBe(0);
    });
  });

  describe('Reputation Thresholds', () => {
    it('should block agent when reputation falls below threshold via suspension', async () => {
      await service.registerAgent('test-agent', 'Test', '1.0.0');

      // Initial reputation is 500
      const agent = service.getAgentStatus('test-agent');
      expect(agent?.reputation.score).toBe(500);

      // Record 5 violations (-100 each = -500, final score = 0)
      // Note: Auto-suspend triggers at 3 violations, so agent will be SUSPENDED
      for (let i = 0; i < 5; i++) {
        await service.recordViolation('test-agent', `Violation ${i + 1}`);
      }

      // Agent should be suspended (due to 3+ violations) which also blocks actions
      const result = await service.checkAction({
        agentId: 'test-agent',
        action: 'test',
        target: { service: 'test', endpoint: '/test', method: 'POST' },
      });

      expect(result.allowed).toBe(false);
      // Agent is suspended due to violations (which is the mechanism that blocks low-rep agents)
      expect(result.reason).toContain('suspended');
    });

    it('should auto-suspend after 3 violations', async () => {
      await service.registerAgent('test-agent', 'Test', '1.0.0');

      // Record 3 violations
      await service.recordViolation('test-agent', 'Violation 1');
      await service.recordViolation('test-agent', 'Violation 2');
      await service.recordViolation('test-agent', 'Violation 3');

      // Agent should be suspended
      const agent = service.getAgentStatus('test-agent');
      expect(agent?.status).toBe(AgentStatus.SUSPENDED);
    });
  });

  describe('isKillSwitchActive', () => {
    it('should return false initially', () => {
      expect(service.isKillSwitchActive()).toBe(false);
    });

    it('should return true after activation', async () => {
      await service.activateGlobalKillSwitch('Test');
      expect(service.isKillSwitchActive()).toBe(true);
    });

    it('should return false after deactivation', async () => {
      await service.activateGlobalKillSwitch('Test');
      await service.deactivateGlobalKillSwitch();
      expect(service.isKillSwitchActive()).toBe(false);
    });
  });
});
