/**
 * AgentKern Identity - API Soak Test
 * 
 * Extended duration test to detect memory leaks and resource exhaustion.
 * Run with: k6 run apps/identity/k6/soak-test.js
 * 
 * Duration: 30 minutes with steady load
 */

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';

const errorRate = new Rate('errors');
const responseTime = new Trend('response_time');

const BASE_URL = __ENV.BASE_URL || 'http://localhost:5004';

export const options = {
  scenarios: {
    soak: {
      executor: 'constant-vus',
      vus: 20,
      duration: '30m',
    },
  },
  thresholds: {
    http_req_duration: ['p(95)<500'],
    errors: ['rate<0.01'],
  },
};

export default function () {
  // Health check
  const healthRes = http.get(`${BASE_URL}/health`);
  check(healthRes, { 'health ok': (r) => r.status === 200 });

  // Gate analyze with varying prompts
  const prompts = [
    'Normal user query about products',
    'Help me book a flight',
    'What is the capital of France?',
  ];
  const prompt = prompts[Math.floor(Math.random() * prompts.length)];
  
  const start = Date.now();
  const gateRes = http.post(
    `${BASE_URL}/api/gate/analyze`,
    JSON.stringify({ prompt }),
    { headers: { 'Content-Type': 'application/json' } }
  );
  responseTime.add(Date.now() - start);

  const success = check(gateRes, {
    'gate status 200': (r) => r.status === 200,
    'has threat level': (r) => JSON.parse(r.body).threatLevel !== undefined,
  });
  errorRate.add(!success);

  sleep(2);
}
