/**
 * AgentKernIdentity Load Testing Script
 *
 * Run: npx ts-node test/load/load-test.ts
 *
 * Prerequisites: Start the server first with `npm run start:dev`
 */

interface LoadTestConfig {
  baseUrl: string;
  concurrency: number;
  duration: number; // seconds
  rampUp: number; // seconds
}

interface LoadTestResult {
  endpoint: string;
  method: string;
  totalRequests: number;
  successfulRequests: number;
  failedRequests: number;
  avgLatencyMs: number;
  minLatencyMs: number;
  maxLatencyMs: number;
  p95LatencyMs: number;
  requestsPerSecond: number;
}

async function makeRequest(
  url: string,
  method: string,
  body?: object,
): Promise<{ success: boolean; latency: number }> {
  const start = performance.now();
  try {
    const response = await fetch(url, {
      method,
      headers: { 'Content-Type': 'application/json' },
      body: body ? JSON.stringify(body) : undefined,
    });
    const latency = performance.now() - start;
    return { success: response.ok, latency };
  } catch {
    return { success: false, latency: performance.now() - start };
  }
}

async function runLoadTest(
  config: LoadTestConfig,
  endpoint: string,
  method: string,
  body?: object,
): Promise<LoadTestResult> {
  const url = `${config.baseUrl}${endpoint}`;
  const latencies: number[] = [];
  let successCount = 0;
  let failCount = 0;

  const startTime = Date.now();
  const endTime = startTime + config.duration * 1000;

  console.log(`  Starting load test: ${method} ${endpoint}`);
  console.log(
    `  Concurrency: ${config.concurrency}, Duration: ${config.duration}s`,
  );

  const promises: Promise<void>[] = [];

  while (Date.now() < endTime) {
    for (let i = 0; i < config.concurrency; i++) {
      promises.push(
        (async () => {
          const result = await makeRequest(url, method, body);
          latencies.push(result.latency);
          if (result.success) successCount++;
          else failCount++;
        })(),
      );
    }
    // Brief pause between batches
    await new Promise((resolve) => setTimeout(resolve, 100));
  }

  await Promise.all(promises);

  // Calculate statistics
  latencies.sort((a, b) => a - b);
  const avgLatency = latencies.reduce((a, b) => a + b, 0) / latencies.length;
  const minLatency = latencies[0] || 0;
  const maxLatency = latencies[latencies.length - 1] || 0;
  const p95Index = Math.floor(latencies.length * 0.95);
  const p95Latency = latencies[p95Index] || 0;
  const duration = (Date.now() - startTime) / 1000;

  return {
    endpoint,
    method,
    totalRequests: latencies.length,
    successfulRequests: successCount,
    failedRequests: failCount,
    avgLatencyMs: avgLatency,
    minLatencyMs: minLatency,
    maxLatencyMs: maxLatency,
    p95LatencyMs: p95Latency,
    requestsPerSecond: latencies.length / duration,
  };
}

async function runAllLoadTests() {
  console.log('🔥 AgentKernIdentity Load Testing Suite\n');
  console.log('='.repeat(60));

  const config: LoadTestConfig = {
    baseUrl: 'http://localhost:3000',
    concurrency: 10,
    duration: 10,
    rampUp: 2,
  };

  console.log(`\nConfiguration:`);
  console.log(`  Base URL: ${config.baseUrl}`);
  console.log(`  Concurrency: ${config.concurrency}`);
  console.log(`  Duration: ${config.duration}s\n`);

  const results: LoadTestResult[] = [];

  // Test 1: Health Check
  console.log('\n📊 Test 1: Health Check Endpoint');
  console.log('-'.repeat(40));
  results.push(await runLoadTest(config, '/api/v1/proof/health', 'GET'));

  // Test 2: DNS Resolve
  console.log('\n📊 Test 2: DNS Resolve Endpoint');
  console.log('-'.repeat(40));
  results.push(
    await runLoadTest(
      config,
      '/api/v1/dns/resolve?agentId=test-agent&principalId=test-principal',
      'GET',
    ),
  );

  // Test 3: Mesh Node Info
  console.log('\n📊 Test 3: Mesh Node Info');
  console.log('-'.repeat(40));
  results.push(await runLoadTest(config, '/api/v1/mesh/node', 'GET'));

  // Test 4: Dashboard
  console.log('\n📊 Test 4: Dashboard');
  console.log('-'.repeat(40));
  results.push(await runLoadTest(config, '/api/v1/dashboard', 'GET'));

  // Print Summary
  console.log('\n' + '='.repeat(80));
  console.log('📈 LOAD TEST RESULTS');
  console.log('='.repeat(80));

  console.log(
    `\n${'Endpoint'.padEnd(35)} | ${'RPS'.padStart(8)} | ${'Avg'.padStart(8)} | ${'P95'.padStart(8)} | ${'Success'.padStart(8)}`,
  );
  console.log('-'.repeat(80));

  for (const result of results) {
    const successRate = (
      (result.successfulRequests / result.totalRequests) *
      100
    ).toFixed(1);
    console.log(
      `${(result.method + ' ' + result.endpoint).substring(0, 35).padEnd(35)} | ` +
        `${result.requestsPerSecond.toFixed(0).padStart(8)} | ` +
        `${result.avgLatencyMs.toFixed(1).padStart(6)}ms | ` +
        `${result.p95LatencyMs.toFixed(1).padStart(6)}ms | ` +
        `${successRate.padStart(6)}%`,
    );
  }

  // Overall summary
  const totalRequests = results.reduce((a, b) => a + b.totalRequests, 0);
  const totalSuccess = results.reduce((a, b) => a + b.successfulRequests, 0);
  const totalFailed = results.reduce((a, b) => a + b.failedRequests, 0);
  const avgRPS =
    results.reduce((a, b) => a + b.requestsPerSecond, 0) / results.length;

  console.log('\n' + '-'.repeat(80));
  console.log(`Total Requests: ${totalRequests.toLocaleString()}`);
  console.log(
    `Successful: ${totalSuccess.toLocaleString()} (${((totalSuccess / totalRequests) * 100).toFixed(1)}%)`,
  );
  console.log(`Failed: ${totalFailed.toLocaleString()}`);
  console.log(`Average RPS: ${avgRPS.toFixed(0)}`);
  console.log('\n✅ Load testing completed!\n');
}

// Run load tests
runAllLoadTests().catch(console.error);
