/**
 * Controller Integration Tests
 *
 * Tests that verify controllers properly integrate with their services
 * and the underlying Rust N-API bridge.
 */

import { Test, TestingModule } from '@nestjs/testing';
import { TreasuryController } from '../controllers/treasury.controller';
import { TreasuryService } from '../services/treasury.service';
import { ArbiterController } from '../controllers/arbiter.controller';
import { ArbiterService } from '../services/arbiter.service';
import { SynapseController } from '../controllers/synapse.controller';
import { SynapseService } from '../services/synapse.service';
import { GateService } from '../services/gate.service';

// =============================================================================
// Mock Services (bridge not available in test environment)
// =============================================================================

const mockTreasuryService = {
  getBalance: jest.fn().mockResolvedValue({
    agent_id: 'test-agent',
    balance: { value: 100000, decimals: 2 },
    currency: 'VMC',
    updated_at: new Date().toISOString(),
  }),
  deposit: jest.fn().mockResolvedValue({
    agent_id: 'test-agent',
    balance: { value: 200000, decimals: 2 },
    currency: 'VMC',
    updated_at: new Date().toISOString(),
  }),
  transfer: jest.fn().mockResolvedValue({
    transaction_id: 'tx_123',
    status: 'Completed',
    timestamp: new Date().toISOString(),
  }),
  getBudget: jest.fn().mockResolvedValue({
    agent_id: 'test-agent',
    remaining: 75,
  }),
  getCarbon: jest.fn().mockResolvedValue({
    total_co2_grams: '1500.5',
    total_energy_kwh: '2.5',
    period_end: new Date().toISOString(),
  }),
  purchaseOffset: jest.fn().mockResolvedValue({
    success: true,
    cost: 0.03,
  }),
};

const mockArbiterService = {
  activateKillSwitch: jest.fn().mockResolvedValue({
    success: true,
    id: 'ks_123',
    target_id: 'all',
    timestamp: new Date().toISOString(),
  }),
  getKillSwitchStatus: jest.fn().mockResolvedValue({ active: false }),
  deactivateKillSwitch: jest.fn().mockResolvedValue({ active: false }),
  getAuditStatistics: jest.fn().mockResolvedValue({
    total_records: 150,
    approved_count: 120,
    denied_count: 10,
    review_count: 5,
    logged_count: 15,
    high_risk_count: 3,
    avg_risk_score: 0.25,
  }),
  getChaosStats: jest.fn().mockReturnValue({
    total_ops: 100,
    latency_injections: 50,
    error_injections: 25,
  }),
};

const mockSynapseService = {
  getState: jest.fn().mockResolvedValue({
    agent_id: 'test-agent',
    state: { key: 'value' },
    version: 1,
  }),
  updateState: jest.fn().mockResolvedValue({ success: true, version: 2 }),
  deleteKeys: jest.fn().mockResolvedValue({ success: true }),
  queryMemory: jest.fn().mockResolvedValue([
    { node_id: 'node-1', score: 0.95 },
    { node_id: 'node-2', score: 0.87 },
  ]),
  storeMemory: jest.fn().mockResolvedValue({ id: 'mem-123' }),
};

const mockGateService = {
  guardContext: jest.fn().mockReturnValue({
    safe: true,
    injections_found: 0,
    suspicious_chunks: [],
    latency_us: 50,
  }),
};

// =============================================================================
// Treasury Controller Tests
// =============================================================================

describe('TreasuryController Integration', () => {
  let controller: TreasuryController;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      controllers: [TreasuryController],
      providers: [{ provide: TreasuryService, useValue: mockTreasuryService }],
    }).compile();

    controller = module.get<TreasuryController>(TreasuryController);
  });

  it('should call TreasuryService.getBalance', async () => {
    const result = await controller.getBalance('test-agent');
    expect(mockTreasuryService.getBalance).toHaveBeenCalledWith('test-agent');
    expect(result.agentId).toBe('test-agent');
    expect(result.balance).toBe(1000); // 100000 / 100
  });

  it('should call TreasuryService.deposit', async () => {
    const result = await controller.deposit('test-agent', { amount: 500 });
    expect(mockTreasuryService.deposit).toHaveBeenCalledWith('test-agent', 500);
    expect(result.balance).toBe(2000); // 200000 / 100
  });

  it('should call TreasuryService.transfer', async () => {
    const result = await controller.transfer({
      fromAgent: 'agent-a',
      toAgent: 'agent-b',
      amount: 100,
      reference: 'test-tx',
    });
    expect(mockTreasuryService.transfer).toHaveBeenCalled();
    expect(result.status).toBe('completed');
  });

  it('should call TreasuryService.getBudget', async () => {
    const result = await controller.getBudget('test-agent');
    expect(mockTreasuryService.getBudget).toHaveBeenCalledWith('test-agent');
    expect(result.remaining).toBe(75);
  });

  it('should call TreasuryService.getCarbon', async () => {
    const result = await controller.getCarbonFootprint('test-agent');
    expect(mockTreasuryService.getCarbon).toHaveBeenCalledWith('test-agent');
    expect(result.totalGramsCO2).toBe(1500.5);
  });

  it('should call TreasuryService.purchaseOffset', async () => {
    const result = await controller.purchaseOffset({
      agentId: 'test-agent',
      grams: 1000,
    });
    expect(mockTreasuryService.purchaseOffset).toHaveBeenCalled();
    expect(result.success).toBe(true);
  });
});

