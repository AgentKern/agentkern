import {
  Controller,
  Post,
  Get,
  Put,
  Body,
  Param,
  HttpCode,
  HttpStatus,
  Logger,
} from '@nestjs/common';
import { ApiTags, ApiOperation, ApiResponse } from '@nestjs/swagger';
import {
  GetBalanceDto,
  BalanceResponseDto,
  DepositDto,
  TransferDto,
  TransferResponseDto,
  BudgetDto,
  SetBudgetDto,
  CarbonFootprintDto,
  CarbonOffsetDto,
} from '../dto/treasury.dto';

/**
 * Treasury Controller - Agent Payment Infrastructure API
 * 
 * Exposes the Treasury pillar's capabilities:
 * - Agent balance management
 * - Agent-to-agent transfers (micropayments)
 * - Spending budgets and limits
 * - Carbon footprint tracking
 */
@ApiTags('Treasury')
@Controller('api/v1/treasury')
export class TreasuryController {
  private readonly logger = new Logger(TreasuryController.name);

  // In-memory store for demo (would use Rust bridge in production)
  private balances: Map<string, { balance: number; currency: string }> = new Map();
  private budgets: Map<string, { limit: number; spent: number; period: string }> = new Map();
  private carbon: Map<string, { totalGrams: number; computeHours: number }> = new Map();

  // =========================================================================
  // Balance Endpoints
  // =========================================================================

  /**
   * Get agent balance
   */
  @Get('balance/:agentId')
  @ApiOperation({ summary: 'Get agent balance' })
  @ApiResponse({ status: 200, description: 'Agent balance', type: BalanceResponseDto })
  @ApiResponse({ status: 404, description: 'Agent not found' })
  async getBalance(@Param('agentId') agentId: string): Promise<BalanceResponseDto> {
    this.logger.log(`Getting balance for agent: ${agentId}`);
    
    const balance = this.balances.get(agentId) || { balance: 0, currency: 'USD' };
    
    return {
      agentId,
      balance: balance.balance,
      currency: balance.currency,
      lastUpdated: new Date().toISOString(),
    };
  }

  /**
   * Deposit funds to agent balance
   */
  @Post('balance/:agentId/deposit')
  @HttpCode(HttpStatus.OK)
  @ApiOperation({ summary: 'Deposit funds to agent balance' })
  @ApiResponse({ status: 200, description: 'Deposit successful', type: BalanceResponseDto })
  async deposit(
    @Param('agentId') agentId: string,
    @Body() dto: DepositDto,
  ): Promise<BalanceResponseDto> {
    this.logger.log(`Depositing ${dto.amount} to agent: ${agentId}`);
    
    const current = this.balances.get(agentId) || { balance: 0, currency: 'USD' };
    current.balance += dto.amount;
    this.balances.set(agentId, current);
    
    return {
      agentId,
      balance: current.balance,
      currency: current.currency,
      lastUpdated: new Date().toISOString(),
    };
  }

  // =========================================================================
  // Transfer Endpoints
  // =========================================================================

  /**
   * Transfer funds between agents (micropayment)
   */
  @Post('transfer')
  @HttpCode(HttpStatus.OK)
  @ApiOperation({ summary: 'Transfer funds between agents' })
  @ApiResponse({ status: 200, description: 'Transfer successful', type: TransferResponseDto })
  @ApiResponse({ status: 400, description: 'Insufficient funds or invalid request' })
  async transfer(@Body() dto: TransferDto): Promise<TransferResponseDto> {
    this.logger.log(`Transfer: ${dto.fromAgent} -> ${dto.toAgent}: ${dto.amount}`);
    
    const fromBalance = this.balances.get(dto.fromAgent) || { balance: 0, currency: 'USD' };
    const toBalance = this.balances.get(dto.toAgent) || { balance: 0, currency: 'USD' };
    
    if (fromBalance.balance < dto.amount) {
      return {
        transactionId: `tx_${Date.now()}`,
        status: 'failed',
        fromAgent: dto.fromAgent,
        toAgent: dto.toAgent,
        amount: dto.amount,
        error: 'Insufficient funds',
        timestamp: new Date().toISOString(),
      };
    }
    
    fromBalance.balance -= dto.amount;
    toBalance.balance += dto.amount;
    this.balances.set(dto.fromAgent, fromBalance);
    this.balances.set(dto.toAgent, toBalance);
    
    return {
      transactionId: `tx_${Date.now()}`,
      status: 'completed',
      fromAgent: dto.fromAgent,
      toAgent: dto.toAgent,
      amount: dto.amount,
      reference: dto.reference,
      timestamp: new Date().toISOString(),
    };
  }

