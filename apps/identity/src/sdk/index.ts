/**
 * AgentKernIdentity SDK - TypeScript Client
 * 
 * Zero-config embedded verification for AI agents.
 * Install: npm install @agentkern/sdk
 * 
 * Usage:
 * ```typescript
 * import { AgentKernIdentity } from '@agentkern/sdk';
 * 
 * const proof = await AgentKernIdentity.createProof({
 *   principal: { id: 'user-123', credentialId: 'cred-456' },
 *   agent: { id: 'my-agent', name: 'My Agent', version: '1.0.0' },
 *   intent: { action: 'transfer', target: { service: 'api.bank.com', endpoint: '/transfer', method: 'POST' } }
 * });
 * 
 * // Add to request
 * const response = await fetch(url, {
 *   headers: { 'X-AgentKernIdentity': proof.header }
 * });
 * ```
 */

export interface Principal {
  id: string;
  credentialId: string;
  deviceAttestation?: string;
}

export interface Agent {
  id: string;
  name: string;
  version: string;
}

export interface IntentTarget {
  service: string;
  endpoint: string;
  method: 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH';
}

export interface Intent {
  action: string;
  target: IntentTarget;
  parameters?: Record<string, unknown>;
}

export interface Constraints {
  maxAmount?: number;
  allowedRecipients?: string[];
  geoFence?: string[];
  validHours?: { start: number; end: number };
  requireConfirmationAbove?: number;
  singleUse?: boolean;
}

export interface CreateProofOptions {
  principal: Principal;
  agent: Agent;
  intent: Intent;
  constraints?: Constraints;
  expiresInSeconds?: number;
}

export interface ProofResult {
  header: string;
  proofId: string;
  expiresAt: string;
}

export interface VerifyResult {
  valid: boolean;
  proofId?: string;
  principalId?: string;
  agentId?: string;
  intent?: { action: string; target: string };
  liabilityAcceptedBy?: string;
  errors?: string[];
}

export interface TrustResolution {
  trusted: boolean;
  trustScore: number;
  ttl: number;
  revoked: boolean;
}

export interface AgentKernIdentityConfig {
  serverUrl?: string;
  timeout?: number;
  retries?: number;
}

const DEFAULT_CONFIG: AgentKernIdentityConfig = {
  serverUrl: 'http://localhost:5002',
  timeout: 5000,
  retries: 3,
};

/**
 * AgentKernIdentity SDK Client
 */
export class AgentKernIdentityClient {
  private config: AgentKernIdentityConfig;

  constructor(config: Partial<AgentKernIdentityConfig> = {}) {
    this.config = { ...DEFAULT_CONFIG, ...config };
  }

  /**
   * Create a signed Liability Proof
   */
  async createProof(options: CreateProofOptions): Promise<ProofResult> {
    const response = await this.fetch('/api/v1/proof/create', {
      method: 'POST',
      body: JSON.stringify({
        principal: options.principal,
        agent: options.agent,
        intent: options.intent,
        constraints: options.constraints,
        expiresInSeconds: options.expiresInSeconds || 300,
      }),
    });

    return response as ProofResult;
  }

  /**
   * Verify a Liability Proof
   */
  async verifyProof(header: string): Promise<VerifyResult> {
    const response = await this.fetch('/api/v1/proof/verify', {
      method: 'POST',
      body: JSON.stringify({ proof: header }),
    });

    return response as VerifyResult;
  }

  /**
   * Resolve trust for an agent-principal pair
   */
  async resolveTrust(agentId: string, principalId: string): Promise<TrustResolution> {
    const response = await this.fetch(
      `/api/v1/dns/resolve?agentId=${encodeURIComponent(agentId)}&principalId=${encodeURIComponent(principalId)}`,
      { method: 'GET' },
    );

    return response as TrustResolution;
  }

  /**
   * Register a trust relationship
   */
  async registerTrust(agentId: string, principalId: string, metadata?: { agentName?: string; agentVersion?: string }): Promise<void> {
    await this.fetch('/api/v1/dns/register', {
      method: 'POST',
      body: JSON.stringify({
        agentId,
        principalId,
        ...metadata,
      }),
    });
  }

  /**
   * Revoke trust
   */
  async revokeTrust(agentId: string, principalId: string, reason: string): Promise<void> {
    await this.fetch('/api/v1/dns/revoke', {
      method: 'POST',
      body: JSON.stringify({ agentId, principalId, reason }),
    });
  }

  /**
   * Get mesh node status
   */
  async getMeshStatus(): Promise<{ nodeId: string; connectedPeers: number; uptime: number }> {
    const response = await this.fetch('/api/v1/mesh/stats', { method: 'GET' });
    return response as { nodeId: string; connectedPeers: number; uptime: number };
  }

  private async fetch(path: string, options: RequestInit): Promise<unknown> {
    const url = `${this.config.serverUrl}${path}`;
    
    const response = await globalThis.fetch(url, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...options.headers,
      },
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ message: 'Request failed' }));
      throw new Error(error.message || `HTTP ${response.status}`);
    }

    return response.json();
  }
}

/**
 * Default singleton instance
 */
export const AgentKernIdentity = new AgentKernIdentityClient();

/**
 * Create a new client with custom config
 */
export function createAgentKernIdentityClient(config: Partial<AgentKernIdentityConfig> = {}): AgentKernIdentityClient {
  return new AgentKernIdentityClient(config);
}

/**
 * Middleware for Express/NestJS to verify incoming requests
 * @deprecated Use agentKernIdentityMiddleware instead
 */
export function agentProofMiddleware(options: { required?: boolean; client?: AgentKernIdentityClient } = {}) {
  return agentKernIdentityMiddleware(options);
}

/**
 * Middleware for Express/NestJS to verify incoming requests (Dec 2025)
 */
export function agentKernIdentityMiddleware(options: { required?: boolean; client?: AgentKernIdentityClient } = {}) {
  const client = options.client || AgentKernIdentity;
  const required = options.required ?? true;

  return async (req: any, res: any, next: any) => {
    const header = req.headers['x-agentkern-identity'];

    if (!header) {
      if (required) {
        return res.status(401).json({ error: 'Missing X-AgentKernIdentity header' });
      }
      return next();
    }

    try {
      const result = await client.verifyProof(header);

      if (!result.valid) {
        return res.status(403).json({ error: 'Invalid proof', details: result.errors });
      }

      // Attach proof info to request (legacy: req.agentProof, new: req.agentKernIdentity)
      req.agentKernIdentity = {
        proofId: result.proofId,
        principalId: result.principalId,
        agentId: result.agentId,
        intent: result.intent,
        liabilityAcceptedBy: result.liabilityAcceptedBy,
      };
      // @deprecated - keep for backward compatibility
      req.agentProof = req.agentKernIdentity;

      next();
    } catch (error) {
      return res.status(500).json({ error: 'Proof verification failed' });
    }
  };
}

/**
 * Decorator for NestJS controllers
 */
export function RequireAgentKernIdentity() {
  return (target: any, propertyKey: string, descriptor: PropertyDescriptor) => {
    const originalMethod = descriptor.value;

    descriptor.value = async function (...args: any[]) {
      const request = args.find(arg => arg?.headers);
      
      if (!request?.headers?.['x-agentkern-identity']) {
        throw new Error('Missing X-AgentKernIdentity header');
      }

      const result = await AgentKernIdentity.verifyProof(request.headers['x-agentkern-identity']);
      
      if (!result.valid) {
        throw new Error(`Invalid proof: ${result.errors?.join(', ')}`);
      }

      return originalMethod.apply(this, args);
    };

    return descriptor;
  };
}
