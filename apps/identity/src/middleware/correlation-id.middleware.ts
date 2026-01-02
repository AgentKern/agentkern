/**
 * Correlation ID Middleware
 *
 * Extracts or generates a correlation ID for each request and propagates it
 * through the request context for logging and tracing.
 *
 * Headers checked (in order):
 * - X-Correlation-ID
 * - X-Request-ID
 * - traceparent (W3C Trace Context)
 */

import { Injectable, NestMiddleware } from '@nestjs/common';
import { Request, Response, NextFunction } from 'express';
import { randomUUID } from 'crypto';
import { trace } from '@opentelemetry/api';

export const CORRELATION_ID_HEADER = 'X-Correlation-ID';
export const REQUEST_ID_HEADER = 'X-Request-ID';

// Extend Express Request to include correlation ID
declare global {
  // eslint-disable-next-line @typescript-eslint/no-namespace
  namespace Express {
    interface Request {
      correlationId: string;
    }
  }
}

@Injectable()
export class CorrelationIdMiddleware implements NestMiddleware {
  use(req: Request, res: Response, next: NextFunction): void {
    // Try to extract existing correlation ID from headers
    let correlationId =
      (req.headers[CORRELATION_ID_HEADER.toLowerCase()] as string) ||
      (req.headers[REQUEST_ID_HEADER.toLowerCase()] as string);

    // If no correlation ID, try to extract from active trace
    if (!correlationId) {
      const activeSpan = trace.getActiveSpan();
      if (activeSpan) {
        correlationId = activeSpan.spanContext().traceId;
      }
    }

    // Generate new UUID if still no correlation ID
    if (!correlationId) {
      correlationId = randomUUID();
    }

    // Attach to request object for downstream use
    req.correlationId = correlationId;

    // Set response headers for client correlation
    res.setHeader(CORRELATION_ID_HEADER, correlationId);
    res.setHeader(REQUEST_ID_HEADER, correlationId);

    // Add to current span attributes if tracing is active
    const currentSpan = trace.getActiveSpan();
    if (currentSpan) {
      currentSpan.setAttribute('correlation.id', correlationId);
      currentSpan.setAttribute(
        'http.request.header.x_correlation_id',
        correlationId,
      );
    }

    next();
  }
}

/**
 * Get correlation ID from current context
 * Utility function for use in services
 */
export function getCurrentCorrelationId(): string | undefined {
  const span = trace.getActiveSpan();
  if (span) {
    return span.spanContext().traceId;
  }
  return undefined;
}
