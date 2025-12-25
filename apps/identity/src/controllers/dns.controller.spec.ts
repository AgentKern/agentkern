/**
 * DNS Controller Unit Tests
 */

import { Test, TestingModule } from '@nestjs/testing';
import { DnsController } from './dns.controller';
import { DnsResolutionService } from '../services/dns-resolution.service';
import { AuditLoggerService } from '../services/audit-logger.service';

describe('DnsController', () => {
  let controller: DnsController;
  let dnsService: DnsResolutionService;

  const mockTrustRecord = {
    id: 'record-1',
    agentId: 'agent-1',
    principalId: 'principal-1',
    trustScore: 750,
    trusted: true,
    revoked: false,
    registeredAt: new Date().toISOString(),
    lastVerifiedAt: new Date().toISOString(),
    verificationCount: 10,
    failureCount: 0,
  };

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      controllers: [DnsController],
      providers: [
        {
          provide: DnsResolutionService,
          useValue: {
            resolve: jest.fn().mockResolvedValue({
              trusted: true,
              trustScore: 750,
              revoked: false,
              ttl: 300,
            }),
            resolveBatch: jest.fn().mockResolvedValue([]),
            registerTrust: jest.fn().mockReturnValue(mockTrustRecord),
            revokeTrust: jest.fn().mockReturnValue(mockTrustRecord),
            reinstateTrust: jest.fn().mockReturnValue(mockTrustRecord),
            getTrustRecordsForPrincipal: jest.fn().mockReturnValue([mockTrustRecord]),
            getTrustRecord: jest.fn().mockReturnValue(mockTrustRecord),
          },
        },
        AuditLoggerService,
      ],
    }).compile();

    controller = module.get<DnsController>(DnsController);
    dnsService = module.get<DnsResolutionService>(DnsResolutionService);
  });

  describe('resolve', () => {
    it('should resolve trust for agent-principal pair', async () => {
      const result = await controller.resolve('agent-1', 'principal-1');
      expect(result.trusted).toBe(true);
      expect(result.trustScore).toBe(750);
    });
  });

  describe('resolveBatch', () => {
    it('should batch resolve queries', async () => {
      jest.spyOn(dnsService, 'resolveBatch').mockResolvedValue([
        { trusted: true, trustScore: 750, revoked: false, ttl: 300, agentId: 'a1', principalId: 'p1' },
        { trusted: true, trustScore: 800, revoked: false, ttl: 300, agentId: 'a2', principalId: 'p1' },
      ]);

      const result = await controller.resolveBatch({
        queries: [
          { agentId: 'a1', principalId: 'p1' },
          { agentId: 'a2', principalId: 'p1' },
        ],
      });

      expect(result.length).toBe(2);
    });
  });

  describe('registerTrust', () => {
    it('should register new trust relationship', async () => {
      const result = await controller.registerTrust({
        agentId: 'agent-1',
        principalId: 'principal-1',
      });

      expect(result.agentId).toBe('agent-1');
      expect(result.trusted).toBe(true);
    });
  });

  describe('revokeTrust', () => {
    it('should revoke trust', async () => {
      const result = await controller.revokeTrust({
        agentId: 'agent-1',
        principalId: 'principal-1',
        reason: 'Compromised',
      });

      expect(result).toBeDefined();
      expect(dnsService.revokeTrust).toHaveBeenCalled();
    });

    it('should throw NotFoundException for non-existent record', async () => {
      jest.spyOn(dnsService, 'revokeTrust').mockReturnValue(null);

      await expect(controller.revokeTrust({
        agentId: 'unknown',
        principalId: 'unknown',
        reason: 'test',
      })).rejects.toThrow();
    });
  });

  describe('reinstateTrust', () => {
    it('should reinstate revoked trust', async () => {
      const result = await controller.reinstateTrust({
        agentId: 'agent-1',
        principalId: 'principal-1',
      });

      expect(result).toBeDefined();
    });
  });

  describe('getRecordsForPrincipal', () => {
    it('should return all trust records for principal', async () => {
      const result = await controller.getRecordsForPrincipal('principal-1');
      expect(Array.isArray(result)).toBe(true);
      expect(result.length).toBeGreaterThan(0);
    });
  });

  describe('getRecord', () => {
    it('should return specific trust record', async () => {
      const result = await controller.getRecord('agent-1', 'principal-1');
      expect(result.agentId).toBe('agent-1');
    });

    it('should throw NotFoundException for non-existent record', async () => {
      jest.spyOn(dnsService, 'getTrustRecord').mockReturnValue(null);

      await expect(controller.getRecord('unknown', 'unknown')).rejects.toThrow();
    });
  });
});
