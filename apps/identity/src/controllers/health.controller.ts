/**
 * Health Check Controller
 * 
 * Provides health check endpoints for monitoring and alerting.
 * Includes bridge health status for all pillar services.
 */

import { Controller, Get } from '@nestjs/common';
import { ApiTags, ApiOperation, ApiResponse } from '@nestjs/swagger';
import { GateService } from '../services/gate.service';
import { SynapseService } from '../services/synapse.service';
import { ArbiterService } from '../services/arbiter.service';
import { NexusService } from '../services/nexus.service';
import { TreasuryService } from '../services/treasury.service';

@ApiTags('Health')
@Controller('health')
export class HealthController {
  constructor(
    private readonly gateService: GateService,
    private readonly synapseService: SynapseService,
    private readonly arbiterService: ArbiterService,
    private readonly nexusService: NexusService,
    private readonly treasuryService: TreasuryService,
  ) {}

  @Get()
  @ApiOperation({ summary: 'Basic health check' })
  @ApiResponse({ status: 200, description: 'Service is healthy' })
  getHealth(): { status: string; timestamp: string } {
    return {
      status: 'healthy',
      timestamp: new Date().toISOString(),
    };
  }

  @Get('bridge')
  @ApiOperation({ summary: 'N-API Bridge health check' })
  @ApiResponse({
    status: 200,
    description: 'Bridge health status for all pillar services',
  })
  async getBridgeHealth(): Promise<{
    status: 'healthy' | 'degraded' | 'unavailable';
    services: {
      gate: boolean;
      synapse: boolean;
      arbiter: boolean;
      nexus: boolean;
      treasury: boolean;
    };
    timestamp: string;
  }> {
    const services = {
      gate: this.gateService.isOperational(),
      synapse: this.synapseService.isOperational(),
      arbiter: this.arbiterService.isOperational(),
      nexus: this.nexusService.isOperational(),
      treasury: this.treasuryService.isOperational(),
    };

    const operationalCount = Object.values(services).filter(Boolean).length;
    const totalServices = Object.keys(services).length;

    let status: 'healthy' | 'degraded' | 'unavailable';
    if (operationalCount === totalServices) {
      status = 'healthy';
    } else if (operationalCount > 0) {
      status = 'degraded';
    } else {
      status = 'unavailable';
    }

    return {
      status,
      services,
      timestamp: new Date().toISOString(),
    };
  }
}

