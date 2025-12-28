import { Test, TestingModule } from '@nestjs/testing';
import { ConfigService } from '@nestjs/config';
import { getRepositoryToken } from '@nestjs/typeorm';
import {
  AgentSandboxService,
  AgentStatus,
} from './agent-sandbox.service';
import { AuditLoggerService } from './audit-logger.service';
import { AgentRecordEntity } from '../entities/agent-record.entity';

// Mock repository factory
const createMockRepository = () => ({
  find: jest.fn().mockResolvedValue([]),
  findOne: jest.fn().mockResolvedValue(null),
  save: jest.fn().mockImplementation(entity => Promise.resolve({ id: 'mock-id', ...entity })),
  create: jest.fn().mockImplementation(entity => entity),
  delete: jest.fn().mockResolvedValue({ affected: 1 }),
});

describe('AgentSandboxService', () => {
  let service: AgentSandboxService;
  let auditLogger: jest.Mocked<AuditLoggerService>;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [
        AgentSandboxService,
        {
          provide: ConfigService,
          useValue: {
            get: jest.fn().mockReturnValue(undefined),
          },
        },
        {
          provide: AuditLoggerService,
          useValue: {
            logSecurityEvent: jest.fn(),
          },
        },
        {
          provide: getRepositoryToken(AgentRecordEntity),
          useValue: createMockRepository(),
        },
      ],
    }).compile();

    service = module.get<AgentSandboxService>(AgentSandboxService);
    auditLogger = module.get(AuditLoggerService);
    await service.onModuleInit();
  });

  describe('agent registration', () => {
    it('should register a new agent', () => {
      const agent = service.registerAgent('agent-1', 'TestAgent', '1.0.0');

      expect(agent).toBeDefined();
      expect(agent.id).toBe('agent-1');
      expect(agent.name).toBe('TestAgent');
      expect(agent.version).toBe('1.0.0');
      expect(agent.status).toBe(AgentStatus.ACTIVE);
      expect(agent.reputation.score).toBe(500); // Neutral starting score
    });

    it('should return existing agent on duplicate registration', () => {
      const first = service.registerAgent('agent-1', 'TestAgent', '1.0.0');
      const second = service.registerAgent('agent-1', 'DifferentName', '2.0.0');

      expect(first).toBe(second);
      expect(second.name).toBe('TestAgent'); // Original name retained
    });

    it('should register agent with custom budget', () => {
      const agent = service.registerAgent('agent-2', 'TestAgent', '1.0.0', {
        maxTokens: 500000,
        maxApiCalls: 5000,
      });

      expect(agent.budget.maxTokens).toBe(500000);
      expect(agent.budget.maxApiCalls).toBe(5000);
    });
  });

  describe('action checking', () => {
    it('should allow valid action', async () => {
      service.registerAgent('agent-1', 'TestAgent', '1.0.0');

      const result = await service.checkAction({
        agentId: 'agent-1',
        action: 'transfer',
        target: { service: 'bank.com', endpoint: '/transfer', method: 'POST' },
      });

      expect(result.allowed).toBe(true);
      expect(result.agentStatus).toBe(AgentStatus.ACTIVE);
    });

    it('should auto-register unknown agent', async () => {
      const result = await service.checkAction({
        agentId: 'unknown-agent',
        action: 'test',
        target: { service: 'test.com', endpoint: '/test', method: 'GET' },
      });

      expect(result.allowed).toBe(true);
      const agent = service.getAgentStatus('unknown-agent');
      expect(agent).toBeDefined();
    });

    it('should block suspended agent', async () => {
      service.registerAgent('agent-1', 'TestAgent', '1.0.0');
      service.suspendAgent('agent-1', 'Test suspension');

      const result = await service.checkAction({
        agentId: 'agent-1',
        action: 'test',
        target: { service: 'test.com', endpoint: '/test', method: 'GET' },
      });

      expect(result.allowed).toBe(false);
      expect(result.agentStatus).toBe(AgentStatus.SUSPENDED);
    });

    it('should block terminated agent', async () => {
      service.registerAgent('agent-1', 'TestAgent', '1.0.0');
      service.terminateAgent('agent-1', 'Test termination');

      const result = await service.checkAction({
        agentId: 'agent-1',
        action: 'test',
        target: { service: 'test.com', endpoint: '/test', method: 'GET' },
      });

      expect(result.allowed).toBe(false);
      expect(result.agentStatus).toBe(AgentStatus.TERMINATED);
    });
  });

  describe('budget enforcement', () => {
    it('should block when token budget exceeded', async () => {
      service.registerAgent('agent-1', 'TestAgent', '1.0.0', {
        maxTokens: 100,
        periodSeconds: 86400,
      });

      const result = await service.checkAction({
        agentId: 'agent-1',
        action: 'test',
        target: { service: 'test.com', endpoint: '/test', method: 'GET' },
        estimatedTokens: 150,
      });

      expect(result.allowed).toBe(false);
      expect(result.reason).toBe('Token budget exceeded');
    });

    it('should track remaining budget', async () => {
      service.registerAgent('agent-1', 'TestAgent', '1.0.0', {
        maxTokens: 1000,
        maxApiCalls: 100,
        maxCostUsd: 10,
        periodSeconds: 86400,
      });

      const result = await service.checkAction({
        agentId: 'agent-1',
        action: 'test',
        target: { service: 'test.com', endpoint: '/test', method: 'GET' },
        estimatedTokens: 100,
      });

      expect(result.allowed).toBe(true);
      expect(result.remainingBudget?.tokens).toBe(1000); // Not deducted yet
      expect(result.remainingBudget?.apiCalls).toBe(100);
    });
  });

  describe('reputation management', () => {
    it('should increase reputation on success', () => {
      service.registerAgent('agent-1', 'TestAgent', '1.0.0');
      const initialScore = service.getAgentReputation('agent-1')!.score;

      service.recordSuccess('agent-1', 100, 0.01);

      const newScore = service.getAgentReputation('agent-1')!.score;
      expect(newScore).toBeGreaterThan(initialScore);
    });

    it('should decrease reputation on failure', () => {
      service.registerAgent('agent-1', 'TestAgent', '1.0.0');
      const initialScore = service.getAgentReputation('agent-1')!.score;

      service.recordFailure('agent-1', 'Test failure');

      const newScore = service.getAgentReputation('agent-1')!.score;
      expect(newScore).toBeLessThan(initialScore);
    });

    it('should severely decrease reputation on violation', () => {
      service.registerAgent('agent-1', 'TestAgent', '1.0.0');
      const initialScore = service.getAgentReputation('agent-1')!.score;

      service.recordViolation('agent-1', 'Security violation');

      const newScore = service.getAgentReputation('agent-1')!.score;
      expect(newScore).toBeLessThan(initialScore - 50); // At least 100 point penalty
    });

    it('should auto-suspend after multiple violations', () => {
      service.registerAgent('agent-1', 'TestAgent', '1.0.0');

      service.recordViolation('agent-1', 'Violation 1');
      service.recordViolation('agent-1', 'Violation 2');
      service.recordViolation('agent-1', 'Violation 3');

      const agent = service.getAgentStatus('agent-1');
      expect(agent?.status).toBe(AgentStatus.SUSPENDED);
    });
  });

  describe('kill switch', () => {
    it('should terminate individual agent', () => {
      service.registerAgent('agent-1', 'TestAgent', '1.0.0');

      const result = service.terminateAgent('agent-1', 'Manual termination');

      expect(result).toBe(true);
      const agent = service.getAgentStatus('agent-1');
      expect(agent?.status).toBe(AgentStatus.TERMINATED);
      expect(agent?.terminationReason).toBe('Manual termination');
    });

    it('should activate global kill switch', async () => {
      service.registerAgent('agent-1', 'TestAgent', '1.0.0');
      service.registerAgent('agent-2', 'TestAgent2', '1.0.0');

      service.activateGlobalKillSwitch('Emergency');

      const result = await service.checkAction({
        agentId: 'agent-1',
        action: 'test',
        target: { service: 'test.com', endpoint: '/test', method: 'GET' },
      });

      expect(result.allowed).toBe(false);
      expect(result.reason).toBe('Global kill switch activated');
    });

    it('should deactivate global kill switch', async () => {
      service.registerAgent('agent-1', 'TestAgent', '1.0.0');
      service.activateGlobalKillSwitch('Emergency');
      service.deactivateGlobalKillSwitch();

      // Agent was terminated by kill switch, need to reactivate
      service.registerAgent('agent-new', 'NewAgent', '1.0.0');

      const result = await service.checkAction({
        agentId: 'agent-new',
        action: 'test',
        target: { service: 'test.com', endpoint: '/test', method: 'GET' },
      });

      expect(result.allowed).toBe(true);
    });
  });

  describe('agent lifecycle', () => {
    it('should suspend and reactivate agent', () => {
      service.registerAgent('agent-1', 'TestAgent', '1.0.0');

      service.suspendAgent('agent-1', 'Temporary suspension');
      expect(service.getAgentStatus('agent-1')?.status).toBe(AgentStatus.SUSPENDED);

      service.reactivateAgent('agent-1');
      expect(service.getAgentStatus('agent-1')?.status).toBe(AgentStatus.ACTIVE);
    });

    it('should not reactivate terminated agent', () => {
      service.registerAgent('agent-1', 'TestAgent', '1.0.0');
      service.terminateAgent('agent-1', 'Permanent termination');

      const result = service.reactivateAgent('agent-1');

      expect(result).toBe(false);
      expect(service.getAgentStatus('agent-1')?.status).toBe(AgentStatus.TERMINATED);
    });

    it('should get all agents', () => {
      service.registerAgent('agent-1', 'TestAgent1', '1.0.0');
      service.registerAgent('agent-2', 'TestAgent2', '1.0.0');

      const agents = service.getAllAgents();

      expect(agents.length).toBe(2);
    });
  });

  describe('rate limiting', () => {
    it('should rate limit excessive requests', async () => {
      service.registerAgent('agent-1', 'TestAgent', '1.0.0');

      // Make 101 requests (limit is 100 per minute)
      for (let i = 0; i < 100; i++) {
        await service.checkAction({
          agentId: 'agent-1',
          action: 'test',
          target: { service: 'test.com', endpoint: '/test', method: 'GET' },
        });
      }

      const result = await service.checkAction({
        agentId: 'agent-1',
        action: 'test',
        target: { service: 'test.com', endpoint: '/test', method: 'GET' },
      });

      expect(result.allowed).toBe(false);
      expect(result.agentStatus).toBe(AgentStatus.RATE_LIMITED);
    });
  });
});
