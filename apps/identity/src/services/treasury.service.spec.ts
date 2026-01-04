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

      if (!('error' in result)) {
        expect(result.transaction_id).toBe('tx-123');
        expect(result.status).toBe('Completed');
      }
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

  describe('getBudget', () => {
    it('should return budget status', async () => {
      const mockBudget = {
        agent_id: 'agent-123',
        remaining: 3500.00,
        message: 'Budget available',
      };
      bridgeMock.treasury_check_budget.mockReturnValue(JSON.stringify(mockBudget));

      const result = await service.getBudget('agent-123');

      expect(result.remaining).toBe(3500.00);
    });
  });

  describe('getCarbon', () => {
    it('should return carbon metrics', async () => {
      const mockCarbon = {
        total_co2_grams: '12500',
        total_energy_kwh: '5.0',
        total_water_liters: '10.0',
        action_count: 100,
      };
      bridgeMock.treasury_carbon_footprint.mockReturnValue(JSON.stringify(mockCarbon));

      const result = await service.getCarbon('agent-123');

      expect(result?.total_co2_grams).toBe('12500');
    });
  });

  describe('purchaseOffset', () => {
    it('should purchase carbon offset', async () => {
      const mockOffset = {
        transaction_id: 'tx-123',
        tons: 10.0,
        cost: 15.0,
        provider: 'Pachama',
        certificate_url: 'https://example.com/cert',
        timestamp: new Date().toISOString(),
      };
      bridgeMock.treasury_purchase_offset.mockReturnValue(JSON.stringify(mockOffset));

      const result = await service.purchaseOffset('agent-123', 10.0);

      if (!('error' in result)) {
        expect(result.transaction_id).toBe('tx-123');
        expect(result.tons).toBe(10.0);
      }
    });
  });
});