  // =========================================================================
  // Budget Endpoints
  // =========================================================================

  /**
   * Get agent spending budget
   */
  @Get('budget/:agentId')
  @ApiOperation({ summary: 'Get agent spending budget' })
  @ApiResponse({ status: 200, description: 'Budget details', type: BudgetDto })
  async getBudget(@Param('agentId') agentId: string): Promise<BudgetDto> {
    const budget = this.budgets.get(agentId) || { limit: 100, spent: 0, period: 'daily' };
    
    return {
      agentId,
      limit: budget.limit,
      spent: budget.spent,
      remaining: budget.limit - budget.spent,
      period: budget.period as 'hourly' | 'daily' | 'weekly' | 'monthly',
    };
  }

  /**
   * Set agent spending budget limit
   */
  @Put('budget/:agentId')
  @ApiOperation({ summary: 'Set agent spending budget' })
  @ApiResponse({ status: 200, description: 'Budget updated', type: BudgetDto })
  async setBudget(
    @Param('agentId') agentId: string,
    @Body() dto: SetBudgetDto,
  ): Promise<BudgetDto> {
    this.logger.log(`Setting budget for ${agentId}: ${dto.limit}/${dto.period}`);
    
    const current = this.budgets.get(agentId) || { limit: 0, spent: 0, period: 'daily' };
    current.limit = dto.limit;
    current.period = dto.period;
    this.budgets.set(agentId, current);
    
    return {
      agentId,
      limit: current.limit,
      spent: current.spent,
      remaining: current.limit - current.spent,
      period: current.period as 'hourly' | 'daily' | 'weekly' | 'monthly',
    };
  }

  // =========================================================================
  // Carbon Footprint Endpoints
  // =========================================================================

  /**
   * Get agent carbon footprint
   */
  @Get('carbon/:agentId')
  @ApiOperation({ summary: 'Get agent carbon footprint' })
  @ApiResponse({ status: 200, description: 'Carbon footprint', type: CarbonFootprintDto })
  async getCarbonFootprint(@Param('agentId') agentId: string): Promise<CarbonFootprintDto> {
    const footprint = this.carbon.get(agentId) || { totalGrams: 0, computeHours: 0 };
    
    return {
      agentId,
      totalGramsCO2: footprint.totalGrams,
      computeHours: footprint.computeHours,
      region: 'us-east-1',
      carbonIntensity: 400, // gCO2/kWh
      lastUpdated: new Date().toISOString(),
    };
  }

  /**
   * Purchase carbon offset
   */
  @Post('carbon/offset')
  @HttpCode(HttpStatus.OK)
  @ApiOperation({ summary: 'Purchase carbon offset for agent' })
  @ApiResponse({ status: 200, description: 'Offset purchased' })
  async purchaseOffset(@Body() dto: CarbonOffsetDto): Promise<{ success: boolean; offsetGrams: number; cost: number }> {
    this.logger.log(`Purchasing ${dto.grams}g CO2 offset for ${dto.agentId}`);
    
    // Simulate offset cost ($0.02 per kg CO2)
    const cost = (dto.grams / 1000) * 0.02;
    
    const footprint = this.carbon.get(dto.agentId) || { totalGrams: 0, computeHours: 0 };
    footprint.totalGrams = Math.max(0, footprint.totalGrams - dto.grams);
    this.carbon.set(dto.agentId, footprint);
    
    return {
      success: true,
      offsetGrams: dto.grams,
      cost,
    };
  }
}
