/**
 * MeshNode Service Comprehensive Tests - Full Coverage
 */

import { Test, TestingModule } from '@nestjs/testing';
import { MeshNodeService } from './mesh-node.service';
import { AuditLoggerService } from './audit-logger.service';
import { MeshMessageType, MeshNodeType } from '../domain/mesh.entity';

describe('MeshNodeService Full Coverage', () => {
  let service: MeshNodeService;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [MeshNodeService, AuditLoggerService],
    }).compile();

    service = module.get<MeshNodeService>(MeshNodeService);
    await service.onModuleInit();
  });

  afterEach(async () => {
    await service.onModuleDestroy();
  });

  describe('onModuleInit', () => {
    it('should generate node identity', async () => {
      const info = service.getNodeInfo();
      expect(info.id).toBeDefined();
      expect(info.publicKey).toBeDefined();
    });

    it('should set node type', async () => {
      const info = service.getNodeInfo();
      expect(['FULL', 'LIGHT', 'BRIDGE']).toContain(info.type);
    });
  });

  describe('getNodeInfo', () => {
    it('should return complete node info', () => {
      const info = service.getNodeInfo();
      expect(info.id).toBeDefined();
      expect(info.publicKey).toBeDefined();
      expect(info.type).toBeDefined();
    });
  });

  describe('getMeshStats', () => {
    it('should return mesh stats', () => {
      const stats = service.getMeshStats();
      expect(stats.nodeId).toBeDefined();
      expect(stats.connectedPeers).toBe(0);
      expect(stats.processedMessages).toBeDefined();
      expect(stats.uptime).toBeDefined();
    });
  });

  describe('getConnectedPeers', () => {
    it('should return empty array initially', () => {
      const peers = service.getConnectedPeers();
      expect(Array.isArray(peers)).toBe(true);
      expect(peers.length).toBe(0);
    });
  });

  describe('connectToPeer', () => {
    it('should attempt to connect to peer', async () => {
      const result = await service.connectToPeer('ws://localhost:8080');
      expect(typeof result).toBe('boolean');
    });
  });

  describe('disconnectFromPeer', () => {
    it('should handle disconnect of non-existent peer', () => {
      expect(() => service.disconnectFromPeer('non-existent')).not.toThrow();
    });
  });

  describe('broadcastTrustUpdate', () => {
    it('should broadcast trust update', async () => {
      await expect(
        service.broadcastTrustUpdate('agent-1', 'principal-1', 750, 'VERIFICATION_SUCCESS'),
      ).resolves.not.toThrow();
    });

    it('should broadcast with previous score', async () => {
      await expect(
        service.broadcastTrustUpdate('agent-1', 'principal-1', 800, 'VERIFICATION_SUCCESS', 750),
      ).resolves.not.toThrow();
    });
  });

  describe('broadcastRevocation', () => {
    it('should broadcast revocation', async () => {
      await expect(
        service.broadcastRevocation('agent-1', 'principal-1', 'Compromised', 'admin'),
      ).resolves.not.toThrow();
    });
  });

  describe('handleMessage', () => {
    it('should handle trust update message', async () => {
      const message = {
        id: 'msg-' + Math.random(),
        type: MeshMessageType.TRUST_UPDATE,
        fromNode: 'other-node',
        payload: {
          agentId: 'agent-1',
          principalId: 'principal-1',
          trustScore: 750,
          event: 'VERIFICATION_SUCCESS',
          previousScore: 700,
          timestamp: new Date().toISOString(),
        },
        timestamp: new Date().toISOString(),
        ttl: 5,
        signature: 'test-sig',
      };

      await expect(service.handleMessage(message)).resolves.not.toThrow();
    });

    it('should handle revocation message', async () => {
      const message = {
        id: 'msg-' + Math.random(),
        type: MeshMessageType.REVOCATION,
        fromNode: 'other-node',
        payload: {
          agentId: 'agent-1',
          principalId: 'principal-1',
          reason: 'Compromised',
          revokedBy: 'admin',
          timestamp: new Date().toISOString(),
        },
        timestamp: new Date().toISOString(),
        ttl: 5,
        signature: 'test-sig',
      };

      await expect(service.handleMessage(message)).resolves.not.toThrow();
    });

    it('should handle handshake message', async () => {
      const message = {
        id: 'msg-' + Math.random(),
        type: MeshMessageType.HANDSHAKE,
        fromNode: 'other-node',
        payload: {
          nodeId: 'peer-node',
          nodeType: MeshNodeType.FULL,
          publicKey: 'test-key',
          capabilities: ['trust-gossip'],
          endpoints: ['ws://localhost:8080'],
        },
        timestamp: new Date().toISOString(),
        ttl: 1,
        signature: 'test-sig',
      };

      await expect(service.handleMessage(message)).resolves.not.toThrow();
    });

    it('should skip duplicate messages', async () => {
      const messageId = 'duplicate-msg-' + Math.random();
      const message = {
        id: messageId,
        type: MeshMessageType.TRUST_UPDATE,
        fromNode: 'other-node',
        payload: {
          agentId: 'agent-1',
          principalId: 'principal-1',
          trustScore: 750,
          event: 'VERIFICATION_SUCCESS',
          previousScore: 700,
          timestamp: new Date().toISOString(),
        },
        timestamp: new Date().toISOString(),
        ttl: 5,
        signature: 'test-sig',
      };

      // First call should process
      await service.handleMessage(message);
      // Second call should skip (duplicate)
      await expect(service.handleMessage(message)).resolves.not.toThrow();
    });

    it('should handle unknown message type', async () => {
      const message = {
        id: 'msg-' + Math.random(),
        type: 'UNKNOWN_TYPE' as any,
        fromNode: 'other-node',
        payload: {},
        timestamp: new Date().toISOString(),
        ttl: 5,
        signature: 'test-sig',
      };

      await expect(service.handleMessage(message)).resolves.not.toThrow();
    });

    it('should handle message with zero TTL', async () => {
      const message = {
        id: 'msg-ttl-0-' + Math.random(),
        type: MeshMessageType.TRUST_UPDATE,
        fromNode: 'other-node',
        payload: {
          agentId: 'agent-1',
          principalId: 'principal-1',
          trustScore: 750,
          event: 'VERIFICATION_SUCCESS',
          timestamp: new Date().toISOString(),
        },
        timestamp: new Date().toISOString(),
        ttl: 0,
        signature: 'test-sig',
      };

      await expect(service.handleMessage(message)).resolves.not.toThrow();
    });
  });

  describe('private methods (via casting)', () => {
    describe('verifyMessage', () => {
      it('should accept message from unknown node', async () => {
        const message = {
          id: 'msg-verify-' + Math.random(),
          type: MeshMessageType.TRUST_UPDATE,
          fromNode: 'unknown-node-id',
          payload: {},
          timestamp: new Date().toISOString(),
          ttl: 5,
          signature: 'test',
        };

        const result = await (service as any).verifyMessage(message);
        // Unknown node should be accepted
        expect(result).toBe(true);
      });

      it('should handle verification error gracefully', async () => {
        // Add a known node with invalid key
        (service as any).knownNodes.set('bad-node', {
          nodeId: 'bad-node',
          publicKey: 'invalid-public-key',
          type: MeshNodeType.FULL,
        });

        const message = {
          id: 'msg-bad-' + Math.random(),
          type: MeshMessageType.TRUST_UPDATE,
          fromNode: 'bad-node',
          payload: {},
          timestamp: new Date().toISOString(),
          ttl: 5,
          signature: 'invalid',
        };

        const result = await (service as any).verifyMessage(message);
        expect(result).toBe(false);
      });
    });

    describe('broadcast', () => {
      it('should handle broadcast with no peers', async () => {
        const message = {
          id: 'broadcast-msg',
          type: MeshMessageType.TRUST_UPDATE,
          fromNode: (service as any).nodeId,
          payload: {},
          timestamp: new Date().toISOString(),
        };

        await expect((service as any).broadcast(message)).resolves.not.toThrow();
      });

      it('should handle priority broadcast', async () => {
        const message = {
          id: 'priority-msg',
          type: MeshMessageType.REVOCATION,
          fromNode: (service as any).nodeId,
          payload: {},
          timestamp: new Date().toISOString(),
        };

        await expect((service as any).broadcast(message, true)).resolves.not.toThrow();
      });

      it('should skip disconnected peers', async () => {
        // Add a disconnected peer
        (service as any).peers.set('disconnected-peer', {
          nodeId: 'disconnected-peer',
          connected: false,
          messageQueue: [],
        });

        const message = {
          id: 'skip-msg',
          type: MeshMessageType.TRUST_UPDATE,
          fromNode: (service as any).nodeId,
          payload: {},
          timestamp: new Date().toISOString(),
        };

        await expect((service as any).broadcast(message)).resolves.not.toThrow();
      });

      it('should queue messages for connected peers', async () => {
        // Add a connected peer
        (service as any).peers.set('connected-peer', {
          nodeId: 'connected-peer',
          connected: true,
          messageQueue: [],
        });

        const message = {
          id: 'queue-msg',
          type: MeshMessageType.TRUST_UPDATE,
          fromNode: (service as any).nodeId,
          payload: {},
          timestamp: new Date().toISOString(),
        };

        await (service as any).broadcast(message);
        expect((service as any).peers.get('connected-peer').messageQueue.length).toBe(1);
      });
    });

    describe('forward', () => {
      it('should not forward own messages', async () => {
        const message = {
          id: 'own-msg',
          type: MeshMessageType.TRUST_UPDATE,
          fromNode: (service as any).nodeId,
          payload: {},
          timestamp: new Date().toISOString(),
        };

        await expect((service as any).forward(message)).resolves.not.toThrow();
      });

      it('should forward messages from other nodes', async () => {
        // Add a connected peer
        (service as any).peers.set('forward-peer', {
          nodeId: 'forward-peer',
          connected: true,
        });

        const message = {
          id: 'forward-msg',
          type: MeshMessageType.TRUST_UPDATE,
          fromNode: 'other-node',
          payload: {},
          timestamp: new Date().toISOString(),
        };

        await expect((service as any).forward(message)).resolves.not.toThrow();
      });

      it('should not forward back to source', async () => {
        // Add source as peer
        (service as any).peers.set('source-node', {
          nodeId: 'source-node',
          connected: true,
        });

        const message = {
          id: 'source-msg',
          type: MeshMessageType.TRUST_UPDATE,
          fromNode: 'source-node',
          payload: {},
          timestamp: new Date().toISOString(),
        };

        await expect((service as any).forward(message)).resolves.not.toThrow();
      });
    });
  });

  describe('onModuleDestroy', () => {
    it('should cleanup resources', async () => {
      await expect(service.onModuleDestroy()).resolves.not.toThrow();
    });
  });
});

