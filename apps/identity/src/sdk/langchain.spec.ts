/**
 * LangChain Integration Tests
 */

import {
  AgentKern IdentityCallbackHandler,
  wrapTool,
  AgentKern IdentityFetch,
  createAgentKern IdentityFetch,
  LangChainAgentConfig,
} from './langchain';
import { AgentKern IdentityClient } from './index';

// Mock fetch
const mockFetch = jest.fn();
globalThis.fetch = mockFetch;

describe('LangChain Integration', () => {
  const mockConfig: LangChainAgentConfig = {
    principal: { id: 'principal-1', credentialId: 'cred-1' },
    agent: { id: 'langchain-agent', name: 'Test Agent', version: '1.0.0' },
    constraints: { maxAmount: 10000 },
    expiresInSeconds: 600,
  };

  beforeEach(() => {
    mockFetch.mockClear();
    // Mock successful proof creation
    mockFetch.mockResolvedValue({
      ok: true,
      json: () => Promise.resolve({
        header: 'AgentKernIdentity v1.test-header',
        proofId: 'proof-123',
        expiresAt: '2025-12-31T23:59:59Z',
      }),
    });
  });

  describe('AgentKern IdentityCallbackHandler', () => {
    it('should create handler with config', () => {
      const handler = new AgentKern IdentityCallbackHandler(mockConfig);
      expect(handler).toBeDefined();
    });

    it('should create handler with custom client', () => {
      const customClient = new AgentKern IdentityClient({ serverUrl: 'http://custom:8080' });
      const handler = new AgentKern IdentityCallbackHandler({
        ...mockConfig,
        client: customClient,
      });
      expect(handler).toBeDefined();
    });

    describe('onToolStart', () => {
      it('should create proof for tool call', async () => {
        const handler = new AgentKern IdentityCallbackHandler(mockConfig);
        const context = {
          toolName: 'search',
          toolInput: { query: 'test query' },
        };

        const proofHeader = await handler.onToolStart(context);

        expect(proofHeader).toBe('AgentKernIdentity v1.test-header');
        expect(mockFetch).toHaveBeenCalled();
      });

      it('should use provided target service and endpoint', async () => {
        const handler = new AgentKern IdentityCallbackHandler(mockConfig);
        const context = {
          toolName: 'api_call',
          toolInput: { data: 'test' },
          targetService: 'https://api.example.com',
          targetEndpoint: '/v1/search',
        };

        await handler.onToolStart(context);

        const [, options] = mockFetch.mock.calls[0];
        const body = JSON.parse(options.body);
        expect(body.intent.target.service).toBe('https://api.example.com');
        expect(body.intent.target.endpoint).toBe('/v1/search');
      });

      it('should return null on error', async () => {
        mockFetch.mockRejectedValue(new Error('Network error'));

        const handler = new AgentKern IdentityCallbackHandler(mockConfig);
        const result = await handler.onToolStart({
          toolName: 'test',
          toolInput: {},
        });

        expect(result).toBeNull();
      });
    });

    describe('onToolEnd', () => {
      it('should log success', async () => {
        const handler = new AgentKern IdentityCallbackHandler(mockConfig);
        
        // First start a tool to set lastProof
        await handler.onToolStart({ toolName: 'test', toolInput: {} });
        
        // Then end it
        await expect(
          handler.onToolEnd({ toolName: 'test', toolInput: {} }, true)
        ).resolves.not.toThrow();
      });

      it('should log failure', async () => {
        const handler = new AgentKern IdentityCallbackHandler(mockConfig);
        
        await handler.onToolStart({ toolName: 'test', toolInput: {} });
        
        await expect(
          handler.onToolEnd({ toolName: 'test', toolInput: {} }, false)
        ).resolves.not.toThrow();
      });
    });

    describe('getLastProof', () => {
      it('should return null initially', () => {
        const handler = new AgentKern IdentityCallbackHandler(mockConfig);
        expect(handler.getLastProof()).toBeNull();
      });

      it('should return last proof after tool start', async () => {
        const handler = new AgentKern IdentityCallbackHandler(mockConfig);
        await handler.onToolStart({ toolName: 'test', toolInput: {} });

        const proof = handler.getLastProof();
        expect(proof).not.toBeNull();
        expect(proof!.proofId).toBe('proof-123');
      });
    });
  });

  describe('wrapTool', () => {
    it('should wrap a tool function', async () => {
      const originalTool = jest.fn().mockResolvedValue({ result: 'success' });
      const wrappedTool = wrapTool(originalTool, 'testTool', mockConfig);

      const result = await wrappedTool({ input: 'test' });

      expect(originalTool).toHaveBeenCalledWith({ input: 'test' });
      expect(result.result).toBe('success');
      expect(result.__agentProof).toBe('AgentKernIdentity v1.test-header');
    });

    it('should handle tool with primitive return', async () => {
      const originalTool = jest.fn().mockResolvedValue('string result');
      const wrappedTool = wrapTool(originalTool, 'stringTool', mockConfig);

      const result = await wrappedTool({});

      expect(result).toBe('string result');
    });

    it('should propagate tool errors', async () => {
      const originalTool = jest.fn().mockRejectedValue(new Error('Tool error'));
      const wrappedTool = wrapTool(originalTool, 'errorTool', mockConfig);

      await expect(wrappedTool({})).rejects.toThrow('Tool error');
    });
  });

  describe('AgentKern IdentityFetch', () => {
    it('should create fetch wrapper', () => {
      const wrapper = new AgentKern IdentityFetch(mockConfig);
      expect(wrapper).toBeDefined();
    });

    it('should add X-AgentKern Identity header to requests', async () => {
      mockFetch.mockResolvedValue({
        ok: true,
        json: () => Promise.resolve({
          header: 'AgentKernIdentity v1.fetch-test',
          proofId: 'fetch-proof',
          expiresAt: '2025-12-31',
        }),
      });

      const wrapper = new AgentKern IdentityFetch(mockConfig);
      await wrapper.fetch('https://api.example.com/endpoint', {
        method: 'POST',
        body: JSON.stringify({ data: 'test' }),
      });

      // Second call should be the actual fetch with header
      expect(mockFetch).toHaveBeenCalledTimes(2);
      const [, fetchOptions] = mockFetch.mock.calls[1];
      expect(fetchOptions.headers['X-AgentKern Identity']).toBe('AgentKernIdentity v1.fetch-test');
    });

    it('should parse URL for intent', async () => {
      mockFetch.mockResolvedValue({
        ok: true,
        json: () => Promise.resolve({
          header: 'test',
          proofId: 'p-1',
          expiresAt: '2025-12-31',
        }),
      });

      const wrapper = new AgentKern IdentityFetch(mockConfig);
      await wrapper.fetch('https://api.bank.com/v1/transfer?amount=100');

      const [, createOptions] = mockFetch.mock.calls[0];
      const body = JSON.parse(createOptions.body);
      expect(body.intent.target.service).toBe('https://api.bank.com');
      expect(body.intent.target.endpoint).toBe('/v1/transfer');
    });
  });

  describe('createAgentKern IdentityFetch', () => {
    it('should return a bound fetch function', async () => {
      mockFetch.mockResolvedValue({
        ok: true,
        json: () => Promise.resolve({
          header: 'test',
          proofId: 'p-1',
          expiresAt: '2025-12-31',
        }),
      });

      const agentFetch = createAgentKern IdentityFetch(mockConfig);
      expect(typeof agentFetch).toBe('function');

      await agentFetch('https://example.com/test');
      expect(mockFetch).toHaveBeenCalled();
    });
  });
});
