/**
 * Treasury Service Unit Tests
 * 
 * Tests balance checking, transfers, budget management, and carbon tracking.
 */
import { Test, TestingModule } from '@nestjs/testing';
import { TreasuryService } from './treasury.service';

// Mock the bridge module
jest.mock('../../native-bridge', () => ({
  treasury_get_balance: jest.fn(),
  treasury_transfer: jest.fn(),
  treasury_check_budget: jest.fn(),
  treasury_carbon_footprint: jest.fn(),
  treasury_purchase_offset: jest.fn(),
}));

describe('TreasuryService', () => {
  let service: TreasuryService;
  let bridgeMock: Record<string, jest.Mock>;

  beforeEach(async () => {
    // Reset all mocks
    jest.resetModules();
    
    const module: TestingModule = await Test.createTestingModule({
      providers: [TreasuryService],
    }).compile();

    service = module.get<TreasuryService>(TreasuryService);
    
    // Get mock references
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
      bridgeMock.treasury_get_balance.mockReturnValue(JSON.stringify({ balance: '0' }));
      
      // Service should not throw during init if bridge works
      await expect(service.onModuleInit()).resolves.not.toThrow();
    });
  });

  describe('getBalance', () => {
    it('should return balance for valid agent', async () => {
      const mockBalance = {
        balance: '1000.00',
        currency: 'USD',
        last_updated: Date.now(),
      };
      bridgeMock.treasury_get_balance.mockReturnValue(JSON.stringify(mockBalance));

      const result = await service.getBalance('agent-123');

      expect(result).toEqual(mockBalance);
      expect(bridgeMock.treasury_get_balance).toHaveBeenCalledWith('agent-123');
    });

    it('should throw error for invalid agent', async () => {
      bridgeMock.treasury_get_balance.mockImplementation(() => {
        throw new Error('Agent not found');
      });

      await expect(service.getBalance('invalid')).rejects.toThrow('Agent not found');
    });

    it('should handle null response from bridge', async () => {
      bridgeMock.treasury_get_balance.mockReturnValue(null);

      await expect(service.getBalance('agent-123')).rejects.toThrow();
    });
  });

  describe('transfer', () => {
    it('should execute successful transfer', async () => {
      const mockResult = {
        transaction_id: 'tx-123',
        status: 'completed',
        amount: '100.00',
        from: 'agent-a',
        to: 'agent-b',
      };
      bridgeMock.treasury_transfer.mockReturnValue(JSON.stringify(mockResult));

      const result = await service.transfer('agent-a', 'agent-b', 100);

      expect(result.transaction_id).toBe('tx-123');
      expect(result.status).toBe('completed');
    });

    it('should reject transfer with insufficient funds', async () => {
      bridgeMock.treasury_transfer.mockImplementation(() => {
        throw new Error('Insufficient funds');
      });

      await expect(service.transfer('agent-a', 'agent-b', 1000000)).rejects.toThrow(
        'Insufficient funds',
      );
    });

    it('should reject transfer with negative amount', async () => {
      await expect(service.transfer('agent-a', 'agent-b', -100)).rejects.toThrow();
    });

    it('should reject self-transfer', async () => {
      await expect(service.transfer('agent-a', 'agent-a', 100)).rejects.toThrow();
    });
  });

  describe('checkBudget', () => {
    it('should return budget status', async () => {
      const mockBudget = {
        agent_id: 'agent-123',
        limit: '5000.00',
        used: '1500.00',
        remaining: '3500.00',
        period: 'monthly',
      };
      bridgeMock.treasury_check_budget.mockReturnValue(JSON.stringify(mockBudget));

      const result = await service.checkBudget('agent-123');

      expect(result.remaining).toBe('3500.00');
      expect(bridgeMock.treasury_check_budget).toHaveBeenCalledWith('agent-123');
    });
  });

  describe('getCarbonFootprint', () => {
    it('should return carbon metrics', async () => {
      const mockCarbon = {
        agent_id: 'agent-123',
        total_emissions_kg: 12.5,
        offset_purchased_kg: 5.0,
        net_emissions_kg: 7.5,
      };
      bridgeMock.treasury_carbon_footprint.mockReturnValue(JSON.stringify(mockCarbon));

      const result = await service.getCarbonFootprint('agent-123');

      expect(result.total_emissions_kg).toBe(12.5);
      expect(result.net_emissions_kg).toBe(7.5);
    });
  });

  describe('purchaseCarbonOffset', () => {
    it('should purchase carbon offset', async () => {
      const mockOffset = {
        offset_id: 'offset-123',
        amount_kg: 10.0,
        cost_usd: 15.0,
        provider: 'Pachama',
      };
      bridgeMock.treasury_purchase_offset.mockReturnValue(JSON.stringify(mockOffset));

      const result = await service.purchaseCarbonOffset('agent-123', 10.0);

      expect(result.offset_id).toBe('offset-123');
      expect(result.amount_kg).toBe(10.0);
    });

    it('should reject invalid offset amount', async () => {
      await expect(service.purchaseCarbonOffset('agent-123', 0)).rejects.toThrow();
      await expect(service.purchaseCarbonOffset('agent-123', -5)).rejects.toThrow();
    });
  });
});
