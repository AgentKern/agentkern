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
  BalanceResponseDto,
  DepositDto,
  TransferDto,
  TransferResponseDto,
  BudgetDto,
  SetBudgetDto,
  CarbonFootprintDto,
  CarbonOffsetDto,
} from '../dto/treasury.dto';
import { TreasuryService } from '../services/treasury.service';

/**
 * Treasury Controller - Agent Payment Infrastructure API
 *
 * Exposes the Treasury pillar's capabilities via Rust N-API bridge:
 * - Agent balance management
 * - Agent-to-agent transfers (micropayments)
 * - Spending budgets and limits
 * - Carbon footprint tracking
 */
@ApiTags('Treasury')
@Controller('api/v1/treasury')
export class TreasuryController {
  private readonly logger = new Logger(TreasuryController.name);

  constructor(private readonly treasuryService: TreasuryService) {}

  // =========================================================================
  // Balance Endpoints
  // =========================================================================

  /**
   * Get agent balance
   */
  @Get('balance/:agentId')
  @ApiOperation({ summary: 'Get agent balance' })
  @ApiResponse({
    status: 200,
    description: 'Agent balance',
    type: BalanceResponseDto,
  })
  @ApiResponse({ status: 404, description: 'Agent not found' })
  async getBalance(
    @Param('agentId') agentId: string,
  ): Promise<BalanceResponseDto> {
    this.logger.log(`Getting balance for agent: ${agentId}`);

    const result = await this.treasuryService.getBalance(agentId);

    if (!result) {
      // Service returned null (bridge not loaded or error)
      return {
        agentId,
        balance: 0,
        currency: 'VMC',
        lastUpdated: new Date().toISOString(),
      };
    }

    return {
      agentId: result.agent_id,
      balance: result.balance.value / Math.pow(10, result.balance.decimals),
      currency: result.currency,
      lastUpdated: result.updated_at,
    };
  }

  /**
   * Deposit funds to agent balance
   */
  @Post('balance/:agentId/deposit')
  @HttpCode(HttpStatus.OK)
  @ApiOperation({ summary: 'Deposit funds to agent balance' })
  @ApiResponse({
    status: 200,
    description: 'Deposit successful',
    type: BalanceResponseDto,
  })
  async deposit(
    @Param('agentId') agentId: string,
    @Body() dto: DepositDto,
  ): Promise<BalanceResponseDto> {
    this.logger.log(`Depositing ${dto.amount} to agent: ${agentId}`);

    const result = await this.treasuryService.deposit(agentId, dto.amount);

    if ('error' in result) {
      this.logger.error(`Deposit failed: ${result.error}`);
      return {
        agentId,
        balance: 0,
        currency: 'VMC',
        lastUpdated: new Date().toISOString(),
      };
    }

    return {
      agentId: result.agent_id,
      balance: result.balance.value / Math.pow(10, result.balance.decimals),
      currency: result.currency,
      lastUpdated: result.updated_at,
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
  @ApiResponse({
    status: 200,
    description: 'Transfer successful',
    type: TransferResponseDto,
  })
  @ApiResponse({
    status: 400,
    description: 'Insufficient funds or invalid request',
  })
  async transfer(@Body() dto: TransferDto): Promise<TransferResponseDto> {
    this.logger.log(
      `Transfer: ${dto.fromAgent} -> ${dto.toAgent}: ${dto.amount}`,
    );

    const result = await this.treasuryService.transfer(
      dto.fromAgent,
      dto.toAgent,
      dto.amount,
      dto.reference,
    );

    if ('error' in result) {
      return {
        transactionId: `tx_${Date.now()}`,
        status: 'failed',
        fromAgent: dto.fromAgent,
        toAgent: dto.toAgent,
        amount: dto.amount,
        error: result.error,
        timestamp: new Date().toISOString(),
      };
    }

    return {
      transactionId: result.transaction_id,
      status: result.status === 'Completed' ? 'completed' : 'pending',
      fromAgent: dto.fromAgent,
      toAgent: dto.toAgent,
      amount: dto.amount,
      reference: dto.reference,
      timestamp: result.timestamp,
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
    const result = await this.treasuryService.getBudget(agentId);

    return {
      agentId: result.agent_id,
      limit: 100, // Default limit (budget manager returns remaining only)
      spent: result.remaining !== null ? 100 - result.remaining : 0,
      remaining: result.remaining ?? 100,
      period: 'daily',
    };
  }

  /**
   * Set agent spending budget limit
   * Note: Budget setting requires bridge extension; returns current state for now.
   */
  @Put('budget/:agentId')
  @ApiOperation({ summary: 'Set agent spending budget' })
  @ApiResponse({ status: 200, description: 'Budget updated', type: BudgetDto })
  async setBudget(
    @Param('agentId') agentId: string,
    @Body() dto: SetBudgetDto,
  ): Promise<BudgetDto> {
    this.logger.log(
      `Setting budget for ${agentId}: ${dto.limit}/${dto.period}`,
    );

    // Note: Bridge currently only supports getBudget, not setBudget.
    // This would require extending the Rust BudgetManager to support set_budget().
    // For now, return the requested values as acknowledgment.
    this.logger.warn(
      'Budget setting not yet implemented in bridge; returning requested values',
    );

    return {
      agentId,
      limit: dto.limit,
      spent: 0,
      remaining: dto.limit,
      period: dto.period,
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
  @ApiResponse({
    status: 200,
    description: 'Carbon footprint',
    type: CarbonFootprintDto,
  })
  async getCarbonFootprint(
    @Param('agentId') agentId: string,
  ): Promise<CarbonFootprintDto> {
    const result = await this.treasuryService.getCarbon(agentId);

    if (!result) {
      return {
        agentId,
        totalGramsCO2: 0,
        computeHours: 0,
        region: 'us-east-1',
        carbonIntensity: 400,
        lastUpdated: new Date().toISOString(),
      };
    }

    return {
      agentId,
      totalGramsCO2: parseFloat(result.total_co2_grams),
      computeHours: parseFloat(result.total_energy_kwh),
      region: 'us-east-1',
      carbonIntensity: 400, // gCO2/kWh
      lastUpdated: result.period_end,
    };
  }

  /**
   * Purchase carbon offset
   */
  @Post('carbon/offset')
  @HttpCode(HttpStatus.OK)
  @ApiOperation({ summary: 'Purchase carbon offset for agent' })
  @ApiResponse({ status: 200, description: 'Offset purchased' })
  async purchaseOffset(
    @Body() dto: CarbonOffsetDto,
  ): Promise<{ success: boolean; offsetGrams: number; cost: number }> {
    this.logger.log(`Purchasing ${dto.grams}g CO2 offset for ${dto.agentId}`);

    // Convert grams to tons for the service
    const tons = dto.grams / 1_000_000;
    const result = await this.treasuryService.purchaseOffset(
      dto.agentId,
      tons,
    );

    if ('error' in result) {
      this.logger.error(`Offset purchase failed: ${result.error}`);
      return {
        success: false,
        offsetGrams: 0,
        cost: 0,
      };
    }

    return {
      success: true,
      offsetGrams: dto.grams,
      cost: result.cost,
    };
  }
}
