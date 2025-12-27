/**
 * AgentKern Gateway - Health Controller
 * 
 * Basic health check endpoint for load balancers and monitoring.
 */

import { Controller, Get } from '@nestjs/common';

@Controller('health')
export class HealthController {
  @Get()
  check() {
    return {
      status: 'healthy',
      version: '0.1.0',
      timestamp: new Date().toISOString(),
      pillars: {
        identity: 'ready',
        gate: 'ready',
        synapse: 'ready',
        arbiter: 'ready',
      },
    };
  }

  @Get('ready')
  ready() {
    return { ready: true };
  }

  @Get('live')
  live() {
    return { live: true };
  }
}
