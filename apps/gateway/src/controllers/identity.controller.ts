/**
 * AgentKern Gateway - Identity Controller
 * 
 * REST API for agent identity operations.
 * Proxies requests to the AgentKern Identity service.
 * 
 * Endpoints:
 * - POST /identity/register - Register a new agent
 * - GET /identity/:id - Get agent identity
 * - POST /identity/:id/sign - Sign an action
 * - POST /identity/verify - Verify a liability proof
 */

import { Controller, Get, Post, Body, Param, HttpCode, HttpStatus } from '@nestjs/common';
import { HttpService } from '@nestjs/axios';
import { firstValueFrom } from 'rxjs';
import { RegisterAgentDto, SignActionDto, VerifyProofDto } from '../dto/identity.dto';

@Controller('identity')
export class IdentityController {
  constructor(private readonly httpService: HttpService) {}

  @Post('register')
  @HttpCode(HttpStatus.CREATED)
  async register(@Body() dto: RegisterAgentDto) {
    const response = await firstValueFrom(
      this.httpService.post('/api/v1/identity/register', {
        name: dto.name,
        capabilities: dto.capabilities,
      }),
    );
    return response.data;
  }

  @Get(':id')
  async getIdentity(@Param('id') id: string) {
    const response = await firstValueFrom(
      this.httpService.get(`/api/v1/identity/${id}`),
    );
    return response.data;
  }

  @Post(':id/sign')
  async signAction(@Param('id') id: string, @Body() dto: SignActionDto) {
    const response = await firstValueFrom(
      this.httpService.post(`/api/v1/identity/${id}/sign`, {
        action: dto.action,
        payload: dto.payload,
      }),
    );
    return response.data;
  }

  @Post('verify')
  async verifyProof(@Body() dto: VerifyProofDto) {
    const response = await firstValueFrom(
      this.httpService.post('/api/v1/proof/verify', { proof: dto.proof }),
    );
    return response.data;
  }

  @Get(':id/trust')
  async getTrustScore(@Param('id') id: string) {
    const response = await firstValueFrom(
      this.httpService.get(`/api/v1/identity/${id}/trust`),
    );
    return response.data;
  }
}
