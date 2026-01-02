import { ApiProperty, ApiPropertyOptional } from '@nestjs/swagger';
import { IsString, IsNumber, IsOptional, IsEnum, Min } from 'class-validator';

// ============================================================================
// Balance DTOs
// ============================================================================

export class GetBalanceDto {
  @ApiProperty({ description: 'Agent ID' })
  @IsString()
  agentId: string;
}

export class BalanceResponseDto {
  @ApiProperty({ description: 'Agent ID' })
  @IsString()
  agentId: string;

  @ApiProperty({ description: 'Current balance' })
  @IsNumber()
  balance: number;

  @ApiProperty({ description: 'Currency code', default: 'USD' })
  @IsString()
  currency: string;

  @ApiProperty({ description: 'Last update timestamp' })
  @IsString()
  lastUpdated: string;
}

export class DepositDto {
  @ApiProperty({ description: 'Amount to deposit' })
  @IsNumber()
  @Min(0.001)
  amount: number;

  @ApiPropertyOptional({ description: 'Deposit reference' })
  @IsOptional()
  @IsString()
  reference?: string;
}

// ============================================================================
// Transfer DTOs
// ============================================================================

export class TransferDto {
  @ApiProperty({ description: 'Source agent ID' })
  @IsString()
  fromAgent: string;

  @ApiProperty({ description: 'Destination agent ID' })
  @IsString()
  toAgent: string;

  @ApiProperty({ description: 'Transfer amount' })
  @IsNumber()
  @Min(0.001)
  amount: number;

  @ApiPropertyOptional({
    description: 'Transfer reference (e.g., service call ID)',
  })
  @IsOptional()
  @IsString()
  reference?: string;
}

export class TransferResponseDto {
  @ApiProperty({ description: 'Transaction ID' })
  @IsString()
  transactionId: string;

  @ApiProperty({
    description: 'Transaction status',
    enum: ['pending', 'completed', 'failed'],
  })
  @IsString()
  status: 'pending' | 'completed' | 'failed';

  @ApiProperty({ description: 'Source agent ID' })
  @IsString()
  fromAgent: string;

  @ApiProperty({ description: 'Destination agent ID' })
  @IsString()
  toAgent: string;

  @ApiProperty({ description: 'Transfer amount' })
  @IsNumber()
  amount: number;

  @ApiPropertyOptional({ description: 'Transfer reference' })
  @IsOptional()
  @IsString()
  reference?: string;

  @ApiPropertyOptional({ description: 'Error message if failed' })
  @IsOptional()
  @IsString()
  error?: string;

  @ApiProperty({ description: 'Transaction timestamp' })
  @IsString()
  timestamp: string;
}

// ============================================================================
// Budget DTOs
// ============================================================================

export class BudgetDto {
  @ApiProperty({ description: 'Agent ID' })
  @IsString()
  agentId: string;

  @ApiProperty({ description: 'Spending limit' })
  @IsNumber()
  limit: number;

  @ApiProperty({ description: 'Amount spent in current period' })
  @IsNumber()
  spent: number;

  @ApiProperty({ description: 'Remaining budget' })
  @IsNumber()
  remaining: number;

  @ApiProperty({
    description: 'Budget period',
    enum: ['hourly', 'daily', 'weekly', 'monthly'],
  })
  @IsString()
  period: 'hourly' | 'daily' | 'weekly' | 'monthly';
}

export class SetBudgetDto {
  @ApiProperty({ description: 'Spending limit' })
  @IsNumber()
  @Min(0)
  limit: number;

  @ApiProperty({
    description: 'Budget period',
    enum: ['hourly', 'daily', 'weekly', 'monthly'],
  })
  @IsEnum(['hourly', 'daily', 'weekly', 'monthly'])
  period: 'hourly' | 'daily' | 'weekly' | 'monthly';
}

// ============================================================================
// Carbon DTOs
// ============================================================================

export class CarbonFootprintDto {
  @ApiProperty({ description: 'Agent ID' })
  @IsString()
  agentId: string;

  @ApiProperty({ description: 'Total grams of CO2' })
  @IsNumber()
  totalGramsCO2: number;

  @ApiProperty({ description: 'Compute hours consumed' })
  @IsNumber()
  computeHours: number;

  @ApiProperty({ description: 'Primary compute region' })
  @IsString()
  region: string;

  @ApiProperty({ description: 'Carbon intensity (gCO2/kWh)' })
  @IsNumber()
  carbonIntensity: number;

  @ApiProperty({ description: 'Last update timestamp' })
  @IsString()
  lastUpdated: string;
}

export class CarbonOffsetDto {
  @ApiProperty({ description: 'Agent ID' })
  @IsString()
  agentId: string;

  @ApiProperty({ description: 'Grams of CO2 to offset' })
  @IsNumber()
  @Min(1)
  grams: number;

  @ApiPropertyOptional({ description: 'Offset provider preference' })
  @IsOptional()
  @IsString()
  provider?: string;
}
