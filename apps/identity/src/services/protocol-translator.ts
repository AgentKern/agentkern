/**
 * AgentKern Nexus - Protocol Translator
 *
 * Translates messages between agent protocols:
 * - A2A (Google Agent-to-Agent Protocol v0.3)
 * - MCP (Anthropic Model Context Protocol 2025-06-18)
 * - AgentKern Native Protocol
 * - ANP, NLIP, AITP (future support)
 */

import { Protocol, NexusMessage } from '../dto/nexus.dto';

/**
 * A2A Message format (Google Agent-to-Agent Protocol)
 * @see https://google.github.io/A2A/
 */
export interface A2AMessage {
  jsonrpc: '2.0';
  id: string | number;
  method: string;
  params?: {
    id?: string;
    sessionId?: string;
    message?: {
      role: 'user' | 'agent';
      parts: Array<{ text?: string; data?: unknown }>;
    };
    metadata?: Record<string, unknown>;
  };
  result?: unknown;
  error?: { code: number; message: string; data?: unknown };
}

/**
 * MCP Message format (Anthropic Model Context Protocol)
 * @see https://modelcontextprotocol.io/
 */
export interface MCPMessage {
  jsonrpc: '2.0';
  id: string | number;
  method: string;
  params?: {
    name?: string;
    arguments?: Record<string, unknown>;
    uri?: string;
    cursor?: string;
  };
  result?: unknown;
  error?: { code: number; message: string; data?: unknown };
}

/**
 * AgentKern Native Message format
 */
export interface AgentKernMessage {
  id: string;
  method: string;
  params: unknown;
  metadata?: {
    proofId?: string;
    trustScore?: number;
    agentId?: string;
    timestamp?: string;
  };
}

/**
 * Protocol Translator - Converts between agent protocols
 */
export class ProtocolTranslator {
  /**
   * Translate message from source to target protocol
   */
  static translate(
    sourceProtocol: Protocol,
    targetProtocol: Protocol,
    message: unknown,
  ): NexusMessage {
    // Parse to unified format
    const unified = this.parseToUnified(sourceProtocol, message);

    // Serialize to target format
    return this.serializeFromUnified(targetProtocol, unified);
  }

  /**
   * Parse protocol-specific message to unified NexusMessage format
   */
  static parseToUnified(protocol: Protocol, message: unknown): NexusMessage {
    const msg = message as Record<string, unknown>;
    const baseId = (msg.id as string) || crypto.randomUUID();

    switch (protocol) {
      case 'a2a':
        return this.parseA2A(msg as unknown as A2AMessage, baseId);

      case 'mcp':
        return this.parseMCP(msg as unknown as MCPMessage, baseId);

      case 'agentkern':
        return this.parseAgentKern(msg as unknown as AgentKernMessage, baseId);

      case 'anp':
      case 'nlip':
      case 'aitp':
        // Future protocol support - pass through with basic mapping
        return {
          id: baseId,
          method: (msg.method as string) || 'unknown',
          params: msg.params || msg,
          sourceProtocol: protocol,
          timestamp: new Date().toISOString(),
          metadata: { originalFormat: protocol },
        };

      default:
        throw new Error(`Unsupported source protocol: ${String(protocol)}`);
    }
  }

  /**
   * Parse A2A message to unified format
   */
  private static parseA2A(msg: A2AMessage, baseId: string): NexusMessage {
    // Extract text content from A2A parts
    let textContent = '';
    if (msg.params?.message?.parts) {
      textContent = msg.params.message.parts
        .map((p) => p.text || '')
        .filter(Boolean)
        .join('\n');
    }

    // Map A2A methods to unified methods
    const methodMap: Record<string, string> = {
      'tasks/send': 'task.create',
      'tasks/get': 'task.get',
      'tasks/cancel': 'task.cancel',
      'tasks/sendSubscribe': 'task.subscribe',
      'message/send': 'message.send',
      'message/stream': 'message.stream',
    };

    return {
      id: String(baseId),
      method: methodMap[msg.method] || msg.method,
      params: {
        taskId: msg.params?.id,
        sessionId: msg.params?.sessionId,
        content: textContent,
        role: msg.params?.message?.role,
        parts: msg.params?.message?.parts,
        ...msg.params?.metadata,
      },
      sourceProtocol: 'a2a',
      timestamp: new Date().toISOString(),
      metadata: {
        originalMethod: msg.method,
        jsonrpc: msg.jsonrpc,
      },
    };
  }

  /**
   * Parse MCP message to unified format
   */
  private static parseMCP(msg: MCPMessage, baseId: string): NexusMessage {
    // Map MCP methods to unified methods
    const methodMap: Record<string, string> = {
      'tools/list': 'tool.list',
      'tools/call': 'tool.call',
      'resources/list': 'resource.list',
      'resources/read': 'resource.read',
      'prompts/list': 'prompt.list',
      'prompts/get': 'prompt.get',
      'sampling/createMessage': 'llm.sample',
    };

    return {
      id: String(msg.id || baseId),
      method: methodMap[msg.method] || msg.method,
      params: {
        toolName: msg.params?.name,
        arguments: msg.params?.arguments,
        uri: msg.params?.uri,
        cursor: msg.params?.cursor,
      },
      sourceProtocol: 'mcp',
      timestamp: new Date().toISOString(),
      metadata: {
        originalMethod: msg.method,
        jsonrpc: msg.jsonrpc,
      },
    };
  }

