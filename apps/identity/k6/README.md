# k6 Performance Tests

This directory contains k6 load testing scripts for the Identity API.

## Prerequisites

Install k6:
```bash
# macOS
brew install k6

# Ubuntu/Debian
sudo gpg -k
sudo gpg --no-default-keyring --keyring /usr/share/keyrings/k6-archive-keyring.gpg --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
echo "deb [signed-by=/usr/share/keyrings/k6-archive-keyring.gpg] https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
sudo apt-get update && sudo apt-get install k6
```

## Running Tests

### Smoke Test (Quick Validation)
```bash
k6 run apps/identity/k6/load-test.js --env K6_SCENARIOS=smoke
```

### Load Test (Normal Load)
```bash
k6 run apps/identity/k6/load-test.js
```

### Stress Test (Find Breaking Point)
```bash
k6 run apps/identity/k6/stress-test.js
```

### Soak Test (Long Duration Stability)
```bash
k6 run apps/identity/k6/soak-test.js
```

## Configuration

Set the target URL with the `BASE_URL` environment variable:
```bash
BASE_URL=https://staging.agentkern.dev k6 run apps/identity/k6/load-test.js
```

## Thresholds

| Metric | Target |
|--------|--------|
| p95 Response Time | < 500ms |
| p99 Response Time | < 1000ms |
| Error Rate | < 1% |
| Health Endpoint | < 100ms |
| Gate Analysis | < 200ms |

## CI Integration

Performance tests run automatically:
- **Smoke tests**: On every PR
- **Load tests**: On merge to main
- **Soak tests**: Weekly (Sunday 2 AM UTC)

See `.github/workflows/performance.yml` for details.
