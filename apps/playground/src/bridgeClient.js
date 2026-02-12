const parseJsonOrText = (text) => {
  if (!text) {
    return {};
  }

  try {
    return JSON.parse(text);
  } catch {
    return text;
  }
};

const extractErrorMessage = (payload, fallback) => {
  if (payload && typeof payload === 'object') {
    const value = payload.error;
    if (typeof value === 'string' && value.trim().length > 0) {
      return value;
    }
  }

  if (typeof payload === 'string' && payload.trim().length > 0) {
    return payload;
  }

  return fallback;
};

export class BridgeApiError extends Error {
  constructor(message, status, payload) {
    super(message);
    this.name = 'BridgeApiError';
    this.status = status;
    this.payload = payload;
  }
}

export function createBridgeClient(options = {}) {
  const baseUrl = (options.baseUrl || 'http://localhost:3000').replace(/\/+$/, '');
  const authAgentId = options.authAgentId || 'playground-auth-agent';
  const authSecret = options.authSecret || 'playground-auth-secret';
  const fetchImpl = options.fetchImpl || globalThis.fetch;

  if (typeof fetchImpl !== 'function') {
    throw new Error('Bridge client requires a fetch implementation');
  }

  let cachedAuthHeader = null;

  const requestRaw = async (path, init = {}) => {
    const response = await fetchImpl(`${baseUrl}${path}`, init);
    const bodyText = await response.text();
    const payload = parseJsonOrText(bodyText);
    return { status: response.status, ok: response.ok, payload };
  };

  const requestJson = async (path, init = {}) => {
    const response = await requestRaw(path, init);
    if (!response.ok) {
      throw new BridgeApiError(
        `${response.status}: ${extractErrorMessage(response.payload, 'Request failed')}`,
        response.status,
        response.payload,
      );
    }
    return response.payload;
  };

  const getAuthHeader = async () => {
    if (cachedAuthHeader) {
      return cachedAuthHeader;
    }

    const loginResponse = await requestJson('/api/v1/auth/login', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        agent_id: authAgentId,
        secret: authSecret,
      }),
    });

    const tokenType = loginResponse.token_type || 'Bearer';
    cachedAuthHeader = `${tokenType} ${loginResponse.token}`;
    return cachedAuthHeader;
  };

  const requestJsonWithAuth = async (path, init = {}, retry = true) => {
    const authHeader = await getAuthHeader();
    const headers = new Headers(init.headers);

    if (init.body && !headers.has('Content-Type')) {
      headers.set('Content-Type', 'application/json');
    }
    headers.set('Authorization', authHeader);

    try {
      return await requestJson(path, { ...init, headers });
    } catch (error) {
      if (retry && error instanceof BridgeApiError && error.status === 401) {
        cachedAuthHeader = null;
        return requestJsonWithAuth(path, init, false);
      }
      throw error;
    }
  };

  const requestRawWithAuth = async (path, init = {}, retry = true) => {
    const authHeader = await getAuthHeader();
    const headers = new Headers(init.headers);

    if (init.body && !headers.has('Content-Type')) {
      headers.set('Content-Type', 'application/json');
    }
    headers.set('Authorization', authHeader);

    const response = await requestRaw(path, { ...init, headers });
    if (retry && response.status === 401) {
      cachedAuthHeader = null;
      return requestRawWithAuth(path, init, false);
    }
    return response;
  };

  return {
    clearAuthCache() {
      cachedAuthHeader = null;
    },

    async checkAvailability() {
      try {
        const health = await requestJson('/health');
        return health.status === 'ok';
      } catch {
        return false;
      }
    },

    getAuthHeader,

    async createIdentityAgent(payload) {
      return requestJsonWithAuth('/api/v1/identity/agents', {
        method: 'POST',
        body: JSON.stringify(payload),
      });
    },

    async getReputation(agentId) {
      return requestJsonWithAuth(`/api/v1/identity/reputation/${encodeURIComponent(agentId)}`);
    },

    async verifyGate(payload) {
      return requestJsonWithAuth('/api/v1/gate/verify', {
        method: 'POST',
        body: JSON.stringify(payload),
      });
    },

    async storeSynapseMemory(payload) {
      return requestJsonWithAuth('/api/v1/synapse/memory/store', {
        method: 'POST',
        body: JSON.stringify(payload),
      });
    },

    async acquireArbiterLock(payload) {
      return requestRawWithAuth('/api/v1/arbiter/locks', {
        method: 'POST',
        body: JSON.stringify(payload),
      });
    },

    async listIdentityAgents() {
      return requestJsonWithAuth('/api/v1/identity/agents');
    },
  };
}
