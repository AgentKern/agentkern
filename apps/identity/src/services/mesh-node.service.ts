/**
 * AgentKern Identity - Mesh Node Service
 * 
 * Manages the local Trust Mesh node, peer connections,
 * and message propagation to the decentralized network.
 */

import { Injectable, Logger, OnModuleInit, OnModuleDestroy } from '@nestjs/common';
import * as jose from 'jose';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import {
  MeshNode,
  MeshNodeType,
  MeshMessage,
  MeshMessageType,
  TrustUpdatePayload,
  RevocationPayload,
  HandshakePayload,
  TrustEvent,
  createHandshake,
  createTrustUpdate,
  createRevocation,
} from '../domain/mesh.entity';
import { AuditLoggerService, AuditEventType } from './audit-logger.service';
import { MeshPeerEntity, NodeIdentityEntity } from '../entities/mesh-node.entity';

interface PeerConnection {
  node: MeshNode;
  connected: boolean;
  messageQueue: MeshMessage[];
  lastPing: number;
}

@Injectable()
export class MeshNodeService implements OnModuleInit, OnModuleDestroy {
  private readonly logger = new Logger(MeshNodeService.name);
  
  // Local node configuration
  private nodeId: string;
  private nodeType: MeshNodeType = MeshNodeType.FULL;
  private publicKey: string = '';
  private privateKey: string = '';
  
  // Peer management
  private peers: Map<string, PeerConnection> = new Map();
  private knownNodes: Map<string, MeshNode> = new Map();
  
  // Message tracking (prevent duplicates)
  private processedMessages: Set<string> = new Set();
  private readonly MESSAGE_TTL = 5 * 60 * 1000; // 5 minutes
  
  // Bootstrap nodes
  private readonly BOOTSTRAP_NODES = [
    'wss://mesh-1.agentkern-identity.io:8080',
    'wss://mesh-2.agentkern-identity.io:8080',
    'wss://mesh-3.agentkern-identity.io:8080',
  ];

  constructor(
    private readonly auditLogger: AuditLoggerService,
    @InjectRepository(MeshPeerEntity)
    private readonly peerRepository: Repository<MeshPeerEntity>,
    @InjectRepository(NodeIdentityEntity)
    private readonly identityRepository: Repository<NodeIdentityEntity>,
  ) {}

  async onModuleInit(): Promise<void> {
    // 1. Load or Generate node identity
    let identity = await this.identityRepository.findOne({ where: { id: 'local' } });

    if (!identity) {
      const { publicKey, privateKey } = await jose.generateKeyPair('Ed25519');
      identity = this.identityRepository.create({
        id: 'local',
        nodeId: crypto.randomUUID(),
        publicKey: await jose.exportSPKI(publicKey),
        privateKey: await jose.exportPKCS8(privateKey),
        type: MeshNodeType.FULL,
      });
      await this.identityRepository.save(identity);
      this.logger.log(`Generated new node identity: ${identity.nodeId}`);
    } else {
      this.logger.log(`Loaded existing node identity: ${identity.nodeId}`);
    }

    this.nodeId = identity.nodeId;
    this.publicKey = identity.publicKey;
    this.privateKey = identity.privateKey;
    this.nodeType = identity.type;

    // 2. Load known peers from database
    const dbPeers = await this.peerRepository.find();
    for (const peer of dbPeers) {
      this.knownNodes.set(peer.id, {
        ...peer,
        connectedAt: peer.connectedAt.toISOString(),
        lastSeen: peer.lastSeen.toISOString(),
      });
    }

    this.logger.log(`Mesh node initialized. Known peers: ${dbPeers.length}`);
    
    // In production, connect to bootstrap nodes
    // For now, we run in standalone mode
  }

  async onModuleDestroy(): Promise<void> {
    // Disconnect from all peers
    for (const [peerId, peer] of this.peers) {
      this.logger.log(`Disconnecting from peer: ${peerId}`);
      peer.connected = false;
    }
    this.peers.clear();
  }

  /**
   * Get local node info
   */
  getNodeInfo(): MeshNode {
    return {
      id: this.nodeId,
      publicKey: this.publicKey,
      type: this.nodeType,
      endpoints: [], // Would be filled with public endpoints
      capabilities: ['FULL_NODE', 'DNS_RESOLVER'],
      connectedAt: new Date().toISOString(),
      lastSeen: new Date().toISOString(),
      trustScore: 1000, // Self-score
    };
  }

