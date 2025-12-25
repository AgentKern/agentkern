/**
 * Mesh Gateway Full Coverage Tests
 */

import { Test, TestingModule } from '@nestjs/testing';
import { MeshGateway } from './mesh.gateway';
import { MeshMessageType, MeshNodeType } from '../domain/mesh.entity';

describe('MeshGateway Comprehensive', () => {
  let gateway: MeshGateway;
  let mockServer: any;
  let mockSocket: any;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [MeshGateway],
    }).compile();

    gateway = module.get<MeshGateway>(MeshGateway);
    
    // Mock WebSocket server
    mockServer = {
      emit: jest.fn(),
      to: jest.fn().mockReturnThis(),
      in: jest.fn().mockReturnThis(),
    };
    
    // Mock socket client
    mockSocket = {
      id: 'test-socket-id',
      broadcast: { emit: jest.fn() },
      join: jest.fn(),
    };
    
    (gateway as any).server = mockServer;
    await gateway.afterInit();
  });

  describe('afterInit', () => {
    it('should generate node identity', async () => {
      expect((gateway as any).nodeId).toBeDefined();
      expect((gateway as any).publicKey).toBeDefined();
    });

    it('should have a valid UUID node ID', () => {
      const nodeId = (gateway as any).nodeId;
      expect(nodeId).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i);
    });

    it('should have Ed25519 public key', () => {
      const publicKey = (gateway as any).publicKey;
      expect(publicKey).toContain('-----BEGIN PUBLIC KEY-----');
    });
  });

  describe('handleConnection', () => {
    it('should handle client connection without error', () => {
      expect(() => gateway.handleConnection(mockSocket as any)).not.toThrow();
    });
  });

  describe('handleDisconnect', () => {
    it('should handle client disconnection without error', () => {
      expect(() => gateway.handleDisconnect(mockSocket as any)).not.toThrow();
    });

    it('should remove peer from map and notify others', () => {
      // Add a mock peer
      (gateway as any).peers.set('peer-id', {
        socketId: 'test-socket-id',
        nodeId: 'peer-id',
        nodeType: MeshNodeType.FULL,
        publicKey: 'test-key',
        connectedAt: new Date(),
        lastSeen: new Date(),
      });

      gateway.handleDisconnect(mockSocket as any);
      expect((gateway as any).peers.has('peer-id')).toBe(false);
      expect(mockServer.emit).toHaveBeenCalledWith('peer:disconnected', { nodeId: 'peer-id' });
    });
  });

  describe('handleHandshake', () => {
    it('should process handshake and add peer', async () => {
      const payload = {
        nodeId: 'remote-node-id',
        nodeType: MeshNodeType.FULL,
        publicKey: 'remote-public-key',
      };

      const result = await gateway.handleHandshake(mockSocket as any, payload as any);

      expect((gateway as any).peers.has('remote-node-id')).toBe(true);
      expect(result.success).toBe(true);
      expect(result.nodeId).toBe((gateway as any).nodeId);
    });
  });

  describe('handleTrustUpdate', () => {
    it('should process trust update and broadcast', async () => {
      const message = {
        fromNode: 'peer-node',
        payload: {
          agentId: 'agent-1',
          principalId: 'principal-1',
          trustScore: 750,
          event: 'VERIFICATION_SUCCESS',
        },
      };

      const result = await gateway.handleTrustUpdate(mockSocket as any, message as any);

      expect(result.acknowledged).toBe(true);
      expect(mockSocket.broadcast.emit).toHaveBeenCalled();
    });
  });

  describe('handleRevocation', () => {
    it('should process revocation and broadcast', async () => {
      const message = {
        fromNode: 'peer-node',
        payload: {
          agentId: 'agent-1',
          principalId: 'principal-1',
          reason: 'Compromised',
        },
      };

      const result = await gateway.handleRevocation(mockSocket as any, message as any);

      expect(result.acknowledged).toBe(true);
      expect(mockServer.emit).toHaveBeenCalledWith('revocation', expect.anything());
    });
  });

  describe('handlePing', () => {
    it('should respond with pong', () => {
      const result = gateway.handlePing(mockSocket as any, { nodeId: 'peer-node' });

      expect(result.pong).toBe(true);
      expect(result.timestamp).toBeDefined();
    });
  });

  describe('handleListPeers', () => {
    it('should return list of connected peers', () => {
      // Add some mock peers
      (gateway as any).peers.set('peer-1', {
        nodeId: 'peer-1',
        nodeType: MeshNodeType.FULL,
      });
      (gateway as any).peers.set('peer-2', {
        nodeId: 'peer-2',
        nodeType: MeshNodeType.LIGHT,
      });

      const result = gateway.handleListPeers();

      expect(result.peers.length).toBe(2);
    });
  });

  describe('broadcastTrustUpdate', () => {
    it('should broadcast trust update to all peers', async () => {
      await gateway.broadcastTrustUpdate('agent-1', 'principal-1', 750, 'VERIFICATION_SUCCESS');

      expect(mockServer.emit).toHaveBeenCalledWith('trust:update', expect.objectContaining({
        type: MeshMessageType.TRUST_UPDATE,
      }));
    });
  });

  describe('broadcastRevocation', () => {
    it('should broadcast revocation to all peers', async () => {
      await gateway.broadcastRevocation('agent-1', 'principal-1', 'Compromised', 'admin');

      expect(mockServer.emit).toHaveBeenCalledWith('revocation', expect.objectContaining({
        type: MeshMessageType.REVOCATION,
      }));
    });
  });

  describe('getStats', () => {
    it('should return gateway statistics', () => {
      const stats = gateway.getStats();

      expect(stats.nodeId).toBe((gateway as any).nodeId);
      expect(stats.connectedPeers).toBe(0);
      expect(stats.uptime).toBeDefined();
    });
  });

  describe('peers management', () => {
    it('should start with empty peers map', () => {
      const freshGateway = new MeshGateway();
      expect((freshGateway as any).peers.size).toBe(0);
    });
  });
});
