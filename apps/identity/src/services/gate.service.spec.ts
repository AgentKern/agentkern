import { Test, TestingModule } from '@nestjs/testing';
import { GateService, PromptAnalysis, ContextScanResult, Attestation, VerificationResult } from './gate.service';
import { GatePolicyRepository } from '../repositories/gate-policy.repository';

// ============================================================================
// MOCKS
// ============================================================================


const mockPolicyRepository = {
  findAll: jest.fn(),
  findById: jest.fn(),
  create: jest.fn(),
  update: jest.fn(),
  delete: jest.fn(),
};

const mockPromptAnalysis: PromptAnalysis = {
  threat_level: 'None',
  attacks: [],
  matched_patterns: [],
  confidence: 0.95,
  action: 'Allow',
  latency_us: 50,
};

const mockHighThreatAnalysis: PromptAnalysis = {
  threat_level: 'High',
  attacks: ['sql_injection', 'prompt_injection'],
  matched_patterns: ['DROP TABLE', 'ignore previous'],
  confidence: 0.99,
  action: 'Block',
  latency_us: 45,
};

const mockContextResult: ContextScanResult = {
  safe: true,
  injections_found: 0,
  suspicious_chunks: [],
  latency_us: 100,
};

const mockAttestation: Attestation = {
  platform: 'sgx',
  quote: [1, 2, 3, 4],
  measurement: [5, 6, 7, 8],
  user_data: [9, 10],
  timestamp: Date.now(),
  cert_chain: ['cert1', 'cert2'],
};

const mockVerificationResult: VerificationResult = {
  request_id: 'req-123',
  allowed: true,
  evaluated_policies: ['policy-1', 'policy-2'],
  blocking_policies: [],
  symbolic_risk_score: 10,
  neural_risk_score: 15,
  final_risk_score: 12,
  reasoning: 'Action permitted by all policies',
  latency: {
    total_us: 150,
    symbolic_us: 100,
    neural_us: 50,
  },
};

const mockPolicyEntity = {
  id: 'policy-1',
  name: 'Rate Limit Policy',
  description: 'Limits requests per agent',
  active: true,
  rules: [
    {
      id: 'rule-1',
      condition: 'rate > 100',
      action: 'deny' as const,
      priority: 1,
    },
  ],
  createdAt: new Date('2025-12-01'),
  updatedAt: new Date('2026-01-01'),
};

const mockBridge = {
  guardPrompt: jest.fn().mockReturnValue(JSON.stringify(mockPromptAnalysis)),
  guardContext: jest.fn().mockReturnValue(JSON.stringify(mockContextResult)),
  attest: jest.fn().mockReturnValue(JSON.stringify(mockAttestation)),
  verify: jest.fn().mockResolvedValue(JSON.stringify(mockVerificationResult)),
  registerPolicy: jest.fn().mockResolvedValue(JSON.stringify({ success: true })),
  // WASM Actor Management
  gateWasmListActors: jest.fn().mockReturnValue(JSON.stringify([
    {
      name: 'prompt-guard',
      version: '1.0.0',
      capabilities: [{ name: 'prompt_guard', inputSchema: { type: 'object' } }],
      sizeBytes: 245760,
      loadedAt: new Date().toISOString(),
      invocations: 0,
      avgLatencyUs: 50,
    },
  ])),
  gateWasmGetActor: jest.fn().mockImplementation((name: string) => {
    if (name === 'prompt-guard') {
      return JSON.stringify({
        name: 'prompt-guard',
        version: '1.0.0',
        capabilities: [{ name: 'prompt_guard' }],
        sizeBytes: 245760,
        loadedAt: new Date().toISOString(),
        invocations: 0,
        avgLatencyUs: 50,
      });
    }
    return JSON.stringify({ error: `Actor not found: ${name}` });
  }),
  gateWasmRegisterActor: jest.fn().mockImplementation((name: string, version: string, _wasmBase64: string, _caps: string) => {
    return JSON.stringify({
      name,
      version,
      sizeBytes: 4,
      loadedAt: new Date().toISOString(),
      invocations: 0,
      avgLatencyUs: 0,
    });
  }),
  gateWasmUnregisterActor: jest.fn().mockReturnValue(true),
  gateWasmStats: jest.fn().mockReturnValue(JSON.stringify({
    actorCount: 1,
    totalSizeBytes: 245760,
    totalInvocations: 0,
  })),
};

