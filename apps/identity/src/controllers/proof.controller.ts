/**
 * AgentKernIdentity - Proof Controller
 * 
 * REST API endpoints for Liability Proof operations.
 * Follows mandate: validation, error handling, logging, documentation.
 */

import {
  Controller,
  Post,
  Get,
  Body,
  Param,
  Query,
  Headers,
  Ip,
  HttpCode,
  HttpStatus,
  UnauthorizedException,
  BadRequestException,
} from '@nestjs/common';
import {
  ApiTags,
  ApiOperation,
  ApiResponse,
  ApiHeader,
  ApiParam,
  ApiQuery,
} from '@nestjs/swagger';
import { ProofVerificationService } from '../services/proof-verification.service';
import { ProofSigningService } from '../services/proof-signing.service';
import { AuditLoggerService, AuditEventType } from '../services/audit-logger.service';
import { AgentSandboxService } from '../services/agent-sandbox.service';
import {
  VerifyProofRequestDto,
  CreateProofRequestDto,
  RegisterKeyRequestDto,
  VerifyProofResponseDto,
  CreateProofResponseDto,
  AuditEventResponseDto,
} from '../dto/proof.dto';
import { serializeProofHeader } from '../domain/liability-proof.entity';

@ApiTags('Proof')
@Controller('api/v1/proof')
export class ProofController {
  constructor(
    private readonly verificationService: ProofVerificationService,
    private readonly signingService: ProofSigningService,
    private readonly auditLogger: AuditLoggerService,
    private readonly sandboxService: AgentSandboxService,
  ) {}

  @Post('verify')
  @HttpCode(HttpStatus.OK)
  @ApiOperation({
    summary: 'Verify a Liability Proof',
    description: 'Validates the cryptographic signature and constraints of a Liability Proof.',
  })
  @ApiResponse({ status: 200, description: 'Verification result', type: VerifyProofResponseDto })
  @ApiResponse({ status: 400, description: 'Invalid request' })
  async verifyProof(
    @Body() dto: VerifyProofRequestDto,
    @Ip() ipAddress: string,
    @Headers('user-agent') userAgent: string,
  ): Promise<VerifyProofResponseDto> {
    const result = await this.verificationService.verifyProof(dto.proof);

    // Log the verification attempt
    if (result.valid && result.agentId) {
      // Check sandbox constraints
      const sandboxResult = await this.sandboxService.checkAction({
        agentId: result.agentId,
        action: result.intent?.action || 'unknown',
        target: typeof result.intent?.target === 'string' 
          ? { service: result.intent.target, endpoint: '/', method: 'UNKNOWN' }
          : result.intent?.target || { service: 'unknown', endpoint: 'unknown', method: 'UNKNOWN' },
      });

      if (!sandboxResult.allowed) {
        this.auditLogger.logSecurityEvent(
          AuditEventType.SUSPICIOUS_ACTIVITY,
          `Agent action blocked by sandbox: ${sandboxResult.reason}`,
          { agentId: result.agentId, reason: sandboxResult.reason },
          { ipAddress, userAgent },
        );
        
        return {
          valid: false,
          errors: [`Agent action blocked by sandbox: ${sandboxResult.reason}`],
        };
      }

      this.auditLogger.logVerificationSuccess(
        result.proofId!,
        result.principalId!,
        result.agentId!,
        result.intent!.action,
        result.intent!.target,
        { ipAddress, userAgent },
      );

      // Record success in sandbox
      await this.sandboxService.recordSuccess(result.agentId);
    } else {
      this.auditLogger.logVerificationFailure(
        result.proofId,
        result.errors?.join(', ') || 'Unknown error',
        { ipAddress, userAgent },
      );
    }

    return result;
  }

  @Post('verify/header')
  @HttpCode(HttpStatus.OK)
  @ApiOperation({
    summary: 'Verify proof from X-AgentKernIdentity header',
    description: 'Extracts and verifies the Liability Proof from the X-AgentKernIdentity header.',
  })
  @ApiHeader({ name: 'X-AgentKernIdentity', description: 'The Liability Proof header', required: true })
  @ApiResponse({ status: 200, description: 'Verification result', type: VerifyProofResponseDto })
  @ApiResponse({ status: 401, description: 'Missing or invalid proof' })
  async verifyFromHeader(
    @Headers('x-agentkern-identity') proofHeader: string,
    @Ip() ipAddress: string,
    @Headers('user-agent') userAgent: string,
  ): Promise<VerifyProofResponseDto> {
    if (!proofHeader) {
      this.auditLogger.logSecurityEvent(
        AuditEventType.INVALID_INPUT,
        'Missing X-AgentKernIdentity header',
        {},
        { ipAddress, userAgent },
      );
      throw new UnauthorizedException('Missing X-AgentKernIdentity header');
    }

    return this.verifyProof({ proof: proofHeader }, ipAddress, userAgent);
  }

