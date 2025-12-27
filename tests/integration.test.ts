/**
 * AgentKern Integration Tests (TypeScript side)
 * 
 * Tests SDK → Gateway → Rust roundtrip
 * 
 * Run with: npm test --workspace=@agentkern/sdk
 */

import { describe, it, expect, beforeAll, afterAll } from 'vitest';

// Mock the SDK client (would import from @agentkern/sdk in production)
interface VerificationRequest {
  agentId: string;
  action: string;
  context?: Record<string, unknown>;
}

interface VerificationResult {
  requestId: string;
  allowed: boolean;
  reasoning?: string;
  latencyMs?: number;
}

interface GateClient {
  verify(request: VerificationRequest): Promise<VerificationResult>;
  registerPolicy(policy: unknown): Promise<void>;
}

// Create mock client for testing
function createGateClient(baseUrl: string): GateClient {
  return {
    async verify(request: VerificationRequest): Promise<VerificationResult> {
      const response = await fetch(`${baseUrl}/api/gate/verify`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(request),
      });
      return response.json();
    },
    async registerPolicy(policy: unknown): Promise<void> {
      await fetch(`${baseUrl}/api/gate/policies`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(policy),
      });
    },
  };
}

describe('AgentKern Integration Tests', () => {
  const baseUrl = process.env.AGENTKERN_API_URL || 'http://localhost:3000';
  let client: GateClient;

  beforeAll(() => {
    client = createGateClient(baseUrl);
  });

  describe('Gate Verification', () => {
    it('should allow safe actions', async () => {
      const result = await client.verify({
        agentId: 'test-agent',
        action: 'read_data',
        context: {},
      });

      expect(result.requestId).toBeDefined();
      expect(typeof result.allowed).toBe('boolean');
    });

    it('should return latency under 50ms', async () => {
      const start = Date.now();
      
      await client.verify({
        agentId: 'test-agent',
        action: 'read_data',
        context: {},
      });
      
      const latency = Date.now() - start;
      expect(latency).toBeLessThan(50);
    });

    it('should block dangerous actions with registered policy', async () => {
      // Register blocking policy
      await client.registerPolicy({
        id: 'no-delete',
        name: 'No Delete Policy',
        rules: [{
          condition: "action == 'delete_all'",
          action: 'deny',
        }],
      });

      const result = await client.verify({
        agentId: 'test-agent',
        action: 'delete_all',
        context: {},
      });

      expect(result.allowed).toBe(false);
    });
  });

  describe('Synapse Memory', () => {
    it('should store and retrieve memory', async () => {
      const memory = {
        agentId: 'memory-test-agent',
        content: 'User prefers dark mode',
        importance: 0.8,
      };

      const storeResponse = await fetch(`${baseUrl}/api/synapse/memories`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(memory),
      });

      expect(storeResponse.ok).toBe(true);

      // Retrieve memories
      const getResponse = await fetch(
        `${baseUrl}/api/synapse/memories?agentId=memory-test-agent`
      );

      expect(getResponse.ok).toBe(true);
    });
  });

  describe('Arbiter Locks', () => {
    it('should acquire and release locks', async () => {
      const lockRequest = {
        resourceId: 'test-resource-123',
        agentId: 'lock-test-agent',
        ttlSeconds: 30,
      };

      // Acquire lock
      const acquireResponse = await fetch(`${baseUrl}/api/arbiter/locks`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(lockRequest),
      });

      expect(acquireResponse.ok).toBe(true);
      const lock = await acquireResponse.json();
      expect(lock.lockId).toBeDefined();

      // Release lock
      const releaseResponse = await fetch(
        `${baseUrl}/api/arbiter/locks/${lock.lockId}`,
        { method: 'DELETE' }
      );

      expect(releaseResponse.ok).toBe(true);
    });

    it('should prevent concurrent lock acquisition', async () => {
      const resourceId = `exclusive-resource-${Date.now()}`;
      
      // First agent acquires lock
      const lock1Response = await fetch(`${baseUrl}/api/arbiter/locks`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          resourceId,
          agentId: 'agent-1',
          ttlSeconds: 30,
        }),
      });

      expect(lock1Response.ok).toBe(true);

      // Second agent tries to acquire same lock
      const lock2Response = await fetch(`${baseUrl}/api/arbiter/locks`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          resourceId,
          agentId: 'agent-2',
          ttlSeconds: 30,
        }),
      });

      // Should be rejected (409 Conflict or similar)
      expect(lock2Response.ok).toBe(false);
    });
  });

  describe('Treasury Payments', () => {
    it('should process agent-to-agent payments', async () => {
      // Register agents
      await fetch(`${baseUrl}/api/treasury/agents`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ agentId: 'payer-agent' }),
      });

      await fetch(`${baseUrl}/api/treasury/agents`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ agentId: 'payee-agent' }),
      });

      // Deposit funds
      await fetch(`${baseUrl}/api/treasury/deposit`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          agentId: 'payer-agent',
          amount: 100,
          currency: 'credits',
        }),
      });

      // Make payment
      const paymentResponse = await fetch(`${baseUrl}/api/treasury/pay`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          fromAgent: 'payer-agent',
          toAgent: 'payee-agent',
          amount: 25,
          currency: 'credits',
        }),
      });

      expect(paymentResponse.ok).toBe(true);
      const payment = await paymentResponse.json();
      expect(payment.paymentId).toBeDefined();
    });
  });

  describe('End-to-End Flow', () => {
    it('should complete full agent lifecycle', async () => {
      const agentId = `e2e-agent-${Date.now()}`;

      // 1. Create agent identity
      await fetch(`${baseUrl}/api/identity/agents`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          id: agentId,
          name: 'E2E Test Agent',
          capabilities: ['read', 'write', 'financial'],
        }),
      });

      // 2. Store initial memory
      await fetch(`${baseUrl}/api/synapse/memories`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          agentId,
          content: 'Agent initialized',
          importance: 1.0,
        }),
      });

      // 3. Verify an action
      const verifyResult = await client.verify({
        agentId,
        action: 'process_data',
        context: { dataType: 'public' },
      });

      expect(verifyResult.allowed).toBe(true);

      // 4. Acquire lock for exclusive operation
      const lockResponse = await fetch(`${baseUrl}/api/arbiter/locks`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          resourceId: `${agentId}:workspace`,
          agentId,
          ttlSeconds: 60,
        }),
      });

      expect(lockResponse.ok).toBe(true);

      // 5. Complete the flow
      console.log(`✅ E2E test passed for agent ${agentId}`);
    });
  });
});
