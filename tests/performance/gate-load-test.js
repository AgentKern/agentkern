// AgentKern Gate API Performance Test
// Run with: k6 run tests/performance/gate-load-test.js

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';

// Custom metrics
const errorRate = new Rate('errors');
const verifyLatency = new Trend('verify_latency');

// Test configuration
export const options = {
  stages: [
    { duration: '30s', target: 10 },   // Ramp up to 10 users
    { duration: '1m', target: 10 },    // Stay at 10 users
    { duration: '30s', target: 50 },   // Ramp up to 50 users
    { duration: '1m', target: 50 },    // Stay at 50 users
    { duration: '30s', target: 0 },    // Ramp down
  ],
  thresholds: {
    http_req_duration: ['p(95)<50'],   // 95% of requests under 50ms (per AgentKern SLA)
    http_req_failed: ['rate<0.01'],    // Less than 1% failure rate
    errors: ['rate<0.01'],             // Custom error rate
    verify_latency: ['p(99)<100'],     // 99% verify calls under 100ms
  },
};

const BASE_URL = __ENV.GATE_URL || 'http://localhost:3000';

// Test scenarios
export default function () {
  // Test 1: Health check
  const healthRes = http.get(`${BASE_URL}/api/gate/health`);
  check(healthRes, {
    'health status is 200': (r) => r.status === 200,
    'health response is healthy': (r) => {
      try {
        const body = JSON.parse(r.body);
        return body.status === 'healthy';
      } catch {
        return false;
      }
    },
  });

  // Test 2: Policy verification (critical path)
  const verifyPayload = JSON.stringify({
    agentId: `load-test-agent-${__VU}`,
    action: 'read_data',
    context: {
      timestamp: new Date().toISOString(),
      vu: __VU,
      iteration: __ITER,
    },
  });

  const verifyStart = Date.now();
  const verifyRes = http.post(`${BASE_URL}/api/gate/verify`, verifyPayload, {
    headers: { 'Content-Type': 'application/json' },
  });
  const verifyTime = Date.now() - verifyStart;
  
  verifyLatency.add(verifyTime);
  
  const verifySuccess = check(verifyRes, {
    'verify status is 200': (r) => r.status === 200,
    'verify has requestId': (r) => {
      try {
        const body = JSON.parse(r.body);
        return body.requestId !== undefined;
      } catch {
        return false;
      }
    },
    'verify latency under 50ms': (r) => verifyTime < 50,
  });

  errorRate.add(!verifySuccess);

  sleep(1);
}

// Lifecycle hooks
export function setup() {
  console.log('Starting AgentKern Gate performance test');
  console.log(`Target URL: ${BASE_URL}`);
  
  // Verify server is reachable
  const res = http.get(`${BASE_URL}/api/gate/health`);
  if (res.status !== 200) {
    console.error('Gate server not reachable, test may fail');
  }
  
  return { startTime: new Date().toISOString() };
}

export function teardown(data) {
  console.log(`Test completed. Started at: ${data.startTime}`);
}
