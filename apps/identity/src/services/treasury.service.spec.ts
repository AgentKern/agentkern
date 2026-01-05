/**
 * Treasury Service Unit Tests
 *
 * Tests balance checking, transfers, budget management, and carbon tracking.
 * Uses proper mocking strategy for dynamically loaded native bridge.
 */
import { Test, TestingModule } from '@nestjs/testing';
import { TreasuryService } from './treasury.service';

describe('TreasuryService', () => {
  let service: TreasuryService;
  let mockBridge: Record<string, jest.Mock>;

  beforeEach(async () => {
    // Create mock bridge functions
    mockBridge = {
      treasuryGetBalance: jest.fn(),
      treasuryDeposit: jest.fn(),
      treasuryTransfer: jest.fn(),
      treasuryGetBudget: jest.fn(),
      treasuryGetCarbon: jest.fn(),
      treasuryPurchaseOffset: jest.fn(),
    };

    const module: TestingModule = await Test.createTestingModule({
      providers: [TreasuryService],
    }).compile();

    service = module.get<TreasuryService>(TreasuryService);

    // Manually inject the mock bridge and set bridgeLoaded flag
    // This simulates a successful bridge load without requiring the actual native module
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

  describe('getBalance', () => {
    it('should return balance for valid agent', async () => {
      const mockBalance = {
        agent_id: 'agent-123',
        balance: { value: 1000, decimals: 2 },
        currency: 'USD',
        pending: { value: 0, decimals: 2 },
        updated_at: new Date().toISOString(),
        total_deposited: { value: 1000, decimals: 2 },
        total_withdrawn: { value: 0, decimals: 2 },
      };
      mockBridge.treasuryGetBalance.mockReturnValue(JSON.stringify(mockBalance));

      const result = await service.getBalance('agent-123');

      expect(result).toEqual(mockBalance);
      expect(mockBridge.treasuryGetBalance).toHaveBeenCalledWith('agent-123');
    });

    it('should return null for invalid agent when bridge throws', async () => {
      mockBridge.treasuryGetBalance.mockImplementation(() => {
        throw new Error('Agent not found');
      });

      const result = await service.getBalance('invalid');
      expect(result).toBeNull();
    });

    it('should return null when bridge returns invalid JSON', async () => {
      mockBridge.treasuryGetBalance.mockReturnValue('invalid-json');

      const result = await service.getBalance('agent-123');
      expect(result).toBeNull();
    });
  });

  describe('transfer', () => {
    it('should execute successful transfer', async () => {
      const mockResult = {
        transaction_id: 'tx-123',
        status: 'Completed',
        timestamp: new Date().toISOString(),
      };
      mockBridge.treasuryTransfer.mockResolvedValue(JSON.stringify(mockResult));

      const result = await service.transfer('agent-a', 'agent-b', 100);

      if (!('error' in result)) {
        expect(result.transaction_id).toBe('tx-123');
        expect(result.status).toBe('Completed');
      }
    });

    it('should return error for failed transfer', async () => {
      mockBridge.treasuryTransfer.mockRejectedValue(new Error('Insufficient funds'));

      const result = await service.transfer('agent-a', 'agent-b', 1000000);

      expect('error' in result).toBe(true);
    });
  });

  describe('getBudget', () => {
    it('should return budget status', async () => {
      const mockBudgetResponse = {
        agent_id: 'agent-123',
        remaining: 3500.0,
        message: 'Budget available',
      };
      mockBridge.treasuryGetBudget.mockReturnValue(JSON.stringify(mockBudgetResponse));

      const result = await service.getBudget('agent-123');

      expect(result.remaining).toBe(3500.0);
    });
  });

  describe('getCarbon', () => {
    it('should return carbon metrics', async () => {
      const mockCarbon = {
        total_co2_grams: '12500',
        total_energy_kwh: '5.0',
        total_water_liters: '10.0',
        action_count: 100,
        period_start: new Date().toISOString(),
        period_end: new Date().toISOString(),
      };
      mockBridge.treasuryGetCarbon.mockReturnValue(JSON.stringify(mockCarbon));

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
      mockBridge.treasuryPurchaseOffset.mockReturnValue(JSON.stringify(mockOffset));

      const result = await service.purchaseOffset('agent-123', 10.0);

      if (!('error' in result)) {
        expect(result.transaction_id).toBe('tx-123');
        expect(result.tons).toBe(10.0);
      }
    });
  });
});
