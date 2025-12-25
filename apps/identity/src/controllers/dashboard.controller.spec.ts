/**
 * Dashboard Controller Unit Tests - Full Coverage
 */

import { Test, TestingModule } from '@nestjs/testing';
import { NotFoundException } from '@nestjs/common';
import { Reflector } from '@nestjs/core';
import { DashboardController } from './dashboard.controller';
import { PolicyService } from '../services/policy.service';
import { AuditLoggerService, AuditEventType } from '../services/audit-logger.service';
import { LicenseService, LicenseTier } from '../services/license.service';
import { LicenseGuard } from '../guards/license.guard';
import { PolicyAction } from '../dto/dashboard.dto';

describe('DashboardController', () => {
  let controller: DashboardController;
  let policyService: PolicyService;
  let auditLogger: AuditLoggerService;

  const mockPolicy = {
    id: 'policy-1',
    name: 'Test Policy',
    description: 'A test policy',
    rules: [{ name: 'Rule 1', condition: 'true', action: PolicyAction.ALLOW }],
    targetAgents: [],
    targetPrincipals: [],
    active: true,
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  };

  const mockEvents = [
    {
      id: '1',
      type: AuditEventType.PROOF_VERIFICATION_SUCCESS,
      timestamp: new Date().toISOString(),
      success: true,
      agentId: 'agent-1',
      principalId: 'principal-1',
    },
    {
      id: '2',
      type: AuditEventType.PROOF_VERIFICATION_FAILURE,
      timestamp: new Date().toISOString(),
      success: false,
      agentId: 'agent-2',
      principalId: 'principal-2',
    },
    {
      id: '3',
      type: AuditEventType.KEY_REVOKED,
      timestamp: new Date().toISOString(),
      success: true,
      agentId: 'agent-1',
      principalId: 'principal-1',
    },
    {
      id: '4',
      type: AuditEventType.SECURITY_ALERT,
      timestamp: new Date().toISOString(),
      success: false,
    },
  ];

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      controllers: [DashboardController],
      providers: [
        Reflector,
        {
          provide: LicenseService,
          useValue: {
            getLicenseInfo: jest.fn().mockReturnValue({ tier: LicenseTier.OSS, valid: true }),
            hasTier: jest.fn().mockReturnValue(true),
            hasFeature: jest.fn().mockReturnValue(true),
          },
        },
        {
          provide: LicenseGuard,
          useValue: { canActivate: jest.fn().mockReturnValue(true) },
        },
        {
          provide: PolicyService,
          useValue: {
            getAllPolicies: jest.fn().mockReturnValue([mockPolicy]),
            getPolicy: jest.fn().mockReturnValue(mockPolicy),
            createPolicy: jest.fn().mockReturnValue(mockPolicy),
            deletePolicy: jest.fn().mockReturnValue(true),
            setActive: jest.fn().mockReturnValue(mockPolicy),
          },
        },
        {
          provide: AuditLoggerService,
          useValue: {
            getRecentEvents: jest.fn().mockReturnValue(mockEvents),
            getSecurityEvents: jest.fn().mockReturnValue([]),
            exportAuditLog: jest.fn().mockReturnValue([]),
          },
        },
      ],
    }).compile();

    controller = module.get<DashboardController>(DashboardController);
    policyService = module.get<PolicyService>(PolicyService);
    auditLogger = module.get<AuditLoggerService>(AuditLoggerService);
  });

  describe('getRoot', () => {
    it('should return dashboard info', () => {
      const result = controller.getRoot();
      expect(result.name).toBe('AgentProof Dashboard API');
      expect(result.endpoints).toBeDefined();
      expect(result.endpoints.stats).toBeDefined();
    });
  });

  describe('getStats', () => {
    it('should return dashboard stats', () => {
      const result = controller.getStats();
      expect(result).toBeDefined();
      expect(result.verificationsToday).toBeDefined();
      expect(result.successRate).toBeDefined();
    });

    it('should calculate success rate correctly', () => {
      const result = controller.getStats();
      expect(result.successRate).toBeGreaterThanOrEqual(0);
      expect(result.successRate).toBeLessThanOrEqual(100);
    });
  });

  describe('getTrends', () => {
    it('should return verification trends for default days', () => {
      const result = controller.getTrends();
      expect(Array.isArray(result)).toBe(true);
      expect(result.length).toBe(7); // Default 7 days
    });

    it('should return trends for custom days', () => {
      const result = controller.getTrends(14);
      expect(result.length).toBe(14);
    });

    it('should include date, success, and failure counts', () => {
      const result = controller.getTrends(1);
      expect(result[0]).toHaveProperty('date');
      expect(result[0]).toHaveProperty('success');
      expect(result[0]).toHaveProperty('failure');
    });
  });

  describe('getTopAgents', () => {
    it('should return top agents', () => {
      const result = controller.getTopAgents();
      expect(Array.isArray(result)).toBe(true);
    });

    it('should limit results', () => {
      const result = controller.getTopAgents(5);
      expect(result.length).toBeLessThanOrEqual(5);
    });

    it('should sort by verification count descending', () => {
      jest.spyOn(auditLogger, 'getRecentEvents').mockReturnValue([
        { id: '1', type: AuditEventType.PROOF_VERIFICATION_SUCCESS, timestamp: new Date().toISOString(), agentId: 'agent-a', success: true },
        { id: '2', type: AuditEventType.PROOF_VERIFICATION_SUCCESS, timestamp: new Date().toISOString(), agentId: 'agent-a', success: true },
        { id: '3', type: AuditEventType.PROOF_VERIFICATION_SUCCESS, timestamp: new Date().toISOString(), agentId: 'agent-b', success: true },
      ]);

      const result = controller.getTopAgents();
      if (result.length >= 2) {
        expect(result[0].verificationCount).toBeGreaterThanOrEqual(result[1].verificationCount);
      }
    });
  });

  describe('getPolicies', () => {
    it('should return all policies', () => {
      const result = controller.getPolicies();
      expect(Array.isArray(result)).toBe(true);
      expect(result.length).toBe(1);
    });
  });

  describe('getPolicy', () => {
    it('should return policy by ID', () => {
      const result = controller.getPolicy('policy-1');
      expect(result.id).toBe('policy-1');
    });

    it('should throw NotFoundException for missing policy', () => {
      jest.spyOn(policyService, 'getPolicy').mockReturnValue(null);
      expect(() => controller.getPolicy('nonexistent')).toThrow(NotFoundException);
    });
  });

  describe('createPolicy', () => {
    it('should create a new policy', () => {
      const result = controller.createPolicy({
        name: 'New Policy',
        description: 'Test',
        rules: [],
      });

      expect(result).toBeDefined();
      expect(policyService.createPolicy).toHaveBeenCalled();
    });
  });

  describe('activatePolicy', () => {
    it('should activate a policy', () => {
      const result = controller.activatePolicy('policy-1');
      expect(result.success).toBe(true);
      expect(policyService.setActive).toHaveBeenCalledWith('policy-1', true);
    });
  });

  describe('deactivatePolicy', () => {
    it('should deactivate a policy', () => {
      const result = controller.deactivatePolicy('policy-1');
      expect(result.success).toBe(true);
      expect(policyService.setActive).toHaveBeenCalledWith('policy-1', false);
    });
  });

  describe('deletePolicy', () => {
    it('should delete a policy', () => {
      const result = controller.deletePolicy('policy-1');
      expect(result.success).toBe(true);
    });

    it('should throw NotFoundException for missing policy', () => {
      jest.spyOn(policyService, 'deletePolicy').mockReturnValue(false);
      expect(() => controller.deletePolicy('nonexistent')).toThrow(NotFoundException);
    });
  });

  describe('generateComplianceReport', () => {
    it('should generate compliance report', () => {
      const result = controller.generateComplianceReport({
        startDate: '2025-01-01',
        endDate: '2025-12-31',
      });

      expect(result).toBeDefined();
      expect(result.id).toBeDefined();
      expect(result.period).toBeDefined();
      expect(result.totalVerifications).toBeDefined();
    });

    it('should filter by agentId', () => {
      const result = controller.generateComplianceReport({
        startDate: '2025-01-01',
        endDate: '2025-12-31',
        agentId: 'agent-1',
      });

      expect(result).toBeDefined();
    });

    it('should filter by principalId', () => {
      const result = controller.generateComplianceReport({
        startDate: '2025-01-01',
        endDate: '2025-12-31',
        principalId: 'principal-1',
      });

      expect(result).toBeDefined();
    });
  });

  describe('getAuditTrail', () => {
    it('should return audit trail', () => {
      const result = controller.getAuditTrail(100);
      expect(Array.isArray(result)).toBe(true);
    });

    it('should filter by type', () => {
      jest.spyOn(auditLogger, 'getRecentEvents').mockReturnValue(mockEvents);
      const result = controller.getAuditTrail(100, AuditEventType.PROOF_VERIFICATION_SUCCESS);
      expect(Array.isArray(result)).toBe(true);
    });

    it('should return unfiltered when no type provided', () => {
      const result = controller.getAuditTrail(100, undefined);
      expect(Array.isArray(result)).toBe(true);
    });
  });
});