  /**
   * Broadcast a trust update to all peers
   */
  async broadcastTrustUpdate(
    agentId: string,
    principalId: string,
    trustScore: number,
    event: TrustEvent,
    previousScore?: number,
  ): Promise<void> {
    const message = createTrustUpdate(
      this.nodeId,
      agentId,
      principalId,
      trustScore,
      event,
      previousScore,
    );
    
    const signedMessage = await this.signMessage(message);
    await this.broadcast(signedMessage);
    
    this.logger.debug(`Broadcasted trust update: ${agentId}:${principalId} → ${trustScore}`);
  }

  /**
   * Broadcast a revocation to all peers (critical priority)
   */
  async broadcastRevocation(
    agentId: string,
    principalId: string,
    reason: string,
    revokedBy: string,
  ): Promise<void> {
    const message = createRevocation(
      this.nodeId,
      agentId,
      principalId,
      reason,
      revokedBy,
      'CRITICAL',
    );
    
    const signedMessage = await this.signMessage(message);
    await this.broadcast(signedMessage, true); // Priority broadcast
    
    this.auditLogger.log({
      type: AuditEventType.KEY_REVOKED,
      agentId,
      principalId,
      success: true,
      metadata: { reason, broadcastedToMesh: true },
    });
    
    this.logger.warn(`Broadcasted revocation: ${agentId}:${principalId}`);
  }

  /**
   * Connect to a peer node
   */
  async connectToPeer(endpoint: string): Promise<boolean> {
    try {
      // In production, this would establish a WebSocket connection
      this.logger.log(`Connecting to peer: ${endpoint}`);
      
      // Create peer node entry
      const peerId = crypto.randomUUID();
      const peerNode: MeshNode = {
        id: peerId,
        publicKey: '',
        type: MeshNodeType.FULL,
        endpoints: [endpoint],
        capabilities: [],
        connectedAt: new Date().toISOString(),
        lastSeen: new Date().toISOString(),
        trustScore: 500,
      };
      
      this.peers.set(peerId, {
        node: peerNode,
        connected: true,
        messageQueue: [],
        lastPing: Date.now(),
      });
      
      // Send handshake
      const handshake = createHandshake(
        this.nodeId,
        this.nodeType,
        this.publicKey,
      );
      
      const signedHandshake = await this.signMessage(handshake);
      // In production, send via WebSocket
      
      this.logger.log(`Connected to peer: ${peerId}`);
      return true;
    } catch (error) {
      this.logger.error(`Failed to connect to peer: ${endpoint}`, error);
      return false;
    }
  }

  /**
   * Disconnect from a peer
   */
  disconnectFromPeer(peerId: string): void {
    const peer = this.peers.get(peerId);
    if (peer) {
      peer.connected = false;
      this.peers.delete(peerId);
      this.logger.log(`Disconnected from peer: ${peerId}`);
    }
  }

  /**
   * Get connected peers
   */
  getConnectedPeers(): MeshNode[] {
    return Array.from(this.peers.values())
      .filter(p => p.connected)
      .map(p => p.node);
  }

  /**
   * Get mesh statistics
   */
  getMeshStats(): {
    nodeId: string;
    nodeType: MeshNodeType;
    connectedPeers: number;
    processedMessages: number;
    uptime: number;
  } {
    return {
      nodeId: this.nodeId,
      nodeType: this.nodeType,
      connectedPeers: this.peers.size,
      processedMessages: this.processedMessages.size,
      uptime: process.uptime(),
    };
  }

  /**
   * Handle incoming message from peer
   */
  async handleMessage(message: MeshMessage): Promise<void> {
    // Check for duplicate
    if (this.processedMessages.has(message.id)) {
      return;
    }
    this.processedMessages.add(message.id);
    
    // Verify signature
    const isValid = await this.verifyMessage(message);
    if (!isValid) {
      this.logger.warn(`Invalid message signature: ${message.id}`);
      return;
    }
    
    // Process based on type
    switch (message.type) {
      case MeshMessageType.TRUST_UPDATE:
        await this.handleTrustUpdate(message as MeshMessage<TrustUpdatePayload>);
        break;
      case MeshMessageType.REVOCATION:
        await this.handleRevocation(message as MeshMessage<RevocationPayload>);
        break;
      case MeshMessageType.HANDSHAKE:
        await this.handleHandshake(message as MeshMessage<HandshakePayload>);
        break;
      default:
        this.logger.debug(`Unhandled message type: ${message.type}`);
    }
    
    // Forward to other peers (gossip protocol)
    await this.forward(message);
  }

