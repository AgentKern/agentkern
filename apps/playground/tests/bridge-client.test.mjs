import assert from 'node:assert/strict'
import { describe, it } from 'node:test'

import { createBridgeClient } from '../src/bridgeClient.js'

const jsonResponse = (status, payload) =>
  new Response(JSON.stringify(payload), {
    status,
    headers: { 'content-type': 'application/json' },
  })

describe('bridge client integration', () => {
  it('checks bridge availability via /health', async () => {
    const fetchCalls = []
    const fetchImpl = async (url, init) => {
      fetchCalls.push({ url, init })
      return jsonResponse(200, { status: 'ok' })
    }

    const client = createBridgeClient({
      baseUrl: 'https://agentkern.test',
      fetchImpl,
    })

    const available = await client.checkAvailability()

    assert.equal(available, true)
    assert.equal(fetchCalls.length, 1)
    assert.equal(fetchCalls[0].url, 'https://agentkern.test/health')
    assert.equal(fetchCalls[0].init?.method, undefined)
  })

  it('caches auth token across multiple authenticated calls', async () => {
    let loginCalls = 0
    const authHeaders = []

    const fetchImpl = async (url, init) => {
      if (url.endsWith('/api/v1/auth/login')) {
        loginCalls += 1
        return jsonResponse(200, { token: 'token-1', token_type: 'Bearer' })
      }

      if (url.endsWith('/api/v1/gate/verify')) {
        const headers = new Headers(init?.headers)
        authHeaders.push(headers.get('authorization'))
        return jsonResponse(200, {
          allowed: true,
          final_risk_score: 15,
          reasoning: 'ok',
          blocking_policies: [],
        })
      }

      throw new Error(`Unexpected URL: ${url}`)
    }

    const client = createBridgeClient({
      baseUrl: 'https://agentkern.test',
      fetchImpl,
    })

    await client.verifyGate({
      agent_id: 'agent-1',
      action: 'read_data',
      namespace: 'playground',
      context: {},
    })
    await client.verifyGate({
      agent_id: 'agent-1',
      action: 'read_data',
      namespace: 'playground',
      context: {},
    })

    assert.equal(loginCalls, 1)
    assert.deepEqual(authHeaders, ['Bearer token-1', 'Bearer token-1'])
  })

  it('retries auth flow once after a 401 response', async () => {
    let loginCalls = 0
    let verifyCalls = 0
    const verifyAuthHeaders = []

    const fetchImpl = async (url, init) => {
      if (url.endsWith('/api/v1/auth/login')) {
        loginCalls += 1
        return jsonResponse(200, {
          token: `token-${loginCalls}`,
          token_type: 'Bearer',
        })
      }

      if (url.endsWith('/api/v1/gate/verify')) {
        verifyCalls += 1
        const headers = new Headers(init?.headers)
        verifyAuthHeaders.push(headers.get('authorization'))

        if (verifyCalls === 1) {
          return jsonResponse(401, { error: 'expired token' })
        }

        return jsonResponse(200, {
          allowed: true,
          final_risk_score: 3,
          reasoning: 'recovered',
          blocking_policies: [],
        })
      }

      throw new Error(`Unexpected URL: ${url}`)
    }

    const client = createBridgeClient({
      baseUrl: 'https://agentkern.test',
      fetchImpl,
    })

    const result = await client.verifyGate({
      agent_id: 'agent-1',
      action: 'read_data',
      namespace: 'playground',
      context: {},
    })

    assert.equal(result.allowed, true)
    assert.equal(loginCalls, 2)
    assert.equal(verifyCalls, 2)
    assert.deepEqual(verifyAuthHeaders, ['Bearer token-1', 'Bearer token-2'])
  })

  it('returns lock conflict payload for 409 responses', async () => {
    const fetchImpl = async (url) => {
      if (url.endsWith('/api/v1/auth/login')) {
        return jsonResponse(200, { token: 'token-1', token_type: 'Bearer' })
      }

      if (url.endsWith('/api/v1/arbiter/locks')) {
        return jsonResponse(409, { locked: false, error: 'resource busy' })
      }

      throw new Error(`Unexpected URL: ${url}`)
    }

    const client = createBridgeClient({
      baseUrl: 'https://agentkern.test',
      fetchImpl,
    })

    const response = await client.acquireArbiterLock({
      agent_id: 'agent-1',
      resource: 'database:accounts',
      priority: 5,
    })

    assert.equal(response.status, 409)
    assert.equal(response.ok, false)
    assert.equal(response.payload.error, 'resource busy')
  })

  it('creates identity agent and fetches reputation with authenticated endpoints', async () => {
    let capturedCreatePayload = null

    const fetchImpl = async (url, init) => {
      if (url.endsWith('/api/v1/auth/login')) {
        return jsonResponse(200, { token: 'token-1', token_type: 'Bearer' })
      }

      if (url.endsWith('/api/v1/identity/agents') && init?.method === 'POST') {
        capturedCreatePayload = JSON.parse(init?.body ?? '{}')
        return jsonResponse(201, {
          id: capturedCreatePayload.id,
          name: capturedCreatePayload.name,
        })
      }

      if (url.endsWith('/api/v1/identity/reputation/agent-test')) {
        return jsonResponse(200, { score: 88 })
      }

      throw new Error(`Unexpected URL: ${url}`)
    }

    const client = createBridgeClient({
      baseUrl: 'https://agentkern.test',
      fetchImpl,
    })

    const created = await client.createIdentityAgent({
      id: 'agent-test',
      name: 'playground',
      version: '1.0.0',
      namespace: 'playground',
    })
    const reputation = await client.getReputation(created.id)

    assert.equal(created.id, 'agent-test')
    assert.equal(capturedCreatePayload.namespace, 'playground')
    assert.equal(reputation.score, 88)
  })
})
