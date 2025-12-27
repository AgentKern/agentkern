/**
 * AgentKern Gateway - Identity Controller
 * 
 * REST API for agent identity operations.
 * Endpoints:
 * - POST /identity/register - Register a new agent
 * - GET /identity/:id - Get agent identity
 * - POST /identity/:id/sign - Sign an action
 * - POST /identity/verify - Verify a liability proof
 */

import { Controller, Get, Post, Body, Param, HttpCode, HttpStatus } from '@nestjs/common';
import { IdentityService } from '../services/identity.service';
import { RegisterAgentDto, SignActionDto, VerifyProofDto } from '../dto/identity.dto';

@Controller('identity')
export class IdentityController {
  constructor(private readonly identityService: IdentityService) {}

  @Post('register')
  @HttpCode(HttpStatus.CREATED)
  async register(@Body() dto: RegisterAgentDto) {
    return this.identityService.register(dto.name, dto.capabilities);
  }

  @Get(':id')
  async getIdentity(@Param('id') id: string) {
    return this.identityService.getIdentity(id);
  }

  @Post(':id/sign')
  async signAction(@Param('id') id: string, @Body() dto: SignActionDto) {
    return this.identityService.signAction(id, dto.action, dto.payload);
  }

  @Post('verify')
  async verifyProof(@Body() dto: VerifyProofDto) {
    return this.identityService.verifyProof(dto.proof);
  }

  @Get(':id/trust')
  async getTrustScore(@Param('id') id: string) {
    const score = await this.identityService.getTrustScore(id);
    return { agentId: id, trustScore: score };
  }
}
