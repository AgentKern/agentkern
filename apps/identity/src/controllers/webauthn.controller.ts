/**
 * AgentKernIdentity - WebAuthn Controller
 * 
 * API endpoints for Passkey registration and authentication.
 */

import { Controller, Post, Get, Body, Param, HttpCode, HttpStatus } from '@nestjs/common';
import { ApiTags, ApiOperation, ApiResponse } from '@nestjs/swagger';
import { WebAuthnService } from '../services/webauthn.service';
import {
  StartRegistrationRequestDto,
  VerifyRegistrationRequestDto,
  StartAuthenticationRequestDto,
  VerifyAuthenticationRequestDto,
  RegistrationOptionsResponseDto,
  AuthenticationOptionsResponseDto,
  VerificationResultDto,
} from '../dto/webauthn.dto';

@ApiTags('WebAuthn')
@Controller('api/v1/webauthn')
export class WebAuthnController {
  constructor(private readonly webAuthnService: WebAuthnService) {}

  @Post('register/start')
  @HttpCode(HttpStatus.OK)
  @ApiOperation({
    summary: 'Start Passkey registration',
    description: 'Generates registration options for a new Passkey.',
  })
  @ApiResponse({ status: 200, description: 'Registration options' })
  async startRegistration(
    @Body() dto: StartRegistrationRequestDto,
  ): Promise<RegistrationOptionsResponseDto> {
    const options = await this.webAuthnService.generateRegistrationOptions(
      dto.principalId,
      dto.userName,
      dto.displayName || dto.userName,
    );
    return { options };
  }

  @Post('register/verify')
  @HttpCode(HttpStatus.OK)
  @ApiOperation({
    summary: 'Verify Passkey registration',
    description: 'Verifies the registration response and stores the credential.',
  })
  @ApiResponse({ status: 200, type: VerificationResultDto })
  async verifyRegistration(
    @Body() dto: VerifyRegistrationRequestDto,
  ): Promise<VerificationResultDto> {
    return this.webAuthnService.verifyRegistration(dto.principalId, dto.response);
  }

  @Post('authenticate/start')
  @HttpCode(HttpStatus.OK)
  @ApiOperation({
    summary: 'Start Passkey authentication',
    description: 'Generates authentication options for an existing Passkey.',
  })
  @ApiResponse({ status: 200, description: 'Authentication options' })
  async startAuthentication(
    @Body() dto: StartAuthenticationRequestDto,
  ): Promise<AuthenticationOptionsResponseDto | { error: string }> {
    const options = await this.webAuthnService.generateAuthenticationOptions(
      dto.principalId,
    );
    if (!options) {
      return { error: 'No credentials found for principal' };
    }
    return { options };
  }

  @Post('authenticate/verify')
  @HttpCode(HttpStatus.OK)
  @ApiOperation({
    summary: 'Verify Passkey authentication',
    description: 'Verifies the authentication response.',
  })
  @ApiResponse({ status: 200, type: VerificationResultDto })
  async verifyAuthentication(
    @Body() dto: VerifyAuthenticationRequestDto,
  ): Promise<VerificationResultDto> {
    return this.webAuthnService.verifyAuthentication(dto.principalId, dto.response);
  }

  @Get('credentials/:principalId')
  @ApiOperation({
    summary: 'Get credentials for a principal',
    description: 'Returns all registered credentials for a principal.',
  })
  @ApiResponse({ status: 200, description: 'List of credentials' })
  getCredentials(@Param('principalId') principalId: string) {
    const credentials = this.webAuthnService.getCredentials(principalId);
    return {
      principalId,
      credentials: credentials.map((c) => ({
        id: c.id,
        deviceType: c.credentialDeviceType,
        backedUp: c.credentialBackedUp,
      })),
    };
  }
}
