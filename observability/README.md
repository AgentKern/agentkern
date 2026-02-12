# AgentKern Observability Stack

Production-ready observability setup using OpenTelemetry, Prometheus, Grafana, and Tempo.

## Quick Start

```bash
# Start the full observability stack
cd observability
docker compose -f docker-compose.otel.yml up -d

# Access dashboards
# Grafana: http://localhost:3001 (admin/admin)
# Prometheus: http://localhost:9090
# Tempo (traces): http://localhost:3200
```

## Components

| Component | Port | Purpose |
|-----------|------|---------|
| OTEL Collector | 4317 | Receives traces/metrics from apps |
| Prometheus | 9090 | Metrics storage & querying |
| Grafana | 3001 | Dashboards & visualization |
| Tempo | 3200 | Distributed tracing backend |

## Architecture

```
┌─────────────────┐     ┌──────────────────┐
│ AgentKern Server│────▶│  OTEL Collector  │
│    (Rust)       │     │  :4317 (gRPC)    │
└─────────────────┘     └────────┬─────────┘
                                 │
        ┌────────────────────────┼────────────────────────┐
        │                        │                        │
        ▼                        ▼                        ▼
┌───────────────┐       ┌───────────────┐       ┌───────────────┐
│  Prometheus   │       │    Tempo      │       │   (Loki)      │
│  (metrics)    │       │   (traces)    │       │   (logs)      │
└───────┬───────┘       └───────┬───────┘       └───────────────┘
        │                       │
        └───────────┬───────────┘
                    │
            ┌───────▼───────┐
            │    Grafana    │
            │  (dashboards) │
            └───────────────┘
```

## Configuration

### Environment Variables

Set these in your app to enable telemetry:

```bash
# Enable OpenTelemetry
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
OTEL_SERVICE_NAME=agentkern-server
OTEL_TRACES_SAMPLER=parentbased_traceidratio
OTEL_TRACES_SAMPLER_ARG=0.1  # 10% sampling in prod
```

### Server Integration

The unified Rust server exports telemetry when configured with the OTEL variables above.

## Grafana Dashboards

Pre-configured dashboards in `grafana/provisioning/dashboards/`:

| Dashboard | Description |
|-----------|-------------|
| AgentKern Overview | Service health, latency, errors |
| Pillar Performance | Per-pillar request metrics |
| Agent Activity | Agent registration, message flow |

### Importing Dashboards

Dashboards auto-provision on startup. For manual import:

1. Open Grafana → Dashboards → Import
2. Use dashboard JSON from `grafana/dashboards/`

## Prometheus Queries

Example PromQL queries:

```promql
# Request rate by pillar
sum(rate(http_requests_total[5m])) by (pillar)

# 99th percentile latency
histogram_quantile(0.99, sum(rate(http_request_duration_seconds_bucket[5m])) by (le))

# Error rate
sum(rate(http_requests_total{status=~"5.."}[5m])) / sum(rate(http_requests_total[5m]))
```

## Alerting

Configure alerts in `prometheus.yml`:

```yaml
alerting:
  alertmanagers:
    - static_configs:
        - targets: ['alertmanager:9093']

rule_files:
  - '/etc/prometheus/alerts/*.yml'
```

## Production Considerations

1. **Sampling**: Set `OTEL_TRACES_SAMPLER_ARG=0.1` (10%) to reduce volume
2. **Retention**: Prometheus default 15d; adjust `--storage.tsdb.retention.time`
3. **Security**: Add auth proxy in front of Grafana/Prometheus
4. **Scaling**: Use Thanos or Cortex for HA Prometheus

## Troubleshooting

```bash
# Check OTEL collector logs
docker logs otel-collector

# Verify Prometheus targets
curl http://localhost:9090/api/v1/targets

# Test trace export
# Make request to app, then check Tempo UI
```
