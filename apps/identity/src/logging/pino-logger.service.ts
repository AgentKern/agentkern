/**
 * Structured JSON Logger using Pino
 * 
 * Provides structured logging with:
 * - JSON output for production
 * - Pretty printing for development
 * - Correlation ID support
 * - Request context propagation
 */

import pino, { Logger as PinoLogger } from 'pino';
import { Injectable, LoggerService, Scope } from '@nestjs/common';
import { trace, context } from '@opentelemetry/api';

// Custom log levels aligned with NestJS
const levels = {
  fatal: 60,
  error: 50,
  warn: 40,
  log: 30,  // NestJS 'log' = Pino 'info'
  debug: 20,
  verbose: 10,
};

// Base logger configuration
const baseLogger = pino({
  level: process.env.LOG_LEVEL || 'log',
  customLevels: levels,
  useOnlyCustomLevels: true,
  formatters: {
    level: (label) => ({ level: label }),
    bindings: () => ({}), // Remove pid/hostname from logs
  },
  timestamp: pino.stdTimeFunctions.isoTime,
  // Pretty print in development
  transport: process.env.NODE_ENV !== 'production' 
    ? {
        target: 'pino-pretty',
        options: {
          colorize: true,
          translateTime: 'SYS:standard',
          ignore: 'pid,hostname',
        },
      }
    : undefined,
});

/**
 * Get current trace context for log correlation
 */
function getTraceContext(): Record<string, string> {
  const span = trace.getActiveSpan();
  if (!span) return {};
  
  const spanContext = span.spanContext();
  return {
    traceId: spanContext.traceId,
    spanId: spanContext.spanId,
  };
}

/**
 * NestJS-compatible Logger Service using Pino
 */
@Injectable({ scope: Scope.TRANSIENT })
export class PinoLoggerService implements LoggerService {
  private logger: PinoLogger;
  private context?: string;

  constructor() {
    this.logger = baseLogger;
  }

  setContext(context: string) {
    this.context = context;
  }

  private formatMessage(message: any, optionalParams: any[]): Record<string, any> {
    const base: Record<string, any> = {
      ...getTraceContext(),
      context: this.context,
    };

    if (typeof message === 'object') {
      return { ...base, ...message };
    }

    base.message = optionalParams.length > 0 
      ? `${message} ${optionalParams.join(' ')}`
      : message;

    return base;
  }

  log(message: any, ...optionalParams: any[]) {
    this.logger.info(this.formatMessage(message, optionalParams));
  }

  error(message: any, ...optionalParams: any[]) {
    const logObj = this.formatMessage(message, optionalParams);
    // Extract stack trace if present
    if (optionalParams[0] instanceof Error) {
      logObj.err = {
        message: optionalParams[0].message,
        stack: optionalParams[0].stack,
        name: optionalParams[0].name,
      };
    }
    this.logger.error(logObj);
  }

  warn(message: any, ...optionalParams: any[]) {
    this.logger.warn(this.formatMessage(message, optionalParams));
  }

  debug(message: any, ...optionalParams: any[]) {
    this.logger.debug(this.formatMessage(message, optionalParams));
  }

  verbose(message: any, ...optionalParams: any[]) {
    this.logger.trace(this.formatMessage(message, optionalParams));
  }

  fatal(message: any, ...optionalParams: any[]) {
    this.logger.fatal(this.formatMessage(message, optionalParams));
  }
}

/**
 * Child logger factory for specific contexts
 */
export function createChildLogger(context: string): PinoLogger {
  return baseLogger.child({ context });
}

export { baseLogger as logger };
