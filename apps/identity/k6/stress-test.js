/**
 * AgentKern Identity - Stress Test
 * 
 * High-load stress test to find breaking points.
 * Run with: k6 run apps/identity/k6/stress-test.js
 * 
 * Gradually increases load to find system limits.
 */

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Counter } from 'k6/metrics';

const errorRate = new Rate('errors');
const requestsCount = new Counter('requests');

const BASE_URL = __ENV.BASE_URL || 'http://localhost:5004';

export const options = {
  scenarios: {
    stress: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '2m', target: 50 },   // Ramp to normal load
        { duration: '3m', target: 100 },  // Ramp to high load
        { duration: '3m', target: 200 },  // Ramp to stress level
        { duration: '2m', target: 300 },  // Peak stress
        { duration: '2m', target: 0 },    // Recovery
      ],
    },
  },
  thresholds: {
    http_req_duration: ['p(95)<2000'],  // Allow higher latency under stress
    errors: ['rate<0.10'],               // Allow up to 10% errors under stress
  },
};

export default function () {
  requestsCount.add(1);

  // Health check (lightweight)
  const healthRes = http.get(`${BASE_URL}/health`, { timeout: '5s' });
  const healthOk = check(healthRes, { 'health ok': (r) => r.status === 200 });
  errorRate.add(!healthOk);

  // Gate analyze (compute-intensive)
  const gateRes = http.post(
    `${BASE_URL}/api/gate/analyze`,
    JSON.stringify({ prompt: 'Stress test prompt to evaluate system performance' }),
    { 
      headers: { 'Content-Type': 'application/json' },
      timeout: '10s',
    }
  );
  const gateOk = check(gateRes, { 'gate ok': (r) => r.status === 200 });
  errorRate.add(!gateOk);

  sleep(0.5); // Reduced think time for higher stress
}
