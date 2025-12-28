/**
 * AgentKernIdentity - Liability Proof Domain Entity
 * 
 * A Liability Proof is a cryptographic attestation that proves:
 * 1. A specific human authorized a specific action
 * 2. The authorization was made via a hardware-bound credential (Passkey)
 * 3. The authorizer accepts liability for the agent's action
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

export interface Liability {
  acceptedBy: 'principal' | 'agent_operator';
  termsVersion: string;
  disputeWindowHours: number;
}

export interface LiabilityProofPayload {
  version: string;
  proofId: string;
  issuedAt: string;
  expiresAt: string;
  principal: Principal;
  agent: Agent;
  intent: Intent;
  constraints?: Constraints;
  liability: Liability;
}

export interface LiabilityProof {
  version: string;
  payload: LiabilityProofPayload;
  signature: string;
}

/**
 * Parse a Liability Proof from X-AgentKernIdentity header
 */
export function parseProofHeader(header: string): LiabilityProof | null {
  try {
    const parts = header.split('.');
    if (parts.length !== 3) return null;

    const [version, payloadBase64, signature] = parts;
    const payloadJson = Buffer.from(payloadBase64, 'base64url').toString('utf-8');
    const payload = JSON.parse(payloadJson) as LiabilityProofPayload;

    return { version, payload, signature };
  } catch {
    return null;
  }
}

/**
 * Serialize a Liability Proof to X-AgentKernIdentity header format
 */
export function serializeProofHeader(proof: LiabilityProof): string {
  const payloadJson = JSON.stringify(proof.payload);
  const payloadBase64 = Buffer.from(payloadJson).toString('base64url');
  return `${proof.version}.${payloadBase64}.${proof.signature}`;
}

/**
 * Create a new Liability Proof payload (unsigned)
 */
export function createProofPayload(
  principal: Principal,
  agent: Agent,
  intent: Intent,
  options: {
    constraints?: Constraints;
    expiresInSeconds?: number;
    disputeWindowHours?: number;
  } = {},
): LiabilityProofPayload {
  const now = new Date();
  const expiresAt = new Date(now.getTime() + (options.expiresInSeconds || 300) * 1000);

  return {
    version: '1.0',
    proofId: crypto.randomUUID(),
    issuedAt: now.toISOString(),
    expiresAt: expiresAt.toISOString(),
    principal,
    agent,
    intent,
    constraints: options.constraints,
    liability: {
      acceptedBy: 'principal',
      termsVersion: '1.0',
      disputeWindowHours: options.disputeWindowHours || 72,
    },
  };
}
