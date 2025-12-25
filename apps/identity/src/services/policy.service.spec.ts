/**
 * Policy Service Tests - Full Coverage
 */

import { Test, TestingModule } from '@nestjs/testing';
import { PolicyService, Policy } from './policy.service';
import { PolicyAction } from '../dto/dashboard.dto';

describe('PolicyService', () => {
  let service: PolicyService;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [PolicyService],
    }).compile();

    service = module.get<PolicyService>(PolicyService);
  });

  describe('initialization', () => {
    it('should create default policies on init', () => {
      const policies = service.getAllPolicies();
      expect(policies.length).toBeGreaterThanOrEqual(2);
    });

    it('should have High-Value Transactions policy', () => {
      const policies = service.getAllPolicies();
      const hvt = policies.find(p => p.name === 'High-Value Transactions');
      expect(hvt).toBeDefined();
      expect(hvt?.active).toBe(true);
    });

    it('should have Destructive Actions policy', () => {
      const policies = service.getAllPolicies();
      const da = policies.find(p => p.name === 'Destructive Actions');
      expect(da).toBeDefined();
    });
  });

  describe('createPolicy', () => {
    it('should create a new policy', () => {
      const policy = service.createPolicy(
        'Test Policy',
        'A test policy',
        [{ name: 'Test Rule', condition: 'true', action: PolicyAction.ALLOW }],
      );
      
      expect(policy.id).toBeDefined();
      expect(policy.name).toBe('Test Policy');
      expect(policy.active).toBe(true);
    });

    it('should include policy in getAllPolicies', () => {
      service.createPolicy('New Policy', 'Description', []);
      const policies = service.getAllPolicies();
      expect(policies.find(p => p.name === 'New Policy')).toBeDefined();
    });

    it('should create policy with target agents', () => {
      const policy = service.createPolicy(
        'Agent Policy',
        'Targets specific agents',
        [],
        ['agent-1', 'agent-2'],
      );
      expect(policy.targetAgents).toContain('agent-1');
    });

    it('should create policy with target principals', () => {
      const policy = service.createPolicy(
        'Principal Policy',
        'Targets specific principals',
        [],
        [],
        ['principal-1'],
      );
      expect(policy.targetPrincipals).toContain('principal-1');
    });
  });

  describe('getPolicy', () => {
    it('should return policy by ID', () => {
      const created = service.createPolicy('Find Me', 'desc', []);
      const found = service.getPolicy(created.id);
      expect(found?.name).toBe('Find Me');
    });

    it('should return null for non-existent ID', () => {
      expect(service.getPolicy('non-existent')).toBeNull();
    });
  });

  describe('updatePolicy', () => {
    it('should update policy properties', () => {
      const policy = service.createPolicy('Original', 'desc', []);
      const updated = service.updatePolicy(policy.id, { name: 'Updated' });
      expect(updated?.name).toBe('Updated');
    });

    it('should update policy description', () => {
      const policy = service.createPolicy('Test', 'old desc', []);
      const updated = service.updatePolicy(policy.id, { description: 'new desc' });
      expect(updated?.description).toBe('new desc');
    });

    it('should return null for non-existent policy', () => {
      expect(service.updatePolicy('fake-id', { name: 'Test' })).toBeNull();
    });
  });

  describe('deletePolicy', () => {
    it('should delete existing policy', () => {
      const policy = service.createPolicy('ToDelete', 'desc', []);
      expect(service.deletePolicy(policy.id)).toBe(true);
      expect(service.getPolicy(policy.id)).toBeNull();
    });

    it('should return false for non-existent policy', () => {
      expect(service.deletePolicy('non-existent')).toBe(false);
    });
  });

  describe('setActive', () => {
    it('should activate/deactivate policy', () => {
      const policy = service.createPolicy('Toggle', 'desc', []);
      
      service.setActive(policy.id, false);
      expect(service.getPolicy(policy.id)?.active).toBe(false);
      
      service.setActive(policy.id, true);
      expect(service.getPolicy(policy.id)?.active).toBe(true);
    });
  });

  describe('evaluatePolicies', () => {
    it('should allow by default if no policy matches', () => {
      const result = service.evaluatePolicies({
        agentId: 'agent-1',
        principalId: 'principal-1',
        action: 'unknown-action',
        target: { service: 'test', endpoint: '/test', method: 'GET' },
      });
      expect(result.allowed).toBe(true);
    });

    it('should evaluate amount > condition', () => {
      service.createPolicy(
        'Deny High Amount',
        'Deny amounts over 5000',
        [{ name: 'Deny', condition: 'amount > 5000', action: PolicyAction.DENY }],
      );

      const result = service.evaluatePolicies({
        agentId: 'agent-1',
        principalId: 'principal-1',
        action: 'transfer',
        target: { service: 'bank', endpoint: '/transfer', method: 'POST' },
        amount: 10000,
      });
      
      expect(result.allowed).toBe(false);
      expect(result.action).toBe(PolicyAction.DENY);
    });

    it('should evaluate amount < condition', () => {
      service.createPolicy(
        'Deny Small Amount',
        'Deny amounts under 100',
        [{ name: 'Deny', condition: 'amount < 100', action: PolicyAction.DENY }],
      );

      const result = service.evaluatePolicies({
        agentId: 'agent-1',
        principalId: 'principal-1',
        action: 'transfer',
        target: { service: 'bank', endpoint: '/transfer', method: 'POST' },
        amount: 50,
      });
      
      expect(result.allowed).toBe(false);
    });

    it('should evaluate amount >= condition', () => {
      service.createPolicy(
        'Confirm Equal or Above',
        'Confirm amounts >= 1000',
        [{ name: 'Confirm', condition: 'amount >= 1000', action: PolicyAction.REQUIRE_CONFIRMATION }],
      );

      const result = service.evaluatePolicies({
        agentId: 'agent-1',
        principalId: 'principal-1',
        action: 'transfer',
        target: { service: 'bank', endpoint: '/transfer', method: 'POST' },
        amount: 1000,
      });
      
      expect(result.action).toBe(PolicyAction.REQUIRE_CONFIRMATION);
    });

    it('should evaluate amount <= condition', () => {
      service.createPolicy(
        'Allow Small',
        'Allow amounts <= 50',
        [{ name: 'Allow', condition: 'amount <= 50', action: PolicyAction.ALLOW }],
      );

      const result = service.evaluatePolicies({
        agentId: 'agent-1',
        principalId: 'principal-1',
        action: 'transfer',
        target: { service: 'bank', endpoint: '/transfer', method: 'POST' },
        amount: 50,
      });
      
      expect(result.allowed).toBe(true);
    });

    it('should evaluate amount == condition', () => {
      service.createPolicy(
        'Exact Amount',
        'Match exact amount',
        [{ name: 'Exact', condition: 'amount == 100', action: PolicyAction.ALLOW }],
      );

      const result = service.evaluatePolicies({
        agentId: 'agent-1',
        principalId: 'principal-1',
        action: 'transfer',
        target: { service: 'bank', endpoint: '/transfer', method: 'POST' },
        amount: 100,
      });
      
      expect(result.matchedRule).toBe('Exact');
    });

    it('should evaluate method conditions', () => {
      const result = service.evaluatePolicies({
        agentId: 'agent-1',
        principalId: 'principal-1',
        action: 'delete',
        target: { service: 'api', endpoint: '/resource', method: 'DELETE' },
      });
      
      expect(result.action).toBe(PolicyAction.RATE_LIMIT);
    });

    it('should evaluate action conditions', () => {
      service.createPolicy(
        'Action Match',
        'Match specific action',
        [{ name: 'Action', condition: "action == 'transfer'", action: PolicyAction.ALLOW }],
      );

      const result = service.evaluatePolicies({
        agentId: 'agent-1',
        principalId: 'principal-1',
        action: 'transfer',
        target: { service: 'bank', endpoint: '/transfer', method: 'POST' },
      });
      
      expect(result.matchedRule).toBe('Action');
    });

    it('should evaluate service conditions', () => {
      service.createPolicy(
        'Service Match',
        'Match specific service',
        [{ name: 'Service', condition: "service == 'banking-api'", action: PolicyAction.REQUIRE_CONFIRMATION }],
      );

      const result = service.evaluatePolicies({
        agentId: 'agent-1',
        principalId: 'principal-1',
        action: 'transfer',
        target: { service: 'banking-api', endpoint: '/transfer', method: 'POST' },
      });
      
      expect(result.action).toBe(PolicyAction.REQUIRE_CONFIRMATION);
    });

    it('should evaluate "true" condition', () => {
      service.createPolicy(
        'Always Allow',
        'Always allow',
        [{ name: 'Always', condition: 'true', action: PolicyAction.ALLOW }],
      );

      const result = service.evaluatePolicies({
        agentId: 'agent-1',
        principalId: 'principal-1',
        action: 'anything',
        target: { service: 'any', endpoint: '/any', method: 'GET' },
      });
      
      expect(result.matchedRule).toBe('Always');
    });

    it('should evaluate "false" condition as no match', () => {
      service.createPolicy(
        'Never Match',
        'Never matches',
        [{ name: 'Never', condition: 'false', action: PolicyAction.DENY }],
      );

      const result = service.evaluatePolicies({
        agentId: 'agent-1',
        principalId: 'principal-1',
        action: 'test',
        target: { service: 'test', endpoint: '/test', method: 'GET' },
      });
      
      // Should not match 'Never' rule
      expect(result.matchedRule).not.toBe('Never');
    });

    it('should handle invalid conditions gracefully', () => {
      service.createPolicy(
        'Invalid',
        'Invalid condition',
        [{ name: 'Bad', condition: 'this is not valid %%% syntax', action: PolicyAction.DENY }],
      );

      const result = service.evaluatePolicies({
        agentId: 'agent-1',
        principalId: 'principal-1',
        action: 'test',
        target: { service: 'test', endpoint: '/test', method: 'GET' },
      });
      
      // Should not crash, should not match
      expect(result.matchedRule).not.toBe('Bad');
    });
  });

  describe('policyApplies (via evaluatePolicies)', () => {
    it('should apply policy with matching target agent', () => {
      service.createPolicy(
        'Agent Specific',
        'Only for agent-1',
        [{ name: 'Deny', condition: 'true', action: PolicyAction.DENY }],
        ['agent-1'],
      );

      const result = service.evaluatePolicies({
        agentId: 'agent-1',
        principalId: 'principal-1',
        action: 'test',
        target: { service: 'test', endpoint: '/test', method: 'GET' },
      });
      
      expect(result.matchedPolicy).toBe('Agent Specific');
    });

    it('should not apply policy with non-matching agent', () => {
      service.createPolicy(
        'Wrong Agent',
        'Only for agent-X',
        [{ name: 'Deny', condition: 'true', action: PolicyAction.DENY }],
        ['agent-X'],
      );

      const result = service.evaluatePolicies({
        agentId: 'agent-1',
        principalId: 'principal-1',
        action: 'test',
        target: { service: 'test', endpoint: '/test', method: 'GET' },
      });
      
      expect(result.matchedPolicy).not.toBe('Wrong Agent');
    });

    it('should apply policy with matching principal', () => {
      service.createPolicy(
        'Principal Specific',
        'Only for principal-1',
        [{ name: 'Allow', condition: 'true', action: PolicyAction.ALLOW }],
        [],
        ['principal-1'],
      );

      const result = service.evaluatePolicies({
        agentId: 'agent-1',
        principalId: 'principal-1',
        action: 'test',
        target: { service: 'test', endpoint: '/test', method: 'GET' },
      });
      
      expect(result.matchedPolicy).toBe('Principal Specific');
    });

    it('should not apply policy with non-matching principal', () => {
      service.createPolicy(
        'Wrong Principal',
        'Only for principal-X',
        [{ name: 'Deny', condition: 'true', action: PolicyAction.DENY }],
        [],
        ['principal-X'],
      );

      const result = service.evaluatePolicies({
        agentId: 'agent-1',
        principalId: 'principal-1',
        action: 'test',
        target: { service: 'test', endpoint: '/test', method: 'GET' },
      });
      
      expect(result.matchedPolicy).not.toBe('Wrong Principal');
    });

    it('should skip inactive policies', () => {
      const policy = service.createPolicy(
        'Inactive',
        'This is inactive',
        [{ name: 'Deny', condition: 'true', action: PolicyAction.DENY }],
      );
      service.setActive(policy.id, false);

      const result = service.evaluatePolicies({
        agentId: 'agent-1',
        principalId: 'principal-1',
        action: 'test',
        target: { service: 'test', endpoint: '/test', method: 'GET' },
      });
      
      expect(result.matchedPolicy).not.toBe('Inactive');
    });
  });

  describe('getActionMessage (via evaluatePolicies)', () => {
    it('should return message for ALLOW', () => {
      service.createPolicy(
        'Allow Policy',
        'Always allow',
        [{ name: 'Allow Rule', condition: 'true', action: PolicyAction.ALLOW }],
      );

      const result = service.evaluatePolicies({
        agentId: 'agent-1',
        principalId: 'principal-1',
        action: 'test',
        target: { service: 'test', endpoint: '/test', method: 'GET' },
      });
      
      expect(result.message).toBe('Action allowed by policy');
    });

    it('should return message for DENY', () => {
      service.createPolicy(
        'Deny Policy',
        'Always deny',
        [{ name: 'Deny Rule', condition: 'true', action: PolicyAction.DENY }],
      );

      const result = service.evaluatePolicies({
        agentId: 'agent-1',
        principalId: 'principal-1',
        action: 'test',
        target: { service: 'test', endpoint: '/test', method: 'GET' },
      });
      
      expect(result.message).toContain('Action denied by rule');
    });

    it('should return message for REQUIRE_CONFIRMATION', () => {
      service.createPolicy(
        'Confirm Policy',
        'Require confirmation',
        [{ name: 'Confirm Rule', condition: 'true', action: PolicyAction.REQUIRE_CONFIRMATION }],
      );

      const result = service.evaluatePolicies({
        agentId: 'agent-1',
        principalId: 'principal-1',
        action: 'test',
        target: { service: 'test', endpoint: '/test', method: 'GET' },
      });
      
      expect(result.message).toBe('Action requires manual confirmation');
    });

    it('should return message for RATE_LIMIT with limit value', () => {
      service.createPolicy(
        'Rate Policy',
        'Rate limit',
        [{ name: 'Rate Rule', condition: 'true', action: PolicyAction.RATE_LIMIT, rateLimit: 5 }],
      );

      const result = service.evaluatePolicies({
        agentId: 'agent-1',
        principalId: 'principal-1',
        action: 'test',
        target: { service: 'test', endpoint: '/test', method: 'GET' },
      });
      
      expect(result.message).toContain('rate limited');
    });
  });

  describe('evaluateCondition (error handling)', () => {
    it('should handle invalid conditions gracefully (catch block)', () => {
      // Mock logger to verify warning
      const warnSpy = jest.spyOn((service as any).logger, 'warn').mockImplementation();
      
      // Force an error by passing a context object that throws on access
      const context = {
        agentId: 'agent-1',
        principalId: 'principal-1',
        action: 'test',
        target: { service: 'test', endpoint: '/test', method: 'GET' },
        get amount() { throw new Error('Force error'); }
      };

      service.createPolicy(
        'Error Policy',
        'Causes error',
        [{ name: 'Error Rule', condition: 'amount > 100', action: PolicyAction.ALLOW }],
      );

      const result = service.evaluatePolicies(context as any);
      
      expect(result.allowed).toBe(true); // Default allow
      expect(warnSpy).toHaveBeenCalled();
    });
  });

  describe('getActionMessage (edge cases)', () => {
      it('should return Unknown action for invalid enum value', () => {
          service.createPolicy(
            'Unknown Policy',
            'Has unknown action',
            [{ name: 'Unknown Rule', condition: 'true', action: 'INVALID_ACTION' as PolicyAction }],
          );

          const result = service.evaluatePolicies({
            agentId: 'agent-1',
            principalId: 'principal-1',
            action: 'test',
            target: { service: 'test', endpoint: '/test', method: 'GET' },
          });
          
          expect(result.message).toBe('Unknown action');
      });
  });
});

