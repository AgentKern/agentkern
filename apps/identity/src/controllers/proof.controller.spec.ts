/**
 * Proof Controller Unit Tests
 */

import { Test, TestingModule } from '@nestjs/testing';
import { ProofController } from './proof.controller';
import { ProofVerificationService } from '../services/proof-verification.service';
import { ProofSigningService } from '../services/proof-signing.service';
import { AuditLoggerService } from '../services/audit-logger.service';
import { AgentSandboxService } from '../services/agent-sandbox.service';

describe('ProofController', () => {
  let controller: ProofController;
  let verificationService: ProofVerificationService;
  let signingService: ProofSigningService;
  let auditLogger: AuditLoggerService;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      controllers: [ProofController],
      providers: [
        {
          provide: ProofVerificationService,
          useValue: {
            verifyProof: jest.fn().mockResolvedValue({
              valid: true,
              proofId: 'test-proof-id',
              principalId: 'principal-1',
              agentId: 'agent-1',
              intent: { action: 'test', target: '/test' },
            }),
            registerPublicKey: jest.fn(),
          },
        },
        {
          provide: ProofSigningService,
          useValue: {
            generateKeyPair: jest.fn().mockResolvedValue({
              publicKey: 'test-public-key',
              privateKey: 'test-private-key',
            }),
            createSignedProof: jest.fn().mockResolvedValue({
              version: 'v1',
              payload: {
                proofId: 'generated-proof-id',
                expiresAt: new Date().toISOString(),
              },
              signature: 'test-signature',
            }),
          },
        },
        {
          provide: AuditLoggerService,
          useValue: {
            logVerificationSuccess: jest.fn(),
            logVerificationFailure: jest.fn(),
            logSecurityEvent: jest.fn(),
            log: jest.fn(),
            getAuditTrailForPrincipal: jest.fn().mockReturnValue([]),
          },
        },
        {
          provide: AgentSandboxService,
          useValue: {
            checkAction: jest.fn().mockResolvedValue({ allowed: true, agentStatus: 'ACTIVE' }),
            recordSuccess: jest.fn(),
            recordFailure: jest.fn(),
          },
        },
      ],
    }).compile();

    controller = module.get<ProofController>(ProofController);
    verificationService = module.get<ProofVerificationService>(ProofVerificationService);
    signingService = module.get<ProofSigningService>(ProofSigningService);
    auditLogger = module.get<AuditLoggerService>(AuditLoggerService);
  });

  describe('healthCheck', () => {
    it('should return healthy status', () => {
      const result = controller.healthCheck();
      expect(result.status).toBe('healthy');
      expect(result.timestamp).toBeDefined();
    });
  });

  describe('verifyProof', () => {
    it('should verify a valid proof', async () => {
      const result = await controller.verifyProof(
        { proof: 'AgentKernIdentity v1.xxx.yyy' },
        '127.0.0.1',
        'test-agent',
      );

      expect(result.valid).toBe(true);
      expect(verificationService.verifyProof).toHaveBeenCalled();
      expect(auditLogger.logVerificationSuccess).toHaveBeenCalled();
    });

    it('should log failure for invalid proof', async () => {
      jest.spyOn(verificationService, 'verifyProof').mockResolvedValue({
        valid: false,
        errors: ['Invalid signature'],
      });

      const result = await controller.verifyProof(
        { proof: 'invalid' },
        '127.0.0.1',
        'test-agent',
      );

      expect(result.valid).toBe(false);
      expect(auditLogger.logVerificationFailure).toHaveBeenCalled();
    });
  });

  describe('registerKey', () => {
    it('should register a public key', async () => {
      const result = await controller.registerKey(
        {
          principalId: 'principal-1',
          publicKey: 'test-key',
          credentialId: 'cred-1',
        },
        '127.0.0.1',
        'test-agent',
      );

      expect(result.success).toBe(true);
      expect(verificationService.registerPublicKey).toHaveBeenCalled();
    });
  });

  describe('createProof', () => {
    it('should create a signed proof', async () => {
      const result = await controller.createProof({
        principal: { id: 'principal-1', credentialId: 'cred-1' },
        agent: { id: 'agent-1', name: 'Test Agent', version: '1.0.0' },
        intent: {
          action: 'test',
          target: { service: 'api', endpoint: '/test', method: 'GET' },
        },
      });

      expect(result.proofId).toBeDefined();
      expect(signingService.generateKeyPair).toHaveBeenCalled();
      expect(signingService.createSignedProof).toHaveBeenCalled();
    });
  });

  describe('getAuditTrail', () => {
    it('should return audit trail for principal', async () => {
      const result = await controller.getAuditTrail('principal-1', 100);
      expect(Array.isArray(result)).toBe(true);
      expect(auditLogger.getAuditTrailForPrincipal).toHaveBeenCalledWith('principal-1', 100);
    });
  });
});