  @Post('register-key')
  @HttpCode(HttpStatus.CREATED)
  @ApiOperation({
    summary: 'Register a public key for verification',
    description: 'Registers a principal\'s public key for future proof verification.',
  })
  @ApiResponse({ status: 201, description: 'Key registered successfully' })
  @ApiResponse({ status: 400, description: 'Invalid key format' })
  async registerKey(
    @Body() dto: RegisterKeyRequestDto,
    @Ip() ipAddress: string,
    @Headers('user-agent') userAgent: string,
  ): Promise<{ success: boolean; message: string }> {
    try {
      await this.verificationService.registerPublicKey({
        principalId: dto.principalId,
        credentialId: dto.credentialId,
        publicKey: dto.publicKey,
        algorithm: dto.algorithm || 'ES256',
      });

      this.auditLogger.log({
        type: AuditEventType.KEY_REGISTERED,
        principalId: dto.principalId,
        success: true,
        ipAddress,
        userAgent,
        metadata: { credentialId: dto.credentialId },
      });

      return { success: true, message: 'Key registered successfully' };
    } catch (error) {
      this.auditLogger.logSecurityEvent(
        AuditEventType.INVALID_INPUT,
        `Key registration failed: ${error}`,
        { principalId: dto.principalId },
        { ipAddress, userAgent },
      );
      throw new BadRequestException('Invalid key format');
    }
  }

  @Post('create')
  @HttpCode(HttpStatus.CREATED)
  @ApiOperation({
    summary: 'Create a signed Liability Proof (Testing Only)',
    description: 'Creates a signed proof for testing. In production, signing happens client-side via WebAuthn.',
  })
  @ApiResponse({ status: 201, description: 'Proof created', type: CreateProofResponseDto })
  async createProof(
    @Body() dto: CreateProofRequestDto,
  ): Promise<CreateProofResponseDto> {
    // Generate a test key pair for demonstration
    const { publicKey, privateKey } = await this.signingService.generateKeyPair();

    // Register the public key
    await this.verificationService.registerPublicKey({
      principalId: dto.principal.id,
      credentialId: dto.principal.credentialId,
      publicKey,
      algorithm: 'ES256',
    });

    // Create the signed proof
    const proof = await this.signingService.createSignedProof({
      principal: dto.principal,
      agent: dto.agent,
      intent: {
        action: dto.intent.action,
        target: {
          service: dto.intent.target.service,
          endpoint: dto.intent.target.endpoint,
          method: dto.intent.target.method,
        },
        parameters: dto.intent.parameters,
      },
      constraints: dto.constraints,
      expiresInSeconds: dto.expiresInSeconds,
      privateKey,
    });

    return {
      header: serializeProofHeader(proof),
      proofId: proof.payload.proofId,
      expiresAt: proof.payload.expiresAt,
    };
  }

  @Get('audit/:principalId')
  @ApiOperation({
    summary: 'Get audit trail for a principal',
    description: 'Retrieves the verification audit history for a principal.',
  })
  @ApiParam({ name: 'principalId', description: 'Principal identifier' })
  @ApiQuery({ name: 'limit', required: false, description: 'Maximum events to return' })
  @ApiResponse({ status: 200, description: 'Audit events', type: [AuditEventResponseDto] })
  async getAuditTrail(
    @Param('principalId') principalId: string,
    @Query('limit') limit?: number,
  ): Promise<AuditEventResponseDto[]> {
    return this.auditLogger.getAuditTrailForPrincipal(principalId, limit || 100);
  }

  @Get('health')
  @ApiOperation({ summary: 'Health check endpoint' })
  @ApiResponse({ status: 200, description: 'Service is healthy' })
  healthCheck(): { status: string; timestamp: string } {
    return {
      status: 'healthy',
      timestamp: new Date().toISOString(),
    };
  }
}
