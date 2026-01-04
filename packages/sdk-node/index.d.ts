/**
 * AgentKern SDK for Node.js
 *
 * Native Ed25519 cryptography and Liability Proof creation.
 *
 * @example
 * ```typescript
 * import { Agent, LiabilityProof } from '@agentkern/sdk';
 *
 * // Generate a new agent
 * const agent = Agent.generate('my-agent');
 *
 * // Create a liability proof
 * const proof = agent.createProof('payment:transfer:100');
 *
 * // Verify a proof
 * const isValid = Agent.verifyProof(proof);
 * ```
 */

/** SDK version */
export const VERSION: string;

/** Agent configuration options */
export interface AgentConfig {
  /** Agent name/label */
  name: string;
  /** Default proof expiry in seconds */
  proofExpirySeconds?: number;
  /** Issuer identifier (DID or domain) */
  issuer?: string;
}

/**
 * Agent - The core identity unit in AgentKern.
 *
 * Holds an Ed25519 keypair and can:
 * - Sign arbitrary data
 * - Create Liability Proofs
 * - Verify other agents' proofs
 */
export class Agent {
  /** Generate a new Agent with a random Ed25519 keypair */
  static generate(name: string): Agent;

  /** Generate a new Agent with custom configuration */
  static generateWithConfig(config: AgentConfig): Agent;

  /** Restore an Agent from an existing keypair seed (base64url encoded) */
  static fromSeed(name: string, seedBase64: string): Agent;

  /** Verify a Liability Proof */
  static verifyProof(proof: LiabilityProof): boolean;

  /** Agent's unique ID (DID format) */
  readonly id: string;

  /** Agent's name */
  readonly name: string;

  /** Agent's public key (base64url encoded) */
  readonly publicKey: string;

  /** Keypair seed for persistence (base64url encoded) - SENSITIVE */
  readonly seed: string;

  /** Sign arbitrary data (returns base64url signature) */
  sign(data: Buffer): string;

  /** Sign a string message (returns base64url signature) */
  signMessage(message: string): string;

  /** Create a Liability Proof for an action */
  createProof(action: string): LiabilityProof;

  /** Create a Liability Proof with custom expiry (seconds) */
  createProofWithExpiry(action: string, expirySeconds: number): LiabilityProof;
}

/** Liability Proof - A signed JWT proving authorization */
export interface LiabilityProof {
  /** JWT algorithm */
  alg: string;
  /** JWT type */
  typ: string;
  /** Key ID (public key) */
  kid: string;
  /** Issuer (DID) */
  issuer: string;
  /** Subject (agent DID) */
  subject: string;
  /** Authorized action */
  action: string;
  /** Issued at (Unix timestamp) */
  issuedAt: number;
  /** Expiration (Unix timestamp) */
  expiresAt: number;
  /** JWT ID */
  jti: string;
  /** Full raw JWT string */
  jwt: string;
}

/** Parse a JWT string into a LiabilityProof */
export function parseProof(jwt: string): LiabilityProof;

/** Check if a proof is expired */
export function isProofExpired(proof: LiabilityProof): boolean;

/** A2A Message types */
export type MessageType =
  | 'Request'
  | 'Response'
  | 'Notification'
  | 'Error'
  | 'Ping'
  | 'Pong'
  | 'Capabilities';

/** Create an A2A request message */
export function createA2aRequest(from: string, to: string, payload: unknown): string;

/** Create an A2A notification message */
export function createA2aNotification(from: string, to: string, payload: unknown): string;

/** Parse an A2A message from JSON */
export function parseA2aMessage(json: string): unknown;
