import { Test, TestingModule } from '@nestjs/testing';
import { getRepositoryToken } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import { TrustService, TrustLevel, TrustScore } from './trust.service';
import {
  TrustScoreEntity,
  TrustEventEntity,
  TrustEventType as TrustEventTypeEnum,
} from '../entities/trust-event.entity';

// ============================================================================
// MOCKS
// ============================================================================

const createMockRepository = () => ({
  findOne: jest.fn(),
  find: jest.fn(),
  create: jest.fn(),
  save: jest.fn(),
  update: jest.fn(),
  delete: jest.fn(),
  count: jest.fn(),
  createQueryBuilder: jest.fn(() => ({
    where: jest.fn().mockReturnThis(),
    andWhere: jest.fn().mockReturnThis(),
    orderBy: jest.fn().mockReturnThis(),
    take: jest.fn().mockReturnThis(),
    skip: jest.fn().mockReturnThis(),
    getMany: jest.fn().mockResolvedValue([]),
    getOne: jest.fn().mockResolvedValue(null),
  })),
});

const mockScoreEntity: Partial<TrustScoreEntity> = {
  id: 'score-1',
  agentId: 'agent-test-123',
  score: 75,
  level: 'high',
  transactionSuccessRate: 95,
  averageResponseTimeMs: 150,
  policyComplianceRate: 100,
  peerEndorsementCount: 5,
  accountAgeDays: 30,
  verifiedCredentialCount: 3,
  totalTransactions: 100,
  failedTransactions: 5,
  createdAt: new Date('2025-12-01'),
  calculatedAt: new Date('2026-01-01'),
};

const mockEventEntity: Partial<TrustEventEntity> = {
  id: 'event-1',
  agentId: 'agent-test-123',
  type: TrustEventTypeEnum.TRANSACTION_SUCCESS,
  delta: 2,
  reason: 'Successful transaction',
  timestamp: new Date(),
};

// ============================================================================
// TEST SUITE
// ============================================================================

