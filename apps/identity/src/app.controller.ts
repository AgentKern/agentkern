import { Controller, Get, Redirect } from '@nestjs/common';
import { ApiExcludeController } from '@nestjs/swagger';

@ApiExcludeController()
@Controller()
export class AppController {
  @Get()
  getRoot() {
    return {
      name: 'AgentKernIdentity API',
      description: 'Liability Infrastructure for the Agentic Economy',
      version: '1.0.0',
      docs: '/docs',
      endpoints: {
        proof: '/api/v1/proof',
        dns: '/api/v1/dns',
        mesh: '/api/v1/mesh',
        dashboard: '/api/v1/dashboard',
      },
    };
  }

  @Get('health')
  getHealth() {
    return { status: 'healthy', timestamp: new Date().toISOString() };
  }
}
