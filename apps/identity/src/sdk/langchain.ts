/**
 * AgentProof - LangChain Integration
 * 
 * Provides AgentProof verification for LangChain agents.
 * Zero-config: agents automatically include Liability Proofs.
 * 
 * Usage:
 * ```typescript
 * import { AgentProofLangChain } from '@agentproof/sdk/langchain';
 * 
 * const agent = AgentProofLangChain.wrap(myAgent, {
 *   principal: { id: 'user-123', credentialId: 'cred-456' },
 *   agent: { id: 'langchain-agent', name: 'My LangChain Agent', version: '1.0.0' }
 * });
 * ```
 */

import { AgentProofClient, Principal, Agent, Intent, Constraints, ProofResult } from './index';

export interface LangChainAgentConfig {
  principal: Principal;
  agent: Agent;
  constraints?: Constraints;
  expiresInSeconds?: number;
  client?: AgentProofClient;
}

export interface ToolCallContext {
  toolName: string;
  toolInput: Record<string, unknown>;
  targetService?: string;
  targetEndpoint?: string;
}

/**
 * LangChain callback handler that adds AgentProof to tool calls
 */
export class AgentProofCallbackHandler {
  private config: LangChainAgentConfig;
  private client: AgentProofClient;
  private lastProof: ProofResult | null = null;

  constructor(config: LangChainAgentConfig) {
    this.config = config;
    this.client = config.client || new AgentProofClient();
  }

  /**
   * Called before a tool is executed
   */
  async onToolStart(context: ToolCallContext): Promise<string | null> {
    try {
      const intent: Intent = {
        action: context.toolName,
        target: {
          service: context.targetService || 'langchain-tool',
          endpoint: context.targetEndpoint || `/${context.toolName}`,
          method: 'POST',
        },
        parameters: context.toolInput,
      };

      const proof = await this.client.createProof({
        principal: this.config.principal,
        agent: this.config.agent,
        intent,
        constraints: this.config.constraints,
        expiresInSeconds: this.config.expiresInSeconds,
      });

      this.lastProof = proof;
      return proof.header;
    } catch (error) {
      console.error('[AgentProof] Failed to create proof for tool:', context.toolName, error);
      return null;
    }
  }

  /**
   * Called after a tool execution completes
   */
  async onToolEnd(context: ToolCallContext, success: boolean): Promise<void> {
    // In production, report verification result to mesh
    if (this.lastProof) {
      console.debug(`[AgentProof] Tool ${context.toolName} completed:`, success ? 'success' : 'failure');
    }
  }

  /**
   * Get the last generated proof
   */
  getLastProof(): ProofResult | null {
    return this.lastProof;
  }
}

/**
 * Wrap a LangChain-style tool to automatically add AgentProof
 */
export function wrapTool<T extends (...args: any[]) => Promise<any>>(
  tool: T,
  toolName: string,
  config: LangChainAgentConfig,
): T {
  const handler = new AgentProofCallbackHandler(config);

  return (async (...args: any[]) => {
    const context: ToolCallContext = {
      toolName,
      toolInput: args[0] || {},
    };

    // Generate proof before execution
    const proofHeader = await handler.onToolStart(context);

    try {
      // Execute the original tool
      const result = await tool(...args);

      // Report success
      await handler.onToolEnd(context, true);

      // Attach proof to result if object
      if (typeof result === 'object' && result !== null) {
        return { ...result, __agentProof: proofHeader };
      }

      return result;
    } catch (error) {
      // Report failure
      await handler.onToolEnd(context, false);
      throw error;
    }
  }) as T;
}

/**
 * HTTP client wrapper that automatically adds X-AgentProof header
 */
export class AgentProofFetch {
  private config: LangChainAgentConfig;
  private client: AgentProofClient;

  constructor(config: LangChainAgentConfig) {
    this.config = config;
    this.client = config.client || new AgentProofClient();
  }

  /**
   * Fetch with automatic AgentProof header
   */
  async fetch(url: string, options: RequestInit = {}): Promise<Response> {
    const urlObj = new URL(url);
    
    const intent: Intent = {
      action: 'http_request',
      target: {
        service: urlObj.origin,
        endpoint: urlObj.pathname,
        method: (options.method || 'GET') as any,
      },
      parameters: options.body ? JSON.parse(options.body as string) : undefined,
    };

    const proof = await this.client.createProof({
      principal: this.config.principal,
      agent: this.config.agent,
      intent,
      constraints: this.config.constraints,
      expiresInSeconds: this.config.expiresInSeconds,
    });

    return globalThis.fetch(url, {
      ...options,
      headers: {
        ...options.headers,
        'X-AgentProof': proof.header,
      },
    });
  }
}

/**
 * Create an AgentProof-enabled fetch function
 */
export function createAgentProofFetch(config: LangChainAgentConfig) {
  const wrapper = new AgentProofFetch(config);
  return wrapper.fetch.bind(wrapper);
}
