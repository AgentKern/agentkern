/**
 * AgentProof - Trust Mesh Entities
 * 
 * Domain entities for the decentralized Trust Mesh network.
 */

export enum MeshNodeType {
  FULL = 'FULL',
  LIGHT = 'LIGHT',
  BRIDGE = 'BRIDGE',
}

export enum MeshMessageType {
  HANDSHAKE = 'HANDSHAKE',
  PING = 'PING',
  PONG = 'PONG',
  TRUST_UPDATE = 'TRUST_UPDATE',
  REVOCATION = 'REVOCATION',
  PEER_ANNOUNCE = 'PEER_ANNOUNCE',
  PEER_REQUEST = 'PEER_REQUEST',
  SYNC_REQUEST = 'SYNC_REQUEST',
  SYNC_RESPONSE = 'SYNC_RESPONSE',
}

export enum TrustEvent {
  VERIFICATION_SUCCESS = 'VERIFICATION_SUCCESS',
  VERIFICATION_FAILURE = 'VERIFICATION_FAILURE',
  REGISTRATION = 'REGISTRATION',
  REVOCATION = 'REVOCATION',
  REINSTATEMENT = 'REINSTATEMENT',
  SCORE_UPDATE = 'SCORE_UPDATE',
}

export interface MeshNode {
  id: string;
  publicKey: string;
  type: MeshNodeType;
  endpoints: string[];
  capabilities: string[];
  connectedAt: string;
  lastSeen: string;
  trustScore: number; // Node reputation
}

export interface MeshMessage<T = unknown> {
  type: MeshMessageType;
  version: string;
  id: string;
  timestamp: string;
  fromNode: string;
  payload: T;
  signature: string;
}

export interface HandshakePayload {
  nodeId: string;
  nodeType: MeshNodeType;
  publicKey: string;
  capabilities: string[];
  protocolVersion: string;
}

export interface TrustUpdatePayload {
  agentId: string;
  principalId: string;
  trustScore: number;
  event: TrustEvent;
  previousScore?: number;
}

export interface RevocationPayload {
  agentId: string;
  principalId: string;
  reason: string;
  revokedBy: string;
  priority: 'NORMAL' | 'HIGH' | 'CRITICAL';
}

export interface PeerAnnouncePayload {
  nodeId: string;
  nodeType: MeshNodeType;
  endpoints: string[];
  capabilities: string[];
}

export interface SyncRequestPayload {
  fromTimestamp: string;
  limit: number;
}

export interface SyncResponsePayload {
  records: MeshTrustRecord[];
  hasMore: boolean;
  nextTimestamp?: string;
}

export interface MeshTrustRecord {
  agentId: string;
  principalId: string;
  trustScore: number;
  trusted: boolean;
  revoked: boolean;
  lastUpdated: string;
  updateHistory: TrustUpdateEntry[];
  signatures: NodeSignature[];
}

export interface TrustUpdateEntry {
  timestamp: string;
  event: TrustEvent;
  fromNode: string;
  scoreDelta: number;
}

export interface NodeSignature {
  nodeId: string;
  signature: string;
  timestamp: string;
}

/**
 * Create a new mesh message
 */
export function createMeshMessage<T>(
  type: MeshMessageType,
  fromNode: string,
  payload: T,
): Omit<MeshMessage<T>, 'signature'> {
  return {
    type,
    version: '1.0',
    id: crypto.randomUUID(),
    timestamp: new Date().toISOString(),
    fromNode,
    payload,
  };
}

/**
 * Create a handshake message
 */
export function createHandshake(
  nodeId: string,
  nodeType: MeshNodeType,
  publicKey: string,
  capabilities: string[] = ['FULL_NODE'],
): Omit<MeshMessage<HandshakePayload>, 'signature'> {
  return createMeshMessage(MeshMessageType.HANDSHAKE, nodeId, {
    nodeId,
    nodeType,
    publicKey,
    capabilities,
    protocolVersion: '1.0',
  });
}

/**
 * Create a trust update message
 */
export function createTrustUpdate(
  fromNode: string,
  agentId: string,
  principalId: string,
  trustScore: number,
  event: TrustEvent,
  previousScore?: number,
): Omit<MeshMessage<TrustUpdatePayload>, 'signature'> {
  return createMeshMessage(MeshMessageType.TRUST_UPDATE, fromNode, {
    agentId,
    principalId,
    trustScore,
    event,
    previousScore,
  });
}

/**
 * Create a revocation message
 */
export function createRevocation(
  fromNode: string,
  agentId: string,
  principalId: string,
  reason: string,
  revokedBy: string,
  priority: 'NORMAL' | 'HIGH' | 'CRITICAL' = 'CRITICAL',
): Omit<MeshMessage<RevocationPayload>, 'signature'> {
  return createMeshMessage(MeshMessageType.REVOCATION, fromNode, {
    agentId,
    principalId,
    reason,
    revokedBy,
    priority,
  });
}