  private async handleTrustUpdate(message: MeshMessage<TrustUpdatePayload>): Promise<void> {
    const { agentId, principalId, trustScore, event } = message.payload;
    this.logger.debug(`Received trust update from mesh: ${agentId}:${principalId} → ${trustScore}`);
    
    // In production, update local DNS cache
  }

  private async handleRevocation(message: MeshMessage<RevocationPayload>): Promise<void> {
    const { agentId, principalId, reason } = message.payload;
    this.logger.warn(`Received revocation from mesh: ${agentId}:${principalId} - ${reason}`);
    
    // In production, immediately update local DNS cache
  }

  private async handleHandshake(message: MeshMessage<HandshakePayload>): Promise<void> {
    const { nodeId, nodeType, publicKey } = message.payload;
    this.logger.log(`Received handshake from: ${nodeId}`);
    
    // Store peer info
    const peerInfo: MeshNode = {
      id: nodeId,
      publicKey,
      type: nodeType,
      endpoints: [],
      capabilities: message.payload.capabilities,
      connectedAt: new Date().toISOString(),
      lastSeen: new Date().toISOString(),
      trustScore: 500,
    };

    this.knownNodes.set(nodeId, peerInfo);

    // Persist to DB
    await this.peerRepository.save({
      ...peerInfo,
      connectedAt: new Date(peerInfo.connectedAt),
      lastSeen: new Date(peerInfo.lastSeen),
    });
  }

  /**
   * Sign a message with node's private key
   */
  private async signMessage<T>(message: Omit<MeshMessage<T>, 'signature'>): Promise<MeshMessage<T>> {
    const privateKey = await jose.importPKCS8(this.privateKey, 'EdDSA');
    const payload = JSON.stringify(message);
    
    const jws = await new jose.CompactSign(new TextEncoder().encode(payload))
      .setProtectedHeader({ alg: 'EdDSA' })
      .sign(privateKey);
    
    const signature = jws.split('.')[2];
    
    return {
      ...message,
      signature,
    } as MeshMessage<T>;
  }

  /**
   * Verify a message signature
   */
  private async verifyMessage(message: MeshMessage): Promise<boolean> {
    try {
      const peer = this.knownNodes.get(message.fromNode);
      if (!peer) {
        // Unknown node - accept for now, verify later
        return true;
      }
      
      const publicKey = await jose.importSPKI(peer.publicKey, 'EdDSA');
      
      // Reconstruct the signed message
      const { signature, ...messageWithoutSig } = message;
      const payload = JSON.stringify(messageWithoutSig);
      
      // Verify
      await jose.compactVerify(
        `eyJhbGciOiJFZERTQSJ9.${Buffer.from(payload).toString('base64url')}.${signature}`,
        publicKey,
      );
      
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Broadcast message to all connected peers
   */
  private async broadcast(message: MeshMessage, priority = false): Promise<void> {
    for (const [peerId, peer] of this.peers) {
      if (!peer.connected) continue;
      
      if (priority) {
        // Send immediately for critical messages
        // In production, send via WebSocket
        this.logger.debug(`Priority broadcast to ${peerId}`);
      } else {
        // Queue for batch sending
        peer.messageQueue.push(message);
      }
    }
  }

  /**
   * Forward message to peers (gossip)
   */
  private async forward(message: MeshMessage): Promise<void> {
    // Don't forward our own messages
    if (message.fromNode === this.nodeId) return;
    
    // Forward to peers who haven't seen it
    for (const [peerId, peer] of this.peers) {
      if (!peer.connected) continue;
      if (peerId === message.fromNode) continue; // Don't send back to source
      
      // In production, check if peer has already processed this message
      // For now, just log
      this.logger.debug(`Forwarding ${message.type} to ${peerId}`);
    }
  }
}