// =============================================================================
// Arbiter Controller Tests
// =============================================================================

describe('ArbiterController Integration', () => {
  let controller: ArbiterController;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      controllers: [ArbiterController],
      providers: [{ provide: ArbiterService, useValue: mockArbiterService }],
    }).compile();

    controller = module.get<ArbiterController>(ArbiterController);
  });

  it('should call ArbiterService.activateKillSwitch', async () => {
    const result = await controller.activateKillSwitch({
      reason: 'Emergency test',
      agentId: 'agent-1',
    });
    expect(mockArbiterService.activateKillSwitch).toHaveBeenCalledWith(
      'Emergency test',
      'agent-1',
    );
    expect(result.success).toBe(true);
  });

  it('should call ArbiterService.getKillSwitchStatus', async () => {
    const result = await controller.getKillSwitchStatus();
    expect(mockArbiterService.getKillSwitchStatus).toHaveBeenCalled();
    expect(result.active).toBe(false);
  });

  it('should call ArbiterService.deactivateKillSwitch', async () => {
    const result = await controller.deactivateKillSwitch();
    expect(mockArbiterService.deactivateKillSwitch).toHaveBeenCalled();
    expect(result.success).toBe(true);
  });

  it('should call ArbiterService.getAuditStatistics', async () => {
    const result = await controller.queryAuditLog(undefined, undefined, 100);
    expect(mockArbiterService.getAuditStatistics).toHaveBeenCalledWith(100);
    expect(result.totalCount).toBe(150);
    expect(result.statistics?.approved).toBe(120);
  });

  it('should call ArbiterService.getChaosStats', async () => {
    const result = await controller.injectChaos({
      type: 'latency',
      target: 'test-service',
      durationSeconds: 30,
    });
    expect(mockArbiterService.getChaosStats).toHaveBeenCalled();
    expect(result.stats?.totalOps).toBe(100);
  });
});

// =============================================================================
// Synapse Controller Tests
// =============================================================================

describe('SynapseController Integration', () => {
  let controller: SynapseController;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      controllers: [SynapseController],
      providers: [
        { provide: SynapseService, useValue: mockSynapseService },
        { provide: GateService, useValue: mockGateService },
      ],
    }).compile();

    controller = module.get<SynapseController>(SynapseController);
  });

  it('should call SynapseService.getState', async () => {
    const result = await controller.getState('test-agent');
    expect(mockSynapseService.getState).toHaveBeenCalledWith('test-agent');
    expect(result.agentId).toBe('test-agent');
    expect(result.state).toEqual({ key: 'value' });
  });

  it('should call SynapseService.updateState', async () => {
    const result = await controller.updateState('test-agent', {
      state: { newKey: 'newValue' },
    });
    expect(mockSynapseService.updateState).toHaveBeenCalled();
    expect(result.version).toBeGreaterThan(0);
  });

  it('should call GateService.guardContext for context guard', async () => {
    const result = await controller.guardContext({
      agentId: 'test-agent',
      documents: ['doc1', 'doc2'],
    });
    expect(mockGateService.guardContext).toHaveBeenCalledWith(['doc1', 'doc2']);
    expect(result.safe).toBe(true);
    expect(result.analyzedDocuments).toBe(2);
  });

  it('should call SynapseService.queryMemory', async () => {
    const result = await controller.queryGraph({
      query: 'test query',
      limit: 10,
    });
    expect(mockSynapseService.queryMemory).toHaveBeenCalledWith('test query', 10);
    expect(result.results.length).toBe(2);
  });

  it('should call SynapseService.storeMemory', async () => {
    const result = await controller.storeMemory('test-agent', {
      text: 'memory content',
    });
    expect(mockSynapseService.storeMemory).toHaveBeenCalledWith(
      'test-agent',
      'memory content',
    );
    expect(result.id).toBe('mem-123');
  });
});
