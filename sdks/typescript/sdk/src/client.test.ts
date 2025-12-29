/**
 * AgentKern SDK - Unit Tests
 * 
 * Per MANDATE: 100% Coverage (Zero Tolerance)
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { AgentKern } from './client';
import type { AgentKernConfig } from './types';

describe('AgentKern SDK', () => {
  describe('Client Initialization', () => {
    it('should create a client with default config', () => {
      const client = new AgentKern();
      
      expect(client).toBeDefined();
      expect(AgentKern.VERSION).toBe('0.1.0');
    });

    it('should accept custom configuration', () => {
      const config: AgentKernConfig = {
        endpoint: 'https://custom.api.agentkern.io',
        apiKey: 'test-key',
        region: 'eu',
        debug: true,
      };
      
      const client = new AgentKern(config);
      const publicConfig = client.getConfig();
      
      expect(publicConfig.endpoint).toBe('https://custom.api.agentkern.io');
      expect(publicConfig.region).toBe('eu');
      expect(publicConfig.debug).toBe(true);
    });

    it('should not expose API key in public config', () => {
      const client = new AgentKern({ apiKey: 'secret-key' });
      const publicConfig = client.getConfig();
      
      expect((publicConfig as any).apiKey).toBeUndefined();
    });

    it('should check region correctly', () => {
      const euClient = new AgentKern({ region: 'eu' });
      const usClient = new AgentKern({ region: 'us' });
      
      expect(euClient.isRegion('eu')).toBe(true);
      expect(euClient.isRegion('us')).toBe(false);
      expect(usClient.isRegion('us')).toBe(true);
    });
  });

  describe('Identity Module', () => {
    let client: AgentKern;

    beforeEach(() => {
      client = new AgentKern({ region: 'eu' });
    });

    it('should register a new agent', async () => {
      const agent = await client.identity.register('test-agent', ['read', 'write']);
      
      expect(agent).toBeDefined();
      expect(agent.name).toBe('test-agent');
      expect(agent.id).toBeDefined();
      expect(agent.jurisdiction).toBe('eu');
    });

    it('should return null for unknown agent', async () => {
      const agent = await client.identity.getIdentity('unknown-id');
      
      expect(agent).toBeNull();
    });

    it('should sign an action', async () => {
      const proof = await client.identity.signAction('agent-1', 'send_email', { to: 'test@example.com' });
      
      expect(proof).toBeDefined();
      expect(proof.action).toBe('send_email');
      expect(proof.agentId).toBe('agent-1');
      expect(proof.timestamp).toBeInstanceOf(Date);
    });

    it('should verify a proof', async () => {
      const proof = await client.identity.signAction('agent-1', 'action', {});
      const isValid = await client.identity.verifyProof(proof);
      
      expect(isValid).toBe(true);
    });

    it('should get trust score', async () => {
      const score = await client.identity.getTrustScore('agent-1');
      
      expect(score).toBeGreaterThanOrEqual(0);
      expect(score).toBeLessThanOrEqual(100);
    });
  });

  describe('Gate Module (Guardrails)', () => {
    let client: AgentKern;

    beforeEach(() => {
      client = new AgentKern();
    });

    it('should verify an action', async () => {
      const result = await client.gate.verify('agent-1', 'send_email', { to: 'user@example.com' });
      
      expect(result).toBeDefined();
      expect(result.allowed).toBe(true);
      expect(result.riskScore).toBeGreaterThanOrEqual(0);
      expect(result.latencyMs).toBeGreaterThanOrEqual(0);
    });

    it('should return evaluated policies', async () => {
      const result = await client.gate.verify('agent-1', 'send_email');
      
      expect(result.evaluatedPolicies).toBeDefined();
      expect(Array.isArray(result.evaluatedPolicies)).toBe(true);
    });

    it('should get policies', async () => {
      const policies = await client.gate.getPolicies();
      
      expect(Array.isArray(policies)).toBe(true);
    });
  });

  describe('Synapse Module (Memory)', () => {
    let client: AgentKern;

    beforeEach(() => {
      client = new AgentKern();
    });

    it('should start an intent path', async () => {
      const intent = await client.synapse.startIntent('agent-1', 'Process customer order', 5);
      
      expect(intent).toBeDefined();
      expect(intent.originalIntent).toBe('Process customer order');
      expect(intent.totalSteps).toBe(5);
      expect(intent.currentStep).toBe(0);
      expect(intent.driftDetected).toBe(false);
    });

    it('should record a step', async () => {
      const intent = await client.synapse.recordStep('agent-1', 'validate_order', 'success');
      
      expect(intent.history.length).toBeGreaterThan(0);
      expect(intent.history[0].action).toBe('validate_order');
    });

    it('should check for drift', async () => {
      const drift = await client.synapse.checkDrift('agent-1');
      
      expect(drift).toBeDefined();
      expect(typeof drift.drifted).toBe('boolean');
      expect(typeof drift.score).toBe('number');
    });

    it('should set and get state', async () => {
      const state = await client.synapse.setState('agent-1', { orderId: '123', status: 'pending' });
      
      expect(state.agentId).toBe('agent-1');
      expect(state.state.orderId).toBe('123');
      expect(state.version).toBeGreaterThanOrEqual(1);
    });
  });

  describe('Arbiter Module (Coordination)', () => {
    let client: AgentKern;

    beforeEach(() => {
      client = new AgentKern();
    });

    it('should acquire a lock', async () => {
      const lock = await client.arbiter.acquireLock('agent-1', 'customer:123', 10);
      
      expect(lock).toBeDefined();
      expect(lock?.resource).toBe('customer:123');
      expect(lock?.lockedBy).toBe('agent-1');
      expect(lock?.priority).toBe(10);
    });

    it('should release a lock', async () => {
      const released = await client.arbiter.releaseLock('agent-1', 'customer:123');
      
      expect(released).toBe(true);
    });

    it('should request coordination', async () => {
      const result = await client.arbiter.requestCoordination({
        agentId: 'agent-1',
        resource: 'database:accounts',
        operation: 'write',
        expectedDurationMs: 5000,
        priority: 5,
      });
      
      expect(result).toBeDefined();
      expect(result.granted).toBe(true);
      expect(result.lock).toBeDefined();
    });
  });

  describe('Sovereign Module (Data Residency)', () => {
    let client: AgentKern;

    beforeEach(() => {
      client = new AgentKern({ region: 'eu' });
    });

    it('should block transfer from China to outside China (PIPL)', async () => {
      const canTransfer = await client.sovereign.canTransfer('cn', 'us', 'personal_data');
      
      expect(canTransfer).toBe(false);
    });

    it('should allow transfer within China', async () => {
      const canTransfer = await client.sovereign.canTransfer('cn', 'cn', 'personal_data');
      
      expect(canTransfer).toBe(true);
    });

    it('should block EU transfer to non-adequate countries', async () => {
      const canTransfer = await client.sovereign.canTransfer('eu', 'cn', 'personal_data');
      
      expect(canTransfer).toBe(false);
    });

    it('should allow EU transfer to US (adequacy assumed)', async () => {
      const canTransfer = await client.sovereign.canTransfer('eu', 'us', 'personal_data');
      
      expect(canTransfer).toBe(true);
    });

    it('should validate compliance', async () => {
      const result = await client.sovereign.validateCompliance('store_data', { email: 'test@example.com' }, 'eu');
      
      expect(result.compliant).toBe(true);
    });
  });
});
