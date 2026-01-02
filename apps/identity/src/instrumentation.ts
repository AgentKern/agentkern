/**
 * OpenTelemetry Instrumentation Setup
 *
 * This file MUST be imported FIRST in main.ts before any other imports
 * to ensure proper instrumentation of all modules.
 *
 * @see https://opentelemetry.io/docs/languages/js/getting-started/nodejs/
 */

import { NodeSDK } from '@opentelemetry/sdk-node';
import { getNodeAutoInstrumentations } from '@opentelemetry/auto-instrumentations-node';
import { OTLPTraceExporter } from '@opentelemetry/exporter-trace-otlp-proto';

const isProduction = process.env.NODE_ENV === 'production';
const otlpEndpoint =
  process.env.OTEL_EXPORTER_OTLP_ENDPOINT || 'http://localhost:4318';

// Configure SDK only if OTEL is enabled or in production
const otelEnabled = isProduction || process.env.OTEL_ENABLED === 'true';

let sdk: NodeSDK | null = null;

if (otelEnabled) {
  sdk = new NodeSDK({
    serviceName: 'agentkern-identity',
    traceExporter: new OTLPTraceExporter({
      url: `${otlpEndpoint}/v1/traces`,
    }),
    instrumentations: [
      getNodeAutoInstrumentations({
        // Disable fs instrumentation to reduce noise
        '@opentelemetry/instrumentation-fs': { enabled: false },
        // Enable HTTP/Express instrumentation
        '@opentelemetry/instrumentation-http': { enabled: true },
        '@opentelemetry/instrumentation-express': { enabled: true },
        '@opentelemetry/instrumentation-nestjs-core': { enabled: true },
        // Enable database instrumentation
        '@opentelemetry/instrumentation-pg': { enabled: true },
      }),
    ],
  });

  sdk.start();
  console.log('[OTEL] OpenTelemetry SDK started');
  console.log(`[OTEL] Exporting traces to: ${otlpEndpoint}/v1/traces`);
}

// Graceful shutdown
const shutdown = () => {
  if (sdk) {
    void sdk.shutdown().then(() => {
      console.log('[OTEL] OpenTelemetry SDK shut down gracefully');
    });
  }
};

process.on('SIGTERM', shutdown);
process.on('SIGINT', shutdown);

export { sdk };