describe('TrustService', () => {
  let service: TrustService;
  let scoreRepository: jest.Mocked<Repository<TrustScoreEntity>>;
  let eventRepository: jest.Mocked<Repository<TrustEventEntity>>;

  beforeEach(async () => {
    const mockScoreRepo = createMockRepository();
    const mockEventRepo = createMockRepository();

    const module: TestingModule = await Test.createTestingModule({
      providers: [
        TrustService,
        {
          provide: getRepositoryToken(TrustScoreEntity),
          useValue: mockScoreRepo,
        },
        {
          provide: getRepositoryToken(TrustEventEntity),
          useValue: mockEventRepo,
        },
      ],
    }).compile();

    service = module.get<TrustService>(TrustService);
    scoreRepository = module.get(getRepositoryToken(TrustScoreEntity));
    eventRepository = module.get(getRepositoryToken(TrustEventEntity));
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  // =========================================================================
  // INITIALIZATION
  // =========================================================================

  describe('initialization', () => {
    it('should be defined', () => {
      expect(service).toBeDefined();
    });
  });

  // =========================================================================
  // getTrustScore
  // =========================================================================

  describe('getTrustScore', () => {
    it('should return existing trust score', async () => {
      scoreRepository.findOne.mockResolvedValue(mockScoreEntity as TrustScoreEntity);
      eventRepository.find.mockResolvedValue([mockEventEntity as TrustEventEntity]);

      const result = await service.getTrustScore('agent-test-123');

      expect(result).toBeDefined();
      expect(result.agentId).toBe('agent-test-123');
      expect(result.score).toBe(75);
      expect(result.level).toBe(TrustLevel.HIGH);
      expect(scoreRepository.findOne).toHaveBeenCalledWith({
        where: { agentId: 'agent-test-123' },
      });
    });

    it('should initialize trust score for new agent', async () => {
      const newAgentId = 'new-agent-456';
      const newScoreEntity = {
        ...mockScoreEntity,
        id: 'score-new',
        agentId: newAgentId,
        score: 50,
        level: 'medium',
      };

      scoreRepository.findOne.mockResolvedValue(null);
      scoreRepository.create.mockReturnValue(newScoreEntity as TrustScoreEntity);
      scoreRepository.save.mockResolvedValue(newScoreEntity as TrustScoreEntity);
      eventRepository.create.mockReturnValue({
        agentId: newAgentId,
        type: TrustEventTypeEnum.REGISTRATION,
        delta: 0,
        reason: 'Agent registered',
      } as TrustEventEntity);
      eventRepository.save.mockResolvedValue({} as TrustEventEntity);
      eventRepository.find.mockResolvedValue([]);

      const result = await service.getTrustScore(newAgentId);

      expect(result).toBeDefined();
      expect(result.agentId).toBe(newAgentId);
      expect(scoreRepository.create).toHaveBeenCalled();
      expect(scoreRepository.save).toHaveBeenCalled();
    });

    it('should include recent events in history', async () => {
      const events = [
        { ...mockEventEntity, id: 'event-1' },
        { ...mockEventEntity, id: 'event-2' },
        { ...mockEventEntity, id: 'event-3' },
      ];

      scoreRepository.findOne.mockResolvedValue(mockScoreEntity as TrustScoreEntity);
      eventRepository.find.mockResolvedValue(events as TrustEventEntity[]);

      const result = await service.getTrustScore('agent-test-123');

      expect(result.history).toHaveLength(3);
      expect(eventRepository.find).toHaveBeenCalledWith({
        where: { agentId: 'agent-test-123' },
        order: { timestamp: 'DESC' },
        take: 10,
      });
    });
  });

  // =========================================================================
  // recordEvent
  // =========================================================================

  describe('recordEvent', () => {
    beforeEach(() => {
      scoreRepository.findOne.mockResolvedValue(mockScoreEntity as TrustScoreEntity);
      scoreRepository.save.mockResolvedValue(mockScoreEntity as TrustScoreEntity);
      eventRepository.create.mockReturnValue(mockEventEntity as TrustEventEntity);
      eventRepository.save.mockResolvedValue(mockEventEntity as TrustEventEntity);
      eventRepository.find.mockResolvedValue([mockEventEntity as TrustEventEntity]);
    });

    it('should save event and recalculate score', async () => {
      const result = await service.recordEvent('agent-test-123', {
        type: TrustEventTypeEnum.TRANSACTION_SUCCESS,
        delta: 2,
        reason: 'Completed task',
      });

      expect(result).toBeDefined();
      expect(eventRepository.create).toHaveBeenCalled();
      expect(eventRepository.save).toHaveBeenCalled();
    });

    it('should handle events with related agent', async () => {
      await service.recordEvent('agent-test-123', {
        type: TrustEventTypeEnum.PEER_ENDORSEMENT,
        delta: 3,
        reason: 'Endorsed by peer',
        relatedAgentId: 'endorser-agent',
      });

      expect(eventRepository.create).toHaveBeenCalledWith(
        expect.objectContaining({
          relatedAgentId: 'endorser-agent',
        }),
      );
    });

    it('should handle events with response time', async () => {
      await service.recordEvent('agent-test-123', {
        type: TrustEventTypeEnum.TRANSACTION_SUCCESS,
        delta: 2,
        reason: 'Fast completion',
        responseTimeMs: 150,
      });

      expect(eventRepository.create).toHaveBeenCalledWith(
        expect.objectContaining({
          responseTimeMs: 150,
        }),
      );
    });
  });

  // =========================================================================
  // recordTransactionSuccess
  // =========================================================================

  describe('recordTransactionSuccess', () => {
    beforeEach(() => {
      scoreRepository.findOne.mockResolvedValue(mockScoreEntity as TrustScoreEntity);
      scoreRepository.save.mockResolvedValue(mockScoreEntity as TrustScoreEntity);
      eventRepository.create.mockReturnValue(mockEventEntity as TrustEventEntity);
      eventRepository.save.mockResolvedValue(mockEventEntity as TrustEventEntity);
      eventRepository.find.mockResolvedValue([]);
    });

    it('should record success event with response time', async () => {
      const result = await service.recordTransactionSuccess(
        'agent-test-123',
        'counterparty-456',
        200,
      );

      expect(result).toBeDefined();
      expect(eventRepository.create).toHaveBeenCalledWith(
        expect.objectContaining({
          type: TrustEventTypeEnum.TRANSACTION_SUCCESS,
          relatedAgentId: 'counterparty-456',
          responseTimeMs: 200,
        }),
      );
    });

    it('should work without optional parameters', async () => {
      const result = await service.recordTransactionSuccess('agent-test-123');

      expect(result).toBeDefined();
      expect(eventRepository.create).toHaveBeenCalled();
    });
  });

  // =========================================================================
  // recordTransactionFailure
  // =========================================================================

  describe('recordTransactionFailure', () => {
    beforeEach(() => {
      scoreRepository.findOne.mockResolvedValue(mockScoreEntity as TrustScoreEntity);
      scoreRepository.save.mockResolvedValue(mockScoreEntity as TrustScoreEntity);
      eventRepository.create.mockReturnValue({
        ...mockEventEntity,
        type: TrustEventTypeEnum.TRANSACTION_FAILURE,
      } as TrustEventEntity);
      eventRepository.save.mockResolvedValue({} as TrustEventEntity);
      eventRepository.find.mockResolvedValue([]);
    });

    it('should record failure event with reason', async () => {
      const result = await service.recordTransactionFailure(
        'agent-test-123',
        'Timeout exceeded',
        'counterparty-456',
      );

      expect(result).toBeDefined();
      expect(eventRepository.create).toHaveBeenCalledWith(
        expect.objectContaining({
          type: TrustEventTypeEnum.TRANSACTION_FAILURE,
          reason: expect.stringContaining('Timeout exceeded'),
        }),
      );
    });
  });

  // =========================================================================
  // recordPolicyViolation
  // =========================================================================

  describe('recordPolicyViolation', () => {
    beforeEach(() => {
      scoreRepository.findOne.mockResolvedValue(mockScoreEntity as TrustScoreEntity);
      scoreRepository.save.mockResolvedValue(mockScoreEntity as TrustScoreEntity);
      eventRepository.create.mockReturnValue({
        ...mockEventEntity,
        type: TrustEventTypeEnum.POLICY_VIOLATION,
      } as TrustEventEntity);
      eventRepository.save.mockResolvedValue({} as TrustEventEntity);
      eventRepository.find.mockResolvedValue([]);
    });

    it('should record policy violation with negative delta', async () => {
      const result = await service.recordPolicyViolation('agent-test-123', 'policy-rate-limit');

      expect(result).toBeDefined();
      expect(eventRepository.create).toHaveBeenCalledWith(
        expect.objectContaining({
          type: TrustEventTypeEnum.POLICY_VIOLATION,
        }),
      );
    });
  });

  // =========================================================================
  // recordPeerEndorsement
  // =========================================================================

  describe('recordPeerEndorsement', () => {
    beforeEach(() => {
      scoreRepository.findOne.mockResolvedValue(mockScoreEntity as TrustScoreEntity);
      scoreRepository.save.mockResolvedValue(mockScoreEntity as TrustScoreEntity);
      eventRepository.create.mockReturnValue({
        ...mockEventEntity,
        type: TrustEventTypeEnum.PEER_ENDORSEMENT,
      } as TrustEventEntity);
      eventRepository.save.mockResolvedValue({} as TrustEventEntity);
      eventRepository.find.mockResolvedValue([]);
    });

    it('should record endorsement with endorser ID', async () => {
      const result = await service.recordPeerEndorsement('agent-test-123', 'endorser-agent');

      expect(result).toBeDefined();
      expect(eventRepository.create).toHaveBeenCalledWith(
        expect.objectContaining({
          type: TrustEventTypeEnum.PEER_ENDORSEMENT,
          relatedAgentId: 'endorser-agent',
        }),
      );
    });
  });

  // =========================================================================
  // recordCredentialVerified
  // =========================================================================

  describe('recordCredentialVerified', () => {
    beforeEach(() => {
      scoreRepository.findOne.mockResolvedValue(mockScoreEntity as TrustScoreEntity);
      scoreRepository.save.mockResolvedValue(mockScoreEntity as TrustScoreEntity);
      eventRepository.create.mockReturnValue({
        ...mockEventEntity,
        type: TrustEventTypeEnum.CREDENTIAL_VERIFIED,
      } as TrustEventEntity);
      eventRepository.save.mockResolvedValue({} as TrustEventEntity);
      eventRepository.find.mockResolvedValue([]);
    });

    it('should record credential verification', async () => {
      const result = await service.recordCredentialVerified('agent-test-123', 'TrustCredential');

      expect(result).toBeDefined();
      expect(eventRepository.create).toHaveBeenCalledWith(
        expect.objectContaining({
          type: TrustEventTypeEnum.CREDENTIAL_VERIFIED,
          reason: expect.stringContaining('TrustCredential'),
        }),
      );
    });
  });

  // =========================================================================
  // TrustLevel mapping
  // =========================================================================

  describe('trust level mapping', () => {
    it('should map low scores to UNTRUSTED', async () => {
      scoreRepository.findOne.mockResolvedValue({
        ...mockScoreEntity,
        score: 15,
        level: 'untrusted',
      } as TrustScoreEntity);
      eventRepository.find.mockResolvedValue([]);

      const result = await service.getTrustScore('agent-test-123');
      expect(result.level).toBe(TrustLevel.UNTRUSTED);
    });

    it('should map medium scores to MEDIUM', async () => {
      scoreRepository.findOne.mockResolvedValue({
        ...mockScoreEntity,
        score: 55,
        level: 'medium',
      } as TrustScoreEntity);
      eventRepository.find.mockResolvedValue([]);

      const result = await service.getTrustScore('agent-test-123');
      expect(result.level).toBe(TrustLevel.MEDIUM);
    });

    it('should map high scores to VERIFIED', async () => {
      scoreRepository.findOne.mockResolvedValue({
        ...mockScoreEntity,
        score: 95,
        level: 'verified',
      } as TrustScoreEntity);
      eventRepository.find.mockResolvedValue([]);

      const result = await service.getTrustScore('agent-test-123');
      expect(result.level).toBe(TrustLevel.VERIFIED);
    });
  });

  // =========================================================================
  // Edge cases
  // =========================================================================

  describe('edge cases', () => {
    it('should handle empty agent ID gracefully', async () => {
      scoreRepository.findOne.mockResolvedValue(null);
      scoreRepository.create.mockReturnValue({
        agentId: '',
        score: 50,
      } as TrustScoreEntity);
      scoreRepository.save.mockResolvedValue({
        agentId: '',
        score: 50,
      } as TrustScoreEntity);
      eventRepository.create.mockReturnValue({} as TrustEventEntity);
      eventRepository.save.mockResolvedValue({} as TrustEventEntity);
      eventRepository.find.mockResolvedValue([]);

      const result = await service.getTrustScore('');
      expect(result).toBeDefined();
    });

    it('should handle special characters in agent ID', async () => {
      const specialAgentId = 'agent-with-special-chars!@#$%';
      
      scoreRepository.findOne.mockResolvedValue({
        ...mockScoreEntity,
        agentId: specialAgentId,
      } as TrustScoreEntity);
      eventRepository.find.mockResolvedValue([]);

      const result = await service.getTrustScore(specialAgentId);
      expect(result.agentId).toBe(specialAgentId);
    });
  });

  // =========================================================================
  // Mutual Authentication
  // =========================================================================

  describe('initiateMutualAuth', () => {
    it('should create mutual auth request with challenge', () => {
      const result = service.initiateMutualAuth('requester-123', 'target-456');

      expect(result).toBeDefined();
      expect(result.requesterId).toBe('requester-123');
      expect(result.targetId).toBe('target-456');
      expect(result.challenge).toBeDefined();
      expect(result.challenge.length).toBeGreaterThan(0);
      expect(result.timestamp).toBeInstanceOf(Date);
    });

    it('should generate unique challenges for each request', () => {
      const result1 = service.initiateMutualAuth('req1', 'tgt1');
      const result2 = service.initiateMutualAuth('req2', 'tgt2');

      expect(result1.challenge).not.toBe(result2.challenge);
    });
  });

  describe('completeMutualAuth', () => {
    beforeEach(() => {
      scoreRepository.findOne.mockResolvedValue(mockScoreEntity as TrustScoreEntity);
      scoreRepository.save.mockResolvedValue(mockScoreEntity as TrustScoreEntity);
      eventRepository.create.mockReturnValue(mockEventEntity as TrustEventEntity);
      eventRepository.save.mockResolvedValue(mockEventEntity as TrustEventEntity);
      eventRepository.find.mockResolvedValue([]);
    });

    it('should complete auth and return mutual trust score', async () => {
      const request = service.initiateMutualAuth('requester-123', 'target-456');
      
      // Use valid proofs (matching the HMAC logic)
      const result = await service.completeMutualAuth(
        request,
        'valid-requester-proof',
        'valid-target-proof',
      );

      expect(result).toBeDefined();
      expect(result.requesterScore).toBeDefined();
      expect(result.targetScore).toBeDefined();
      expect(typeof result.mutualTrust).toBe('number');
    });

    it('should return verified false for invalid proofs', async () => {
      const request = service.initiateMutualAuth('requester-123', 'target-456');
      
      const result = await service.completeMutualAuth(
        request,
        '',
        '',
      );

      // With empty proofs, verification should fail
      expect(result.verified).toBe(false);
      expect(result.mutualTrust).toBe(0);
    });
  });

  // =========================================================================
  // Recalculate Trust Score
  // =========================================================================

  describe('recalculateTrustScore', () => {
    beforeEach(() => {
      scoreRepository.findOne.mockResolvedValue(mockScoreEntity as TrustScoreEntity);
      scoreRepository.save.mockResolvedValue(mockScoreEntity as TrustScoreEntity);
      eventRepository.find.mockResolvedValue([]);
    });

    it('should recalculate score based on events', async () => {
      const events = [
        {
          ...mockEventEntity,
          type: TrustEventTypeEnum.TRANSACTION_SUCCESS,
          responseTimeMs: 100,
        },
        {
          ...mockEventEntity,
          type: TrustEventTypeEnum.TRANSACTION_SUCCESS,
          responseTimeMs: 200,
        },
      ];
      eventRepository.find.mockResolvedValue(events as TrustEventEntity[]);

      const result = await (service as any).recalculateTrustScore('agent-test-123');

      expect(result).toBeDefined();
      expect(scoreRepository.save).toHaveBeenCalled();
    });

    it('should handle agent with no events', async () => {
      eventRepository.find.mockResolvedValue([]);

      const result = await (service as any).recalculateTrustScore('agent-test-123');

      expect(result).toBeDefined();
    });

    it('should update score based on transaction success rate', async () => {
      const events = [
        { ...mockEventEntity, type: TrustEventTypeEnum.TRANSACTION_SUCCESS },
        { ...mockEventEntity, type: TrustEventTypeEnum.TRANSACTION_SUCCESS },
        { ...mockEventEntity, type: TrustEventTypeEnum.TRANSACTION_FAILURE },
      ];
      eventRepository.find.mockResolvedValue(events as TrustEventEntity[]);

      const result = await (service as any).recalculateTrustScore('agent-test-123');

      expect(result).toBeDefined();
      expect(result.factors.transactionSuccess).toBeDefined();
    });
  });

  // =========================================================================
  // Verifiable Credentials
  // =========================================================================

  describe('issueCredential', () => {
    beforeEach(() => {
      scoreRepository.findOne.mockResolvedValue(mockScoreEntity as TrustScoreEntity);
      scoreRepository.save.mockResolvedValue(mockScoreEntity as TrustScoreEntity);
      eventRepository.create.mockReturnValue(mockEventEntity as TrustEventEntity);
      eventRepository.save.mockResolvedValue(mockEventEntity as TrustEventEntity);
      eventRepository.find.mockResolvedValue([]);
    });

    it('should issue W3C verifiable credential', async () => {
      // Initialize signing key first
      await service.onModuleInit();

      const result = await service.issueCredential('agent-test-123');

      expect(result).toBeDefined();
      expect(result['@context']).toContain('https://www.w3.org/2018/credentials/v1');
      expect(result.type).toContain('VerifiableCredential');
      expect(result.credentialSubject.id).toContain('agent-test-123');
    });

    it('should include trust score in credential subject', async () => {
      await service.onModuleInit();

      const result = await service.issueCredential('agent-test-123', 'TrustScoreCredential');

      expect(result.credentialSubject.trustScore).toBeDefined();
      expect(typeof result.credentialSubject.trustScore).toBe('number');
      expect(result.type).toContain('TrustScoreCredential');
    });

    it('should include proof with JWS signature', async () => {
      await service.onModuleInit();

      const result = await service.issueCredential('agent-test-123');

      expect(result.proof).toBeDefined();
      expect(result.proof?.type).toBe('Ed25519Signature2020');
      expect(result.proof?.jws).toBeDefined();
    });
  });

  describe('verifyCredential', () => {
    beforeEach(() => {
      scoreRepository.findOne.mockResolvedValue(mockScoreEntity as TrustScoreEntity);
      scoreRepository.save.mockResolvedValue(mockScoreEntity as TrustScoreEntity);
      eventRepository.create.mockReturnValue(mockEventEntity as TrustEventEntity);
      eventRepository.save.mockResolvedValue(mockEventEntity as TrustEventEntity);
      eventRepository.find.mockResolvedValue([]);
    });

    it('should verify valid credential', async () => {
      await service.onModuleInit();
      const credential = await service.issueCredential('agent-test-123');

      const isValid = await service.verifyCredential(credential);

      expect(isValid).toBe(true);
    });

    it('should reject credential without context', async () => {
      await service.onModuleInit();

      const invalidCredential = {
        id: 'test',
        type: ['VerifiableCredential'],
        issuer: 'test',
        issuanceDate: new Date().toISOString(),
        credentialSubject: { id: 'test' },
      } as any;

      const isValid = await service.verifyCredential(invalidCredential);

      expect(isValid).toBe(false);
    });

    it('should reject expired credentials', async () => {
      await service.onModuleInit();

      const expiredCredential = {
        '@context': ['https://www.w3.org/2018/credentials/v1'],
        id: 'test',
        type: ['VerifiableCredential'],
        issuer: 'test',
        // 60 days ago (credentials valid for 30 days)
        issuanceDate: new Date(Date.now() - 60 * 24 * 60 * 60 * 1000).toISOString(),
        credentialSubject: { id: 'test' },
        proof: { jws: 'test' },
      } as any;

      const isValid = await service.verifyCredential(expiredCredential);

      expect(isValid).toBe(false);
    });

    it('should reject credential without JWS proof', async () => {
      await service.onModuleInit();

      const noProofCredential = {
        '@context': ['https://www.w3.org/2018/credentials/v1'],
        id: 'test',
        type: ['VerifiableCredential'],
        issuer: 'test',
        issuanceDate: new Date().toISOString(),
        credentialSubject: { id: 'test' },
      } as any;

      const isValid = await service.verifyCredential(noProofCredential);

      expect(isValid).toBe(false);
    });
  });

  // =========================================================================
  // Trust Level Boundaries
  // =========================================================================

  describe('trust level boundaries', () => {
    it('should map score 0 to UNTRUSTED', async () => {
      scoreRepository.findOne.mockResolvedValue({
        ...mockScoreEntity,
        score: 0,
        level: 'untrusted',
      } as TrustScoreEntity);
      eventRepository.find.mockResolvedValue([]);

      const result = await service.getTrustScore('agent-test-123');
      expect(result.level).toBe(TrustLevel.UNTRUSTED);
    });

    it('should map score 20 to UNTRUSTED', async () => {
      scoreRepository.findOne.mockResolvedValue({
        ...mockScoreEntity,
        score: 20,
        level: 'untrusted',
      } as TrustScoreEntity);
      eventRepository.find.mockResolvedValue([]);

      const result = await service.getTrustScore('agent-test-123');
      expect(result.level).toBe(TrustLevel.UNTRUSTED);
    });

    it('should map score 21 to LOW', async () => {
      scoreRepository.findOne.mockResolvedValue({
        ...mockScoreEntity,
        score: 21,
        level: 'low',
      } as TrustScoreEntity);
      eventRepository.find.mockResolvedValue([]);

      const result = await service.getTrustScore('agent-test-123');
      expect(result.level).toBe(TrustLevel.LOW);
    });

    it('should map score 100 to VERIFIED', async () => {
      scoreRepository.findOne.mockResolvedValue({
        ...mockScoreEntity,
        score: 100,
        level: 'verified',
      } as TrustScoreEntity);
      eventRepository.find.mockResolvedValue([]);

      const result = await service.getTrustScore('agent-test-123');
      expect(result.level).toBe(TrustLevel.VERIFIED);
    });
  });
});

