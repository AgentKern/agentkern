import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  stages: [
    { duration: '1m', target: 50 }, // Ramp-up to 50 users
    { duration: '3m', target: 50 }, // Stay at 50 users
    { duration: '1m', target: 100 }, // Peak load: 100 users
    { duration: '1m', target: 0 }, // Ramp-down
  ],
  thresholds: {
    http_req_duration: ['p(95)<500'], // 95% of requests should be under 500ms
    http_req_failed: ['rate<0.01'],    // Less than 1% failure rate
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:3000';

export default function () {
  // 1. Health Check
  const healthRes = http.get(`${BASE_URL}/api/v1/proof/health`);
  check(healthRes, {
    'health status is 200': (r) => r.status === 200,
  });

  // 2. Verify Proof (Simulated Load)
  const verifyPayload = JSON.stringify({
    proof: 'v1.eyJ2ZXJzaW9uIjoiMS4wIiwicHJvb2ZfaWQiOiJmYWtlIn0.fake_signature',
  });

  const params = {
    headers: {
      'Content-Type': 'application/json',
    },
  };

  const verifyRes = http.post(`${BASE_URL}/api/v1/proof/verify`, verifyPayload, params);
  check(verifyRes, {
    'verify status is 200 or 401': (r) => [200, 401].includes(r.status),
  });

  // 3. DNS Resolution (Simulated)
  const resolveRes = http.get(`${BASE_URL}/api/v1/dns/resolve?agentId=agent-1&principalId=user-1`);
  check(resolveRes, {
    'resolve status is 200': (r) => r.status === 200,
  });

  sleep(1);
}
