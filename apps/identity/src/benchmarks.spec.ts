/**
 * AgentKernIdentity Performance Benchmarks
 * 
 * Run: npm run build && node dist/test/benchmarks/performance.benchmark.js
 * Or simpler: npm run test -- --testPathPattern="benchmark" --testTimeout=60000
 */

import { Test, TestingModule } from '@nestjs/testing';
import { AppModule } from './app.module';
import { DnsResolutionService } from './services/dns-resolution.service';
import { PolicyService } from './services/policy.service';
import { AuditLoggerService } from './services/audit-logger.service';

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

describe('Performance Benchmarks', () => {
  let dnsService: DnsResolutionService;
  let policyService: PolicyService;
  let auditLogger: AuditLoggerService;

  beforeAll(async () => {
    const module: TestingModule = await Test.createTestingModule({
      imports: [AppModule],
    }).compile();

    dnsService = module.get<DnsResolutionService>(DnsResolutionService);
    policyService = module.get<PolicyService>(PolicyService);
    auditLogger = module.get<AuditLoggerService>(AuditLoggerService);
  });

  it('DNS Resolve benchmark', async () => {
    dnsService.registerTrust('bench-agent-1', 'bench-principal-1');

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
