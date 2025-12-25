/**
 * Mesh Controller Unit Tests
 */

import { Test, TestingModule } from '@nestjs/testing';
import { MeshController } from './mesh.controller';
import { MeshNodeService } from '../services/mesh-node.service';
import { AuditLoggerService } from '../services/audit-logger.service';
import { MeshNodeType } from '../domain/mesh.entity';

describe('MeshController', () => {
  let controller: MeshController;
  let meshService: MeshNodeService;

  const mockNodeInfo = {
    id: 'node-1',
    publicKey: 'test-public-key',
    type: MeshNodeType.FULL,
    endpoints: ['ws://localhost:8080'],
    capabilities: ['trust-gossip'],
    trustScore: 500,
  };

  const mockStats = {
    nodeId: 'node-1',
    nodeType: MeshNodeType.FULL,
    connectedPeers: 5,
    processedMessages: 1000,
    uptime: 3600,
  };

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      controllers: [MeshController],
      providers: [
        {
          provide: MeshNodeService,
          useValue: {
            getNodeInfo: jest.fn().mockReturnValue(mockNodeInfo),
            getMeshStats: jest.fn().mockReturnValue(mockStats),
            getConnectedPeers: jest.fn().mockReturnValue([mockNodeInfo]),
            connectToPeer: jest.fn().mockResolvedValue(true),
            disconnectFromPeer: jest.fn(),
            broadcastTrustUpdate: jest.fn().mockResolvedValue(undefined),
            broadcastRevocation: jest.fn().mockResolvedValue(undefined),
          },
        },
        AuditLoggerService,
      ],
    }).compile();

    controller = module.get<MeshController>(MeshController);
    meshService = module.get<MeshNodeService>(MeshNodeService);
  });

  describe('getNodeInfo', () => {
    it('should return local node info', () => {
      const result = controller.getNodeInfo();
      expect(result.id).toBe('node-1');
      expect(result.type).toBe(MeshNodeType.FULL);
    });
  });

  describe('getMeshStats', () => {
    it('should return mesh statistics', () => {
      const result = controller.getMeshStats();
      expect(result.nodeId).toBe('node-1');
      expect(result.connectedPeers).toBe(5);
      expect(result.processedMessages).toBe(1000);
    });
  });

  describe('getConnectedPeers', () => {
    it('should return list of connected peers', () => {
      const result = controller.getConnectedPeers();
      expect(Array.isArray(result)).toBe(true);
      expect(result.length).toBe(1);
    });
  });

  describe('connectToPeer', () => {
    it('should connect to peer successfully', async () => {
      const result = await controller.connectToPeer({
        endpoint: 'ws://peer:8080',
      });

      expect(result.success).toBe(true);
      expect(result.message).toContain('Connected');
    });

    it('should return failure message when connection fails', async () => {
      jest.spyOn(meshService, 'connectToPeer').mockResolvedValue(false);

      const result = await controller.connectToPeer({
        endpoint: 'ws://invalid:8080',
      });

      expect(result.success).toBe(false);
      expect(result.message).toContain('Failed');
    });
  });

  describe('disconnectFromPeer', () => {
    it('should disconnect from peer', () => {
      const result = controller.disconnectFromPeer('peer-id');
      expect(result.success).toBe(true);
      expect(meshService.disconnectFromPeer).toHaveBeenCalledWith('peer-id');
    });
  });

  describe('broadcastTrustUpdate', () => {
    it('should broadcast trust update', async () => {
      const result = await controller.broadcastTrustUpdate({
        agentId: 'agent-1',
        principalId: 'principal-1',
        trustScore: 750,
        event: 'VERIFICATION_SUCCESS',
      });

      expect(result.success).toBe(true);
      expect(meshService.broadcastTrustUpdate).toHaveBeenCalled();
    });
  });

  describe('broadcastRevocation', () => {
    it('should broadcast revocation', async () => {
      const result = await controller.broadcastRevocation({
        agentId: 'agent-1',
        principalId: 'principal-1',
        reason: 'Compromised',
      });

      expect(result.success).toBe(true);
      expect(meshService.broadcastRevocation).toHaveBeenCalled();
    });
  });
});
