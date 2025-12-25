import {
  WebSocketGateway,
  WebSocketServer,
  SubscribeMessage,
  OnGatewayInit,
  OnGatewayConnection,
  OnGatewayDisconnect,
  MessageBody,
  ConnectedSocket,
} from '@nestjs/websockets';
import { Logger, UseGuards } from '@nestjs/common';
import { ThrottlerGuard } from '@nestjs/throttler';
import { Server, Socket } from 'socket.io';
import * as jose from 'jose';
import {
  MeshMessageType,
  MeshNodeType,
} from '../domain/mesh.entity';
import type {
  MeshMessage,
  TrustUpdatePayload,
  RevocationPayload,
  HandshakePayload,
  TrustEvent,
} from '../domain/mesh.entity';

interface ConnectedPeer {
  socketId: string;
  nodeId: string;
  nodeType: MeshNodeType;
  publicKey: string;
  connectedAt: Date;
  lastSeen: Date;
}

@WebSocketGateway({
  namespace: '/mesh',
  cors: {
    origin: process.env.MESH_ALLOWED_ORIGINS ? process.env.MESH_ALLOWED_ORIGINS.split(',') : false,
    credentials: true,
  },
})
@UseGuards(ThrottlerGuard)
export class MeshGateway
  implements OnGatewayInit, OnGatewayConnection, OnGatewayDisconnect
{
  @WebSocketServer()
  server: Server;

  private readonly logger = new Logger(MeshGateway.name);
  private peers: Map<string, ConnectedPeer> = new Map();
  private nodeId: string;
  private publicKey: string;
  private privateKey: jose.KeyLike;

  async afterInit(): Promise<void> {
    // Generate node identity
    const keyPair = await jose.generateKeyPair('Ed25519');
    this.publicKey = await jose.exportSPKI(keyPair.publicKey);
    this.privateKey = keyPair.privateKey;
    this.nodeId = crypto.randomUUID();

    this.logger.log(`🌐 Mesh Gateway initialized`);
    this.logger.log(`   Node ID: ${this.nodeId}`);
    this.logger.log(`   WebSocket: ws://localhost:{port}/mesh`);
  }

  handleConnection(client: Socket): void {
    this.logger.log(`➕ Client connected: ${client.id}`);
  }

  handleDisconnect(client: Socket): void {
    // Remove from peers
    for (const [nodeId, peer] of this.peers) {
      if (peer.socketId === client.id) {
        this.peers.delete(nodeId);
        this.logger.log(`➖ Peer disconnected: ${nodeId}`);
        
        // Notify other peers
        this.server.emit('peer:disconnected', { nodeId });
        break;
      }
    }
  }

  /**
   * Handle peer handshake
   */
  @SubscribeMessage('handshake')
  async handleHandshake(
    @ConnectedSocket() client: Socket,
    @MessageBody() payload: HandshakePayload,
  ): Promise<{ success: boolean; nodeId: string; publicKey: string }> {
    this.logger.log(`🤝 Handshake from: ${payload.nodeId}`);

    // Store peer
    this.peers.set(payload.nodeId, {
      socketId: client.id,
      nodeId: payload.nodeId,
      nodeType: payload.nodeType,
      publicKey: payload.publicKey,
      connectedAt: new Date(),
      lastSeen: new Date(),
    });

    // Notify others of new peer
    client.broadcast.emit('peer:connected', {
      nodeId: payload.nodeId,
      nodeType: payload.nodeType,
    });

    // Return our identity
    return {
      success: true,
      nodeId: this.nodeId,
      publicKey: this.publicKey,
    };
  }

  /**
   * Handle trust updates
   */
  @SubscribeMessage('trust:update')
  async handleTrustUpdate(
    @ConnectedSocket() client: Socket,
    @MessageBody() message: MeshMessage<TrustUpdatePayload>,
  ): Promise<{ acknowledged: boolean }> {
    const { agentId, principalId, trustScore, event } = message.payload;

    this.logger.debug(
      `📊 Trust update: ${agentId}:${principalId} → ${trustScore} (${event})`,
    );

    // Update peer's last seen
    this.updatePeerLastSeen(message.fromNode);

    // Broadcast to all other peers (gossip)
    client.broadcast.emit('trust:update', message);

    return { acknowledged: true };
  }

  /**
   * Handle revocations (critical priority)
   */
  @SubscribeMessage('revocation')
  async handleRevocation(
    @ConnectedSocket() client: Socket,
    @MessageBody() message: MeshMessage<RevocationPayload>,
  ): Promise<{ acknowledged: boolean }> {
    const { agentId, principalId, reason } = message.payload;

    this.logger.warn(`🚨 Revocation: ${agentId}:${principalId} - ${reason}`);

    // Update peer's last seen
    this.updatePeerLastSeen(message.fromNode);

    // Broadcast immediately to all peers
    this.server.emit('revocation', message);

    return { acknowledged: true };
  }

  /**
   * Handle ping (keepalive)
   */
  @SubscribeMessage('ping')
  handlePing(
    @ConnectedSocket() client: Socket,
    @MessageBody() data: { nodeId: string },
  ): { pong: true; timestamp: string } {
    this.updatePeerLastSeen(data.nodeId);
    return { pong: true, timestamp: new Date().toISOString() };
  }

  /**
   * Get connected peers
   */
  @SubscribeMessage('peers:list')
  handleListPeers(): { peers: Array<{ nodeId: string; nodeType: string }> } {
    const peerList = Array.from(this.peers.values()).map((p) => ({
      nodeId: p.nodeId,
      nodeType: p.nodeType,
    }));
    return { peers: peerList };
  }

  /**
   * Broadcast trust update to all connected peers
   */
  async broadcastTrustUpdate(
    agentId: string,
    principalId: string,
    trustScore: number,
    event: TrustEvent,
  ): Promise<void> {
    const message: MeshMessage<TrustUpdatePayload> = {
      type: MeshMessageType.TRUST_UPDATE,
      version: '1.0',
      id: crypto.randomUUID(),
      timestamp: new Date().toISOString(),
      fromNode: this.nodeId,
      payload: { agentId, principalId, trustScore, event },
      signature: await this.signPayload({ agentId, principalId, trustScore, event }),
    };

    this.server.emit('trust:update', message);
    this.logger.debug(`📤 Broadcasted trust update to ${this.peers.size} peers`);
  }

  /**
   * Broadcast revocation to all connected peers
   */
  async broadcastRevocation(
    agentId: string,
    principalId: string,
    reason: string,
    revokedBy: string,
  ): Promise<void> {
    const payload: RevocationPayload = {
      agentId,
      principalId,
      reason,
      revokedBy,
      priority: 'CRITICAL',
    };

    const message: MeshMessage<RevocationPayload> = {
      type: MeshMessageType.REVOCATION,
      version: '1.0',
      id: crypto.randomUUID(),
      timestamp: new Date().toISOString(),
      fromNode: this.nodeId,
      payload,
      signature: await this.signPayload(payload),
    };

    this.server.emit('revocation', message);
    this.logger.warn(`📤 Broadcasted revocation to ${this.peers.size} peers`);
  }

  /**
   * Get gateway stats
   */
  getStats(): {
    nodeId: string;
    connectedPeers: number;
    uptime: number;
  } {
    return {
      nodeId: this.nodeId,
      connectedPeers: this.peers.size,
      uptime: process.uptime(),
    };
  }

  private updatePeerLastSeen(nodeId: string): void {
    const peer = this.peers.get(nodeId);
    if (peer) {
      peer.lastSeen = new Date();
    }
  }

  private async signPayload(payload: unknown): Promise<string> {
    const data = JSON.stringify(payload);
    const jws = await new jose.CompactSign(new TextEncoder().encode(data))
      .setProtectedHeader({ alg: 'EdDSA' })
      .sign(this.privateKey);
    return jws.split('.')[2]; // Return just signature part
  }
}