// ============================================================================
// TEST SUITE
// ============================================================================

describe('GateService', () => {
  let service: GateService;

  beforeEach(async () => {
    // Reset all mocks
    jest.clearAllMocks();
    
    const module: TestingModule = await Test.createTestingModule({
      providers: [
        GateService,
        {
          provide: GatePolicyRepository,
          useValue: mockPolicyRepository,
        },
      ],
    }).compile();

    service = module.get<GateService>(GateService);
    
    // Inject mock bridge for testing
    (service as any).bridge = { ...mockBridge };
    (service as any).bridgeLoaded = true;
  });

  // =========================================================================
  // INITIALIZATION
  // =========================================================================

  describe('initialization', () => {
    it('should be defined', () => {
      expect(service).toBeDefined();
    });

    it('should report operational status', () => {
      expect(service.isOperational()).toBe(true);
    });
  });

  // =========================================================================
  // verifyBridge
  // =========================================================================

  describe('verifyBridge', () => {
    it('should succeed when bridge returns valid JSON', () => {
      (service as any).bridge.guardPrompt.mockReturnValue(
        JSON.stringify({ test: true }),
      );

      const verifyBridge = (service as any).verifyBridge.bind(service);
      
      expect(() => verifyBridge()).not.toThrow();
    });

    it('should throw when bridge returns null', () => {
      (service as any).bridge.guardPrompt.mockReturnValue(null);

      const verifyBridge = (service as any).verifyBridge.bind(service);
      
      expect(() => verifyBridge()).toThrow('Bridge returned null for test call');
    });

    it('should throw when bridge returns invalid JSON', () => {
      (service as any).bridge.guardPrompt.mockReturnValue('not json {{{');

      const verifyBridge = (service as any).verifyBridge.bind(service);
      
      expect(() => verifyBridge()).toThrow('Bridge verification failed');
    });

    it('should throw when bridge guardPrompt throws', () => {
      (service as any).bridge.guardPrompt.mockImplementation(() => {
        throw new Error('Bridge crashed');
      });

      const verifyBridge = (service as any).verifyBridge.bind(service);
      
      expect(() => verifyBridge()).toThrow('Bridge verification failed: Bridge crashed');
    });
  });

  // =========================================================================
  // guardPrompt
  // =========================================================================


  describe('guardPrompt', () => {
    beforeEach(() => {
      // Reset bridge mock to default state
      (service as any).bridge.guardPrompt.mockReturnValue(
        JSON.stringify(mockPromptAnalysis),
      );
    });

    it('should return analysis for safe prompts', () => {
      const result = service.guardPrompt('Hello, how are you?');

      expect(result).toBeDefined();
      expect(result?.threat_level).toBe('None');
      expect(result?.action).toBe('Allow');
    });

    it('should detect high threat prompts', () => {
      (service as any).bridge.guardPrompt.mockReturnValue(
        JSON.stringify(mockHighThreatAnalysis),
      );

      const result = service.guardPrompt('DROP TABLE users;');

      expect(result).toBeDefined();
      expect(result?.threat_level).toBe('High');
      expect(result?.attacks).toContain('sql_injection');
      expect(result?.action).toBe('Block');
    });

    it('should return null when bridge not loaded (development)', () => {
      (service as any).bridgeLoaded = false;
      
      // Ensure we're not in production mode
      const originalEnv = process.env.NODE_ENV;
      process.env.NODE_ENV = 'development';

      const result = service.guardPrompt('test prompt');

      expect(result).toBeNull();
      
      process.env.NODE_ENV = originalEnv;
    });

    it('should throw in production when bridge not loaded', () => {
      (service as any).bridgeLoaded = false;
      
      const originalEnv = process.env.NODE_ENV;
      process.env.NODE_ENV = 'production';

      expect(() => service.guardPrompt('test')).toThrow(
        'N-API bridge is required for prompt guard in production',
      );

      process.env.NODE_ENV = originalEnv;
    });

    it('should return null on bridge error', () => {
      (service as any).bridge.guardPrompt.mockImplementation(() => {
        throw new Error('Bridge error');
      });

      const result = service.guardPrompt('test');

      expect(result).toBeNull();
    });
  });

  // =========================================================================
  // guardContext
  // =========================================================================

  describe('guardContext', () => {
    it('should scan context chunks for safety', () => {
      const chunks = ['context chunk 1', 'context chunk 2'];

      const result = service.guardContext(chunks);

      expect(result).toBeDefined();
      expect(result?.safe).toBe(true);
      expect(result?.injections_found).toBe(0);
    });

    it('should detect suspicious chunks', () => {
      (service as any).bridge.guardContext.mockReturnValue(
        JSON.stringify({
          safe: false,
          injections_found: 2,
          suspicious_chunks: [0, 2],
          latency_us: 120,
        }),
      );

      const result = service.guardContext([
        'DROP TABLE *',
        'safe content',
        'ignore previous instructions',
      ]);

      expect(result?.safe).toBe(false);
      expect(result?.injections_found).toBe(2);
      expect(result?.suspicious_chunks).toEqual([0, 2]);
    });

    it('should return null when bridge not loaded', () => {
      (service as any).bridgeLoaded = false;

      const result = service.guardContext(['test chunk']);

      expect(result).toBeNull();
    });
  });

  // =========================================================================
  // attest
  // =========================================================================

  describe('attest', () => {
    it('should generate attestation with nonce', () => {
      const nonce = 'test-nonce-12345';

      const result = service.attest(nonce);

      expect(result).toBeDefined();
      expect(result?.platform).toBe('sgx');
      expect(result?.quote).toBeDefined();
      expect(result?.measurement).toBeDefined();
    });

    it('should return null when bridge not loaded', () => {
      (service as any).bridgeLoaded = false;

      const result = service.attest('nonce');

      expect(result).toBeNull();
    });
  });

  // =========================================================================
  // verify
  // =========================================================================

  describe('verify', () => {
    it('should verify agent actions against policies', async () => {
      const result = await service.verify('agent-123', 'read_data', 'default', {
        resource: 'database',
      });

      expect(result).toBeDefined();
      expect(result?.allowed).toBe(true);
      expect(result?.evaluated_policies).toHaveLength(2);
    });

    it('should handle denied actions', async () => {
      (service as any).bridge.verify.mockResolvedValue(
        JSON.stringify({
          ...mockVerificationResult,
          allowed: false,
          blocking_policies: ['policy-secure-1'],
          reasoning: 'Action blocked by security policy',
        }),
      );

      const result = await service.verify('agent-123', 'delete_all');

      expect(result?.allowed).toBe(false);
      expect(result?.blocking_policies).toHaveLength(1);
    });

    it('should return null when bridge not loaded', async () => {
      (service as any).bridgeLoaded = false;

      const result = await service.verify('agent-123', 'action');

      expect(result).toBeNull();
    });
  });

  // =========================================================================
  // shouldBlockPrompt
  // =========================================================================

  describe('shouldBlockPrompt', () => {
    beforeEach(() => {
      // Reset bridge mock to default state
      (service as any).bridge.guardPrompt.mockReturnValue(
        JSON.stringify(mockPromptAnalysis),
      );
    });

    it('should return false for safe prompts', () => {
      const result = service.shouldBlockPrompt('Hello world');

      expect(result).toBe(false);
    });

    it('should return true for high threat prompts', () => {
      (service as any).bridge.guardPrompt.mockReturnValue(
        JSON.stringify(mockHighThreatAnalysis),
      );

      const result = service.shouldBlockPrompt('DROP TABLE users;');

      expect(result).toBe(true);
    });

    it('should return true for critical threat prompts', () => {
      (service as any).bridge.guardPrompt.mockReturnValue(
        JSON.stringify({
          ...mockPromptAnalysis,
          threat_level: 'Critical',
        }),
      );

      const result = service.shouldBlockPrompt('malicious prompt');

      expect(result).toBe(true);
    });

    it('should fail-closed when bridge unavailable (development)', () => {
      (service as any).bridgeLoaded = false;
      
      const originalEnv = process.env.NODE_ENV;
      process.env.NODE_ENV = 'development';

      const result = service.shouldBlockPrompt('any prompt');

      // Should fail-closed (block) when security check unavailable
      expect(result).toBe(true);
      
      process.env.NODE_ENV = originalEnv;
    });
  });

  // =========================================================================
  // analyzePrompt (HTTP API version)
  // =========================================================================

  describe('analyzePrompt', () => {
    beforeEach(() => {
      // Reset bridge mock to default state
      (service as any).bridge.guardPrompt.mockReturnValue(
        JSON.stringify(mockPromptAnalysis),
      );
    });

    it('should return safe analysis for clean prompts', async () => {
      const result = await service.analyzePrompt('Safe prompt text');

      expect(result.safe).toBe(true);
      expect(result.threatLevel).toBe('none');
      expect(result.attacks).toHaveLength(0);
    });

    it('should return unsafe analysis for threats', async () => {
      (service as any).bridge.guardPrompt.mockReturnValue(
        JSON.stringify(mockHighThreatAnalysis),
      );

      const result = await service.analyzePrompt('DROP TABLE users;');

      expect(result.safe).toBe(false);
      expect(result.threatLevel).toBe('high');
      expect(result.attacks).toContain('sql_injection');
      expect(result.threatType).toBe('sql_injection');
    });

    it('should return fail-closed result when bridge unavailable', async () => {
      (service as any).bridgeLoaded = false;
      
      const originalEnv = process.env.NODE_ENV;
      process.env.NODE_ENV = 'development';

      const result = await service.analyzePrompt('test');

      expect(result.safe).toBe(false);
      expect(result.threatLevel).toBe('critical');
      expect(result.threatType).toBe('security_unavailable');
      
      process.env.NODE_ENV = originalEnv;
    });
  });

  // =========================================================================
  // Policy management
  // =========================================================================

  describe('listPolicies', () => {
    it('should list all policies from repository', async () => {
      mockPolicyRepository.findAll.mockResolvedValue([mockPolicyEntity]);

      const result = await service.listPolicies();

      expect(result).toHaveLength(1);
      expect(result[0].id).toBe('policy-1');
      expect(result[0].name).toBe('Rate Limit Policy');
      expect(mockPolicyRepository.findAll).toHaveBeenCalled();
    });

    it('should return empty array when no policies', async () => {
      mockPolicyRepository.findAll.mockResolvedValue([]);

      const result = await service.listPolicies();

      expect(result).toHaveLength(0);
    });
  });

  describe('getPolicy', () => {
    it('should get policy by ID', async () => {
      mockPolicyRepository.findById.mockResolvedValue(mockPolicyEntity);

      const result = await service.getPolicy('policy-1');

      expect(result.id).toBe('policy-1');
      expect(result.active).toBe(true);
      expect(mockPolicyRepository.findById).toHaveBeenCalledWith('policy-1');
    });

    it('should throw when policy not found', async () => {
      mockPolicyRepository.findById.mockResolvedValue(null);

      await expect(service.getPolicy('nonexistent')).rejects.toThrow(
        'Policy nonexistent not found',
      );
    });
  });

  describe('createPolicy', () => {
    it('should create and return new policy', async () => {
      mockPolicyRepository.create.mockResolvedValue(mockPolicyEntity);
      mockPolicyRepository.findAll.mockResolvedValue([mockPolicyEntity]);

      const result = await service.createPolicy({
        name: 'Rate Limit Policy',
        description: 'Limits requests per agent',
        rules: [
          {
            id: 'rule-1',
            condition: 'rate > 100',
            action: 'deny',
            priority: 1,
          },
        ],
      });

      expect(result.id).toBe('policy-1');
      expect(result.name).toBe('Rate Limit Policy');
      expect(mockPolicyRepository.create).toHaveBeenCalled();
    });
  });

  // =========================================================================
  // Compliance checks
  // =========================================================================

  describe('compliance checks', () => {
    it('should check PCI compliance', async () => {
      const result = await service.checkPciCompliance({
        encryptionEnabled: true,
        accessLogged: true,
      });

      expect(result.standard).toBe('PCI-DSS');
      expect(result.compliant).toBeDefined();
    });

    it('should check HIPAA compliance', async () => {
      const result = await service.checkHipaaCompliance({
        phi_encrypted: true,
        audit_trail: true,
      });

      expect(result.standard).toBe('HIPAA Security Rule');
    });

    it('should check GDPR compliance', async () => {
      const result = await service.checkGdprCompliance({
        consent_obtained: true,
        data_portability: true,
      });

      expect(result.standard).toBe('GDPR & EU AI Act');
    });
  });

  // =========================================================================
  // WASM Actors
  // =========================================================================

  describe('WASM actors', () => {
    it('should list WASM actors', async () => {
      const result = await service.listWasmActors();

      expect(result).toHaveLength(1);
      expect(result[0].name).toBe('prompt-guard');
    });

    it('should get specific WASM actor', async () => {
      const result = await service.getWasmActor('prompt-guard');

      expect(result.name).toBe('prompt-guard');
      expect(result.version).toBe('1.0.0');
    });

    it('should throw when WASM actor not found', async () => {
      await expect(service.getWasmActor('nonexistent')).rejects.toThrow(
        'WASM actor nonexistent not found',
      );
    });

    it('should register new WASM actor', async () => {
      const result = await service.registerWasmActor({
        name: 'new-actor',
        version: '1.0.0',
        wasmBase64: 'dGVzdA==', // "test" in base64
        capabilities: [{ name: 'test_capability' }],
      });

      expect(result.name).toBe('new-actor');
      expect(result.sizeBytes).toBe(4); // "test".length
    });
  });

  // =========================================================================
  // Sync Policies
  // =========================================================================

  describe('syncPolicies', () => {
    it('should sync policies to bridge when loaded', async () => {
      mockPolicyRepository.findAll.mockResolvedValue([mockPolicyEntity]);

      await service.syncPolicies();

      expect(mockPolicyRepository.findAll).toHaveBeenCalled();
      expect((service as any).bridge.registerPolicy).toHaveBeenCalled();
    });

    it('should skip sync when bridge not loaded', async () => {
      (service as any).bridgeLoaded = false;

      await service.syncPolicies();

      expect(mockPolicyRepository.findAll).not.toHaveBeenCalled();
    });

    it('should handle policy registration errors gracefully', async () => {
      mockPolicyRepository.findAll.mockResolvedValue([mockPolicyEntity]);
      (service as any).bridge.registerPolicy.mockResolvedValue(
        JSON.stringify({ error: 'Invalid policy format' }),
      );

      // Should not throw
      await expect(service.syncPolicies()).resolves.not.toThrow();
    });

    it('should handle bridge errors during sync', async () => {
      mockPolicyRepository.findAll.mockResolvedValue([mockPolicyEntity]);
      (service as any).bridge.registerPolicy.mockRejectedValue(
        new Error('Bridge error'),
      );

      // Should not throw
      await expect(service.syncPolicies()).resolves.not.toThrow();
    });
  });

  // =========================================================================
  // Threat Level Mapping
  // =========================================================================

  describe('threat level mapping', () => {
    it('should map Low threat as safe', async () => {
      (service as any).bridge.guardPrompt.mockReturnValue(
        JSON.stringify({
          ...mockPromptAnalysis,
          threat_level: 'Low',
        }),
      );

      const result = await service.analyzePrompt('slightly suspicious');

      expect(result.safe).toBe(true);
      expect(result.threatLevel).toBe('low');
    });

    it('should map Medium threat as unsafe', async () => {
      (service as any).bridge.guardPrompt.mockReturnValue(
        JSON.stringify({
          ...mockPromptAnalysis,
          threat_level: 'Medium',
        }),
      );

      const result = await service.analyzePrompt('medium risk prompt');

      expect(result.safe).toBe(false);
      expect(result.threatLevel).toBe('medium');
    });

    it('should not block medium threat prompts', () => {
      (service as any).bridge.guardPrompt.mockReturnValue(
        JSON.stringify({
          ...mockPromptAnalysis,
          threat_level: 'Medium',
        }),
      );

      const result = service.shouldBlockPrompt('medium threat');

      // Only High and Critical are blocked
      expect(result).toBe(false);
    });

    it('should not block low threat prompts', () => {
      (service as any).bridge.guardPrompt.mockReturnValue(
        JSON.stringify({
          ...mockPromptAnalysis,
          threat_level: 'Low',
        }),
      );

      const result = service.shouldBlockPrompt('low threat');

      expect(result).toBe(false);
    });
  });

  // =========================================================================
  // Error Handling
  // =========================================================================

  describe('error handling', () => {
    it('should handle context guard errors', () => {
      (service as any).bridge.guardContext.mockImplementation(() => {
        throw new Error('Context scan failed');
      });

      const result = service.guardContext(['chunk1', 'chunk2']);

      expect(result).toBeNull();
    });

    it('should handle attestation errors', () => {
      (service as any).bridge.attest.mockImplementation(() => {
        throw new Error('Attestation failed');
      });

      const result = service.attest('nonce');

      expect(result).toBeNull();
    });

    it('should handle verify errors', async () => {
      (service as any).bridge.verify.mockRejectedValue(
        new Error('Verify failed'),
      );

      const result = await service.verify('agent', 'action');

      expect(result).toBeNull();
    });

    it('should handle invalid JSON from bridge', () => {
      (service as any).bridge.guardPrompt.mockReturnValue('not valid json');

      const result = service.guardPrompt('test');

      expect(result).toBeNull();
    });
  });

  // =========================================================================
  // Policy Transform
  // =========================================================================

  describe('policy transformation', () => {
    it('should transform policy entity to response', async () => {
      mockPolicyRepository.findById.mockResolvedValue({
        ...mockPolicyEntity,
        description: null, // Test null handling
      });

      const result = await service.getPolicy('policy-1');

      expect(result.description).toBeUndefined();
      expect(result.createdAt).toBeDefined();
      expect(result.updatedAt).toBeDefined();
    });

    it('should handle policies without description', async () => {
      mockPolicyRepository.findAll.mockResolvedValue([
        {
          ...mockPolicyEntity,
          description: null,
          updatedAt: null,
        },
      ]);

      const result = await service.listPolicies();

      expect(result[0].description).toBeUndefined();
    });
  });

  // =========================================================================
  // Verify with context
  // =========================================================================

  describe('verify with context', () => {
    it('should pass context to bridge as JSON', async () => {
      const context = { resource: 'database', action: 'read' };

      await service.verify('agent-123', 'read_data', context);

      expect((service as any).bridge.verify).toHaveBeenCalledWith(
        'agent-123',
        'read_data',
        JSON.stringify(context),
      );
    });

    it('should work without context', async () => {
      await service.verify('agent-123', 'action');

      expect((service as any).bridge.verify).toHaveBeenCalledWith(
        'agent-123',
        'action',
        undefined,
      );
    });
  });

  // =========================================================================
  // Analyze Prompt edge cases
  // =========================================================================

  describe('analyzePrompt edge cases', () => {
    it('should handle empty attacks array', async () => {
      (service as any).bridge.guardPrompt.mockReturnValue(
        JSON.stringify({
          ...mockPromptAnalysis,
          attacks: [],
        }),
      );

      const result = await service.analyzePrompt('clean prompt');

      expect(result.threatType).toBeUndefined();
    });

    it('should handle multiple attacks', async () => {
      (service as any).bridge.guardPrompt.mockReturnValue(
        JSON.stringify({
          ...mockHighThreatAnalysis,
          attacks: ['xss', 'sqli', 'rce'],
        }),
      );

      const result = await service.analyzePrompt('multi-attack');

      expect(result.threatType).toBe('xss'); // First attack
      expect(result.attacks).toHaveLength(3);
    });

    it('should calculate score from confidence', async () => {
      (service as any).bridge.guardPrompt.mockReturnValue(
        JSON.stringify({
          ...mockPromptAnalysis,
          confidence: 0.85,
        }),
      );

      const result = await service.analyzePrompt('test');

      expect(result.score).toBe(85); // 0.85 * 100 rounded
    });

    it('should include matched patterns as reason', async () => {
      (service as any).bridge.guardPrompt.mockReturnValue(
        JSON.stringify({
          ...mockHighThreatAnalysis,
          matched_patterns: ['DROP', 'TABLE'],
        }),
      );

      const result = await service.analyzePrompt('DROP TABLE');

      expect(result.reason).toBe('DROP, TABLE');
    });
  });

  // =========================================================================
  // isOperational
  // =========================================================================

  describe('isOperational', () => {
    it('should return false when bridge not loaded', () => {
      (service as any).bridgeLoaded = false;

      expect(service.isOperational()).toBe(false);
    });

    it('should return true when bridge loaded', () => {
      (service as any).bridgeLoaded = true;

      expect(service.isOperational()).toBe(true);
    });
  });
});

