import {
  Controller,
  Post,
  Get,
  Body,
  Param,
  HttpCode,
  HttpStatus,
  UseGuards,
  Logger,
} from '@nestjs/common';
import { ApiTags, ApiOperation, ApiResponse, ApiBearerAuth } from '@nestjs/swagger';
import { GateService } from '../services/gate.service';
import {
  GuardPromptDto,
  GuardPromptResponseDto,
  PolicyDto,
  CreatePolicyDto,
  ComplianceCheckDto,
  ComplianceResultDto,
  WasmActorDto,
  RegisterWasmActorDto,
} from '../dto/gate.dto';

/**
 * Gate Controller - Security & Policy Enforcement API
 * 
 * Exposes the Gate pillar's capabilities:
 * - Prompt injection detection
 * - Policy management (CRUD)
 * - Compliance checks (PCI, HIPAA, GDPR)
 * - WASM actor management (hot-swap policies)
 */
@ApiTags('Gate')
@Controller('api/v1/gate')
export class GateController {
  private readonly logger = new Logger(GateController.name);

  constructor(private readonly gateService: GateService) {}

  // =========================================================================
  // Prompt Guard Endpoints
  // =========================================================================

  /**
   * Check a prompt for potential injection attacks
   */
  @Post('guard')
  @HttpCode(HttpStatus.OK)
  @ApiOperation({ summary: 'Check prompt for injection attacks' })
  @ApiResponse({ status: 200, description: 'Prompt analysis result', type: GuardPromptResponseDto })
  @ApiResponse({ status: 400, description: 'Invalid request' })
  async guardPrompt(@Body() dto: GuardPromptDto): Promise<GuardPromptResponseDto> {
    this.logger.log(`Analyzing prompt for injection: ${dto.prompt.substring(0, 50)}...`);
    
    const result = await this.gateService.analyzePrompt(dto.prompt, dto.context);
    
    if (!result.safe) {
      this.logger.warn(`Threat detected: ${result.threatType} (score: ${result.score})`);
    }
    
    return result;
  }

  // =========================================================================
  // Policy Management Endpoints
  // =========================================================================

  /**
   * List all active policies
   */
  @Get('policies')
  @ApiOperation({ summary: 'List all active policies' })
  @ApiResponse({ status: 200, description: 'List of policies', type: [PolicyDto] })
  async listPolicies(): Promise<PolicyDto[]> {
    return this.gateService.listPolicies();
  }

  /**
   * Get a specific policy by ID
   */
  @Get('policies/:id')
  @ApiOperation({ summary: 'Get policy by ID' })
  @ApiResponse({ status: 200, description: 'Policy details', type: PolicyDto })
  @ApiResponse({ status: 404, description: 'Policy not found' })
  async getPolicy(@Param('id') id: string): Promise<PolicyDto> {
    return this.gateService.getPolicy(id);
  }

  /**
   * Create a new policy
   */
  @Post('policies')
  @HttpCode(HttpStatus.CREATED)
  @ApiOperation({ summary: 'Create a new policy' })
  @ApiResponse({ status: 201, description: 'Policy created', type: PolicyDto })
  @ApiResponse({ status: 400, description: 'Invalid policy definition' })
  async createPolicy(@Body() dto: CreatePolicyDto): Promise<PolicyDto> {
    this.logger.log(`Creating policy: ${dto.name}`);
    return this.gateService.createPolicy(dto);
  }

  // =========================================================================
  // Compliance Check Endpoints
  // =========================================================================

  /**
   * Check PCI-DSS compliance
   */
  @Post('compliance/pci')
  @HttpCode(HttpStatus.OK)
  @ApiOperation({ summary: 'Check PCI-DSS compliance' })
  @ApiResponse({ status: 200, description: 'Compliance result', type: ComplianceResultDto })
  async checkPciCompliance(@Body() dto: ComplianceCheckDto): Promise<ComplianceResultDto> {
    this.logger.log('Running PCI-DSS compliance check');
    return this.gateService.checkPciCompliance(dto.data, dto.context);
  }

  /**
   * Check HIPAA compliance
   */
  @Post('compliance/hipaa')
  @HttpCode(HttpStatus.OK)
  @ApiOperation({ summary: 'Check HIPAA compliance' })
  @ApiResponse({ status: 200, description: 'Compliance result', type: ComplianceResultDto })
  async checkHipaaCompliance(@Body() dto: ComplianceCheckDto): Promise<ComplianceResultDto> {
    this.logger.log('Running HIPAA compliance check');
    return this.gateService.checkHipaaCompliance(dto.data, dto.context);
  }

  /**
   * Check GDPR compliance (data sovereignty)
   */
  @Post('compliance/gdpr')
  @HttpCode(HttpStatus.OK)
  @ApiOperation({ summary: 'Check GDPR data sovereignty compliance' })
  @ApiResponse({ status: 200, description: 'Compliance result', type: ComplianceResultDto })
  async checkGdprCompliance(@Body() dto: ComplianceCheckDto): Promise<ComplianceResultDto> {
    this.logger.log('Running GDPR compliance check');
    return this.gateService.checkGdprCompliance(dto.data, dto.context);
  }

  // =========================================================================
  // WASM Actor Management Endpoints
  // =========================================================================

  /**
   * List all WASM actors (hot-swappable policies)
   */
  @Get('wasm/actors')
  @ApiOperation({ summary: 'List all WASM policy actors' })
  @ApiResponse({ status: 200, description: 'List of WASM actors', type: [WasmActorDto] })
  async listWasmActors(): Promise<WasmActorDto[]> {
    return this.gateService.listWasmActors();
  }

  /**
   * Get WASM actor details
   */
  @Get('wasm/actors/:name')
  @ApiOperation({ summary: 'Get WASM actor by name' })
  @ApiResponse({ status: 200, description: 'WASM actor details', type: WasmActorDto })
  @ApiResponse({ status: 404, description: 'Actor not found' })
  async getWasmActor(@Param('name') name: string): Promise<WasmActorDto> {
    return this.gateService.getWasmActor(name);
  }

  /**
   * Register a new WASM actor (hot-swap)
   */
  @Post('wasm/actors')
  @HttpCode(HttpStatus.CREATED)
  @ApiOperation({ summary: 'Register or hot-swap a WASM policy actor' })
  @ApiResponse({ status: 201, description: 'Actor registered', type: WasmActorDto })
  @ApiResponse({ status: 400, description: 'Invalid WASM module' })
  async registerWasmActor(@Body() dto: RegisterWasmActorDto): Promise<WasmActorDto> {
    this.logger.log(`Registering WASM actor: ${dto.name} v${dto.version}`);
    return this.gateService.registerWasmActor(dto);
  }
}
