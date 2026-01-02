/**
 * OpenTelemetry Tracing Utilities
 *
 * Provides helpers for creating manual spans for inter-module calls.
 * Per the Epistemic Debt Audit: "Enable OpenTelemetry spans for every
 * cross-module service call."
 *
 * @see https://opentelemetry.io/docs/languages/js/instrumentation/
 */

import { trace, Span, SpanStatusCode } from '@opentelemetry/api';

// Get the tracer for AgentKern Identity services
const tracer = trace.getTracer('agentkern-identity', '1.0.0');

/**
 * Trace a synchronous function call.
 *
 * @param spanName - Name of the span (e.g., 'GateService.guardPrompt')
 * @param fn - Function to execute within the span
 * @param attributes - Optional span attributes
 */
export function withSpan<T>(
  spanName: string,
  fn: (span: Span) => T,
  attributes?: Record<string, string | number | boolean>,
): T {
  return tracer.startActiveSpan(spanName, (span) => {
    if (attributes) {
      span.setAttributes(attributes);
    }

    try {
      const result = fn(span);
      span.setStatus({ code: SpanStatusCode.OK });
      return result;
    } catch (error) {
      span.setStatus({
        code: SpanStatusCode.ERROR,
        message: error instanceof Error ? error.message : String(error),
      });
      span.recordException(error as Error);
      throw error;
    } finally {
      span.end();
    }
  });
}

/**
 * Trace an asynchronous function call.
 *
 * @param spanName - Name of the span (e.g., 'GateService.verify')
 * @param fn - Async function to execute within the span
 * @param attributes - Optional span attributes
 */
export async function withSpanAsync<T>(
  spanName: string,
  fn: (span: Span) => Promise<T>,
  attributes?: Record<string, string | number | boolean>,
): Promise<T> {
  return tracer.startActiveSpan(spanName, async (span) => {
    if (attributes) {
      span.setAttributes(attributes);
    }

    try {
      const result = await fn(span);
      span.setStatus({ code: SpanStatusCode.OK });
      return result;
    } catch (error) {
      span.setStatus({
        code: SpanStatusCode.ERROR,
        message: error instanceof Error ? error.message : String(error),
      });
      span.recordException(error as Error);
      throw error;
    } finally {
      span.end();
    }
  });
}

/**
 * Get the current trace context for propagation.
 */
export function getCurrentTraceContext(): {
  traceId: string;
  spanId: string;
} | null {
  const activeSpan = trace.getActiveSpan();
  if (!activeSpan) return null;

  const ctx = activeSpan.spanContext();
  return {
    traceId: ctx.traceId,
    spanId: ctx.spanId,
  };
}

/**
 * Add an event to the current span.
 */
export function addSpanEvent(
  name: string,
  attributes?: Record<string, string | number | boolean>,
): void {
  const activeSpan = trace.getActiveSpan();
  if (activeSpan) {
    activeSpan.addEvent(name, attributes);
  }
}
