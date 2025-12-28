/**
 * AgentKernIdentity SDK Tests
 * Using mocked fetch
 */

import {
  AgentKernIdentityClient,
  createAgentKernIdentityClient,
  agentProofMiddleware,
  RequireAgentKernIdentity,
  AgentKernIdentity,
} from './index';

// Mock global fetch
const mockFetch = jest.fn();
globalThis.fetch = mockFetch;

describe('AgentKernIdentity SDK', () => {
  let client: AgentKernIdentityClient;

  beforeEach(() => {
    client = new AgentKernIdentityClient({ serverUrl: 'http://test-server:3000' });
    mockFetch.mockClear();
  });

  describe('AgentKernIdentityClient', () => {
    describe('constructor', () => {
      it('should create client with default config', () => {
        const defaultClient = new AgentKernIdentityClient();
        expect(defaultClient).toBeDefined();
      });

      it('should create client with custom config', () => {
        const customClient = new AgentKernIdentityClient({
          serverUrl: 'http://custom:8080',
          timeout: 10000,
          retries: 5,
        });
        expect(customClient).toBeDefined();
      });
    });

    describe('createProof', () => {
      it('should create a proof', async () => {
        const mockResponse = { header: 'AgentKernIdentity v1.xxx', proofId: 'proof-1', expiresAt: '2025-12-31' };
        mockFetch.mockResolvedValue({
          ok: true,
          json: () => Promise.resolve(mockResponse),
        });

        const result = await client.createProof({
          principal: { id: 'principal-1', credentialId: 'cred-1' },
          agent: { id: 'agent-1', name: 'Test Agent', version: '1.0.0' },
          intent: {
            action: 'test',
            target: { service: 'api', endpoint: '/test', method: 'GET' },
          },
        });

        expect(result.proofId).toBe('proof-1');
        expect(mockFetch).toHaveBeenCalledWith(
          'http://test-server:3000/api/v1/proof/create',
          expect.objectContaining({ method: 'POST' }),
        );
      });

      it('should include expiresInSeconds in request', async () => {
        mockFetch.mockResolvedValue({
          ok: true,
          json: () => Promise.resolve({ header: 'test', proofId: 'p-1', expiresAt: '2025-12-31' }),
        });

        await client.createProof({
          principal: { id: 'p-1', credentialId: 'c-1' },
          agent: { id: 'a-1', name: 'Agent', version: '1.0' },
          intent: { action: 'test', target: { service: 's', endpoint: '/e', method: 'POST' } },
          expiresInSeconds: 600,
        });

        const [, options] = mockFetch.mock.calls[0];
        expect(JSON.parse(options.body).expiresInSeconds).toBe(600);
      });
    });

    describe('verifyProof', () => {
      it('should verify a proof', async () => {
        const mockResponse = { valid: true, proofId: 'proof-1', principalId: 'p-1' };
        mockFetch.mockResolvedValue({
          ok: true,
          json: () => Promise.resolve(mockResponse),
        });

        const result = await client.verifyProof('AgentKernIdentity v1.xxx.yyy');

        expect(result.valid).toBe(true);
        expect(mockFetch).toHaveBeenCalledWith(
          'http://test-server:3000/api/v1/proof/verify',
          expect.objectContaining({ method: 'POST' }),
        );
      });
    });

    describe('resolveTrust', () => {
      it('should resolve trust for agent-principal pair', async () => {
        const mockResponse = { trusted: true, trustScore: 750, ttl: 300, revoked: false };
        mockFetch.mockResolvedValue({
          ok: true,
          json: () => Promise.resolve(mockResponse),
        });

        const result = await client.resolveTrust('agent-1', 'principal-1');

        expect(result.trusted).toBe(true);
        expect(result.trustScore).toBe(750);
      });
    });

    describe('registerTrust', () => {
      it('should register trust relationship', async () => {
        mockFetch.mockResolvedValue({
          ok: true,
          json: () => Promise.resolve({}),
        });

        await client.registerTrust('agent-1', 'principal-1');

        expect(mockFetch).toHaveBeenCalledWith(
          'http://test-server:3000/api/v1/dns/register',
          expect.objectContaining({ method: 'POST' }),
        );
      });

      it('should include metadata if provided', async () => {
        mockFetch.mockResolvedValue({
          ok: true,
          json: () => Promise.resolve({}),
        });

        await client.registerTrust('agent-1', 'principal-1', {
          agentName: 'Test Agent',
          agentVersion: '2.0.0',
        });

        const [, options] = mockFetch.mock.calls[0];
        const body = JSON.parse(options.body);
        expect(body.agentName).toBe('Test Agent');
        expect(body.agentVersion).toBe('2.0.0');
      });
    });

    describe('revokeTrust', () => {
      it('should revoke trust', async () => {
        mockFetch.mockResolvedValue({
          ok: true,
          json: () => Promise.resolve({}),
        });

        await client.revokeTrust('agent-1', 'principal-1', 'Compromised');

        expect(mockFetch).toHaveBeenCalledWith(
          'http://test-server:3000/api/v1/dns/revoke',
          expect.objectContaining({ method: 'POST' }),
        );
      });
    });

    describe('getMeshStatus', () => {
      it('should get mesh status', async () => {
        const mockResponse = { nodeId: 'node-1', connectedPeers: 5, uptime: 3600 };
        mockFetch.mockResolvedValue({
          ok: true,
          json: () => Promise.resolve(mockResponse),
        });

        const result = await client.getMeshStatus();

        expect(result.nodeId).toBe('node-1');
        expect(result.connectedPeers).toBe(5);
      });
    });

    describe('error handling', () => {
      it('should throw on HTTP error', async () => {
        mockFetch.mockResolvedValue({
          ok: false,
          status: 400,
          json: () => Promise.resolve({ message: 'Bad Request' }),
        });

        await expect(client.verifyProof('invalid')).rejects.toThrow('Bad Request');
      });

      it('should handle JSON parse error', async () => {
        mockFetch.mockResolvedValue({
          ok: false,
          status: 500,
          json: () => Promise.reject(new Error('JSON parse error')),
        });

        await expect(client.verifyProof('test')).rejects.toThrow('Request failed');
      });
    });
  });

  describe('createAgentKernIdentityClient', () => {
    it('should create a new client instance', () => {
      const newClient = createAgentKernIdentityClient({ serverUrl: 'http://other:4000' });
      expect(newClient).toBeInstanceOf(AgentKernIdentityClient);
    });
  });

  describe('AgentKernIdentity singleton', () => {
    it('should be a default client instance', () => {
      expect(AgentKernIdentity).toBeInstanceOf(AgentKernIdentityClient);
    });
  });

  describe('agentProofMiddleware', () => {
    it('should return middleware function', () => {
      const middleware = agentProofMiddleware();
      expect(typeof middleware).toBe('function');
    });

    it('should reject requests without header when required', async () => {
      const middleware = agentProofMiddleware({ required: true });
      const req = { headers: {} };
      const res = { status: jest.fn().mockReturnThis(), json: jest.fn() };
      const next = jest.fn();

      await middleware(req, res, next);

      expect(res.status).toHaveBeenCalledWith(401);
      expect(next).not.toHaveBeenCalled();
    });

    it('should allow requests without header when not required', async () => {
      const middleware = agentProofMiddleware({ required: false });
      const req = { headers: {} };
      const res = { status: jest.fn().mockReturnThis(), json: jest.fn() };
      const next = jest.fn();

      await middleware(req, res, next);

      expect(next).toHaveBeenCalled();
    });

    it('should verify header and attach proof info', async () => {
      mockFetch.mockResolvedValue({
        ok: true,
        json: () => Promise.resolve({
          valid: true,
          proofId: 'proof-1',
          principalId: 'p-1',
          agentId: 'a-1',
        }),
      });

      const middleware = agentProofMiddleware();
      const req = { headers: { 'x-agentkern-identity': 'AgentKernIdentity v1.xxx' }, agentProof: null };
      const res = { status: jest.fn().mockReturnThis(), json: jest.fn() };
      const next = jest.fn();

      await middleware(req, res, next);

      expect(next).toHaveBeenCalled();
      expect(req.agentProof).toBeDefined();
      expect(req.agentProof.proofId).toBe('proof-1');
    });

    it('should reject invalid proofs', async () => {
      mockFetch.mockResolvedValue({
        ok: true,
        json: () => Promise.resolve({ valid: false, errors: ['Expired'] }),
      });

      const middleware = agentProofMiddleware();
      const req = { headers: { 'x-agentkern-identity': 'expired-proof' } };
      const res = { status: jest.fn().mockReturnThis(), json: jest.fn() };
      const next = jest.fn();

      await middleware(req, res, next);

      expect(res.status).toHaveBeenCalledWith(403);
      expect(next).not.toHaveBeenCalled();
    });

    it('should handle verification errors', async () => {
      mockFetch.mockRejectedValue(new Error('Network error'));

      const middleware = agentProofMiddleware();
      const req = { headers: { 'x-agentkern-identity': 'test-proof' } };
      const res = { status: jest.fn().mockReturnThis(), json: jest.fn() };
      const next = jest.fn();

      await middleware(req, res, next);

      expect(res.status).toHaveBeenCalledWith(500);
    });
  });

  describe('RequireAgentKernIdentity decorator', () => {
    it('should throw if header is missing', async () => {
      class TestController {
        @RequireAgentKernIdentity()
        async testMethod(request: any) {
          return 'success';
        }
      }

      const controller = new TestController();
      const request = { headers: {} };

      await expect(controller.testMethod(request)).rejects.toThrow('Missing X-AgentKernIdentity header');
    });

    it('should throw if proof is invalid', async () => {
      mockFetch.mockResolvedValue({
        ok: true,
        json: () => Promise.resolve({ valid: false, errors: ['Invalid'] }),
      });

      class TestController {
        @RequireAgentKernIdentity()
        async testMethod(request: any) {
          return 'success';
        }
      }

      const controller = new TestController();
      const request = { headers: { 'x-agentkern-identity': 'invalid' } };

      await expect(controller.testMethod(request)).rejects.toThrow('Invalid proof');
    });

    it('should call original method on valid proof', async () => {
      mockFetch.mockResolvedValue({
        ok: true,
        json: () => Promise.resolve({ valid: true }),
      });

      class TestController {
        @RequireAgentKernIdentity()
        async testMethod(request: any) {
          return 'success';
        }
      }

      const controller = new TestController();
      const request = { headers: { 'x-agentkern-identity': 'valid-proof' } };

      const result = await controller.testMethod(request);
      expect(result).toBe('success');
    });
  });
});
