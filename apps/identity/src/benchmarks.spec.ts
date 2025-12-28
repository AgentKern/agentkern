/**
 * AgentKernIdentity Performance Benchmarks
 * 
 * These are integration benchmarks that require a database connection.
 * Skip in CI, run manually with:
 *   npm run test -- --testPathPattern="benchmark" --testTimeout=60000
 * 
 * To run locally, ensure DATABASE_URL is configured.
 */

import { Test, TestingModule } from '@nestjs/testing';
import { getRepositoryToken } from '@nestjs/typeorm';
import { DnsResolutionService } from './services/dns-resolution.service';
import { PolicyService } from './services/policy.service';
import { AuditLoggerService } from './services/audit-logger.service';
import { TrustRecordEntity } from './entities/trust-record.entity';

interface BenchmarkResult {
  name: string;
  operations: number;
  durationMs: number;
  opsPerSecond: number;
  avgLatencyMs: number;
}

async function benchmark<T>(
  name: string,
  operation: () => Promise<T> | T,
  iterations: number = 1000,
): Promise<BenchmarkResult> {
  // Warmup
  for (let i = 0; i < 10; i++) {
    await operation();
  }

  const start = performance.now();
  for (let i = 0; i < iterations; i++) {
    await operation();
  }
  const duration = performance.now() - start;

  return {
    name,
    operations: iterations,
    durationMs: duration,
    opsPerSecond: Math.round((iterations / duration) * 1000),
    avgLatencyMs: duration / iterations,
  };
}

// Mock repository factory with stateful data for benchmarks
const createMockRepository = () => {
  const records = new Map<string, any>();
  return {
    find: jest.fn().mockImplementation(({ where }) => {
      if (where?.principalId) {
        return Promise.resolve(
          Array.from(records.values()).filter(r => r.principalId === where.principalId)
        );
      }
      return Promise.resolve(Array.from(records.values()));
    }),
    findOne: jest.fn().mockImplementation(({ where }) => {
      const key = `${where.agentId}-${where.principalId}`;
      return Promise.resolve(records.get(key) || null);
    }),
    save: jest.fn().mockImplementation(entity => {
      const key = `${entity.agentId}-${entity.principalId}`;
      records.set(key, { ...entity, id: key });
      return Promise.resolve(records.get(key));
    }),
    create: jest.fn().mockImplementation(entity => entity),
    delete: jest.fn().mockResolvedValue({ affected: 1 }),
  };
};

describe('Performance Benchmarks', () => {
  let dnsService: DnsResolutionService;
  let policyService: PolicyService;
  let auditLogger: AuditLoggerService;

  beforeAll(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [
        DnsResolutionService,
        PolicyService,
        AuditLoggerService,
        {
          provide: getRepositoryToken(TrustRecordEntity),
          useValue: createMockRepository(),
        },
      ],
    }).compile();

    dnsService = module.get<DnsResolutionService>(DnsResolutionService);
    policyService = module.get<PolicyService>(PolicyService);
    auditLogger = module.get<AuditLoggerService>(AuditLoggerService);
  });

  it('DNS Resolve benchmark', async () => {
    await dnsService.registerTrust('bench-agent-1', 'bench-principal-1');

    const result = await benchmark(
      'DNS Resolve (cached)',
      () => dnsService.resolve({ agentId: 'bench-agent-1', principalId: 'bench-principal-1' }),
      5000,
    );

    console.log(`\n📊 ${result.name}: ${result.opsPerSecond.toLocaleString()} ops/sec (${result.avgLatencyMs.toFixed(3)}ms avg)`);
    expect(result.opsPerSecond).toBeGreaterThan(1000);
  }, 60000);

  it('DNS Register benchmark', async () => {
    const result = await benchmark(
      'DNS Register (new)',
      () => dnsService.registerTrust(`agent-${Math.random()}`, `principal-${Math.random()}`),
      1000,
    );

    console.log(`📊 ${result.name}: ${result.opsPerSecond.toLocaleString()} ops/sec (${result.avgLatencyMs.toFixed(3)}ms avg)`);
    expect(result.opsPerSecond).toBeGreaterThan(100);
  }, 60000);

  it('Policy Evaluate benchmark', async () => {
    const result = await benchmark(
      'Policy Evaluate',
      () => policyService.evaluatePolicies({
        agentId: 'agent-1',
        principalId: 'principal-1',
        action: 'transfer',
        target: { service: 'bank', endpoint: '/transfer', method: 'POST' },
        amount: 5000,
      }),
      5000,
    );

    console.log(`📊 ${result.name}: ${result.opsPerSecond.toLocaleString()} ops/sec (${result.avgLatencyMs.toFixed(3)}ms avg)`);
    expect(result.opsPerSecond).toBeGreaterThan(1000);
  }, 60000);

  it('Audit Log Write benchmark', async () => {
    const result = await benchmark(
      'Audit Log Write',
      () => auditLogger.logVerificationSuccess(
        'proof-' + Math.random(),
        'principal-1',
        'agent-1',
        'test',
        '/api/test',
      ),
      3000,
    );

    console.log(`📊 ${result.name}: ${result.opsPerSecond.toLocaleString()} ops/sec (${result.avgLatencyMs.toFixed(3)}ms avg)\n`);
    expect(result.opsPerSecond).toBeGreaterThan(100);
  }, 60000);
});
