export class BridgeApiError extends Error {
  status: number
  payload: unknown
  constructor(message: string, status: number, payload: unknown)
}

export interface BridgeClientOptions {
  baseUrl?: string
  authAgentId?: string
  authSecret?: string
  fetchImpl?: typeof fetch
}

export interface BridgeRawResponse<T = unknown> {
  status: number
  ok: boolean
  payload: T
}

export interface BridgeClient {
  clearAuthCache(): void
  checkAvailability(): Promise<boolean>
  getAuthHeader(): Promise<string>
  createIdentityAgent(payload: {
    id: string
    name: string
    version: string
    namespace?: string
  }): Promise<{ id: string; name: string }>
  getReputation(agentId: string): Promise<{ score?: number }>
  verifyGate(payload: {
    agent_id: string
    action: string
    namespace: string
    context: Record<string, unknown>
  }): Promise<{
    allowed: boolean
    final_risk_score?: number
    reasoning?: string
    blocking_policies?: string[]
  }>
  storeSynapseMemory(payload: Record<string, unknown>): Promise<{ stored?: boolean }>
  acquireArbiterLock(payload: {
    agent_id: string
    resource: string
    priority: number
  }): Promise<BridgeRawResponse<{ locked?: boolean; lock_id?: string; error?: string }>>
  listIdentityAgents(): Promise<{
    agents?: Array<{ id: string; name: string; status?: string }>
  }>
}

export function createBridgeClient(options?: BridgeClientOptions): BridgeClient
