# Trust Mesh Protocol Specification v1.0

## Overview

Trust Mesh is a decentralized P2P network for sharing trust signals across AgentProof nodes.
It provides resilience, censorship resistance, and real-time trust propagation.

---

## Architecture

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Node A    │◄───►│   Node B    │◄───►│   Node C    │
│ (Company X) │     │ (Company Y) │     │ (Company Z) │
└─────────────┘     └─────────────┘     └─────────────┘
      │                   │                   │
      └───────────────────┴───────────────────┘
                          │
                    ┌─────────────┐
                    │  Consensus  │
                    │   Layer     │
                    └─────────────┘
```

---

## Node Types

| Type | Description | Requirements |
|------|-------------|--------------|
| **Full Node** | Stores complete trust history | 100GB+, always online |
| **Light Node** | Caches recent data, queries full nodes | 1GB, intermittent |
| **Bridge Node** | Connects to external systems | API access |

---

## Message Types

### 1. Trust Propagation
```json
{
  "type": "TRUST_UPDATE",
  "version": "1.0",
  "id": "msg-uuid",
  "timestamp": "2025-12-24T12:00:00Z",
  "payload": {
    "agentId": "cursor-agent",
    "principalId": "user-123",
    "trustScore": 750,
    "event": "VERIFICATION_SUCCESS",
    "signature": "base64-signature"
  }
}
```

### 2. Revocation Broadcast
```json
{
  "type": "REVOCATION",
  "version": "1.0",
  "id": "msg-uuid",
  "timestamp": "2025-12-24T12:00:00Z",
  "payload": {
    "agentId": "cursor-agent",
    "principalId": "user-123",
    "reason": "compromised",
    "signature": "base64-signature"
  },
  "priority": "CRITICAL"
}
```

### 3. Peer Discovery
```json
{
  "type": "PEER_ANNOUNCE",
  "version": "1.0",
  "nodeId": "node-uuid",
  "endpoints": ["wss://node.example.com:8080"],
  "capabilities": ["FULL_NODE", "DNS_RESOLVER"]
}
```

---

## Consensus Mechanism

### Simple Majority Consensus
For trust score updates:
1. Node receives verification event
2. Broadcasts to connected peers
3. Peers validate signature
4. If >50% of peers accept, update is committed

### Immediate Propagation (No Consensus)
For critical events:
- Revocations
- Key rotations
- Security alerts

These propagate immediately without waiting for consensus.

---

## Peer Discovery

### Bootstrap Nodes
Hard-coded list of known reliable nodes:
```
wss://mesh-1.agentproof.io:8080
wss://mesh-2.agentproof.io:8080
wss://mesh-3.agentproof.io:8080
```

### DHT-Based Discovery
Use Kademlia DHT for finding peers:
- Node ID: SHA-256 of public key
- XOR distance for routing
- 20 nodes per bucket

---

## Security

### Message Signing
All messages signed with node's Ed25519 key.

### Spam Prevention
- Rate limiting: 100 messages/minute per peer
- Proof-of-stake: nodes stake reputation
- Blacklist propagation

### Sybil Resistance
- Bootstrap from trusted nodes only
- Reputation-weighted voting
- Challenge-response for new nodes

---

## WebSocket Protocol

### Connection
```
wss://node.example.com:8080/mesh
```

### Handshake
```json
{
  "type": "HANDSHAKE",
  "nodeId": "my-node-id",
  "version": "1.0",
  "capabilities": ["FULL_NODE"],
  "publicKey": "base64-public-key"
}
```

### Heartbeat
```json
{
  "type": "PING",
  "timestamp": "2025-12-24T12:00:00Z"
}
```

---

## Data Storage

### Trust Record Format
```typescript
interface MeshTrustRecord {
  agentId: string;
  principalId: string;
  trustScore: number;
  lastUpdated: string;
  updateHistory: Array<{
    timestamp: string;
    event: string;
    fromNode: string;
  }>;
  signatures: Array<{
    nodeId: string;
    signature: string;
  }>;
}
```

---

## Sync Protocol

### Initial Sync
New nodes sync from multiple peers:
1. Request latest block height
2. Download missing blocks in parallel
3. Verify all signatures
4. Apply to local state

### Incremental Sync
- Subscribe to real-time updates
- Periodic full sync every 24 hours