  /**
   * Parse AgentKern native message to unified format
   */
  private static parseAgentKern(
    msg: AgentKernMessage,
    baseId: string,
  ): NexusMessage {
    return {
      id: msg.id || baseId,
      method: msg.method,
      params: msg.params,
      sourceProtocol: 'agentkern',
      timestamp: msg.metadata?.timestamp || new Date().toISOString(),
      metadata: msg.metadata,
    };
  }

  /**
   * Serialize unified NexusMessage to target protocol format
   */
  static serializeFromUnified(
    targetProtocol: Protocol,
    msg: NexusMessage,
  ): NexusMessage {
    switch (targetProtocol) {
      case 'a2a':
        return this.serializeToA2A(msg);

      case 'mcp':
        return this.serializeToMCP(msg);

      case 'agentkern':
        return this.serializeToAgentKern(msg);

      case 'anp':
      case 'nlip':
      case 'aitp':
        // Future protocol support - return with target marker
        return {
          ...msg,
          targetProtocol,
          metadata: {
            ...msg.metadata,
            targetFormat: targetProtocol,
          },
        };

      default:
        throw new Error(
          `Unsupported target protocol: ${String(targetProtocol)}`,
        );
    }
  }

  /**
   * Serialize to A2A format
   */
  private static serializeToA2A(msg: NexusMessage): NexusMessage {
    const params = msg.params as Record<string, unknown>;

    // Map unified methods back to A2A methods
    const methodMap: Record<string, string> = {
      'task.create': 'tasks/send',
      'task.get': 'tasks/get',
      'task.cancel': 'tasks/cancel',
      'task.subscribe': 'tasks/sendSubscribe',
      'message.send': 'message/send',
      'message.stream': 'message/stream',
    };

    // Build A2A parts from content
    const parts: Array<{ text?: string }> = [];
    if (typeof params?.content === 'string') {
      parts.push({ text: params.content });
    } else if (params?.content) {
      parts.push({ text: JSON.stringify(params.content) });
    }
    if (params?.parts) {
      parts.push(...(params.parts as Array<{ text?: string }>));
    }

    return {
      ...msg,
      id: msg.id,
      method: methodMap[msg.method] || msg.method,
      params: {
        id: params?.taskId,
        sessionId: params?.sessionId,
        message:
          parts.length > 0
            ? {
                role: (params?.role as 'user' | 'agent') || 'user',
                parts,
              }
            : undefined,
        metadata: msg.metadata,
      },
      targetProtocol: 'a2a',
      metadata: {
        ...msg.metadata,
        jsonrpc: '2.0',
      },
    };
  }

  /**
   * Serialize to MCP format
   */
  private static serializeToMCP(msg: NexusMessage): NexusMessage {
    const params = msg.params as Record<string, unknown>;

    // Map unified methods back to MCP methods
    const methodMap: Record<string, string> = {
      'tool.list': 'tools/list',
      'tool.call': 'tools/call',
      'resource.list': 'resources/list',
      'resource.read': 'resources/read',
      'prompt.list': 'prompts/list',
      'prompt.get': 'prompts/get',
      'llm.sample': 'sampling/createMessage',
    };

    return {
      ...msg,
      id: msg.id,
      method: methodMap[msg.method] || msg.method,
      params: {
        name: params?.toolName,
        arguments: params?.arguments,
        uri: params?.uri,
        cursor: params?.cursor,
      },
      targetProtocol: 'mcp',
      metadata: {
        ...msg.metadata,
        jsonrpc: '2.0',
      },
    };
  }

  /**
   * Serialize to AgentKern native format
   */
  private static serializeToAgentKern(msg: NexusMessage): NexusMessage {
    return {
      ...msg,
      targetProtocol: 'agentkern',
      metadata: {
        ...msg.metadata,
        proofId: (msg.metadata?.proofId as string) || undefined,
        trustScore: (msg.metadata?.trustScore as number) || undefined,
      },
    };
  }

  /**
   * Get method mappings for documentation
   */
  static getMethodMappings(): Record<Protocol, Record<string, string>> {
    return {
      a2a: {
        'tasks/send': 'task.create',
        'tasks/get': 'task.get',
        'tasks/cancel': 'task.cancel',
        'message/send': 'message.send',
      },
      mcp: {
        'tools/list': 'tool.list',
        'tools/call': 'tool.call',
        'resources/list': 'resource.list',
        'resources/read': 'resource.read',
      },
      agentkern: {},
      anp: {},
      nlip: {},
      aitp: {},
    };
  }
}
