/**
 * AgentKernIdentity - Main Entry Point
 *
 * Bootstrap the NestJS application with:
 * - OpenTelemetry instrumentation (MUST be first import!)
 * - Structured JSON logging (Pino)
 * - Swagger documentation
 * - CORS configuration
 * - Security headers
 * - Global validation
 *
 * Follows mandate: documentation, security, observability, production-ready.
 */

// CRITICAL: Import instrumentation FIRST before any other imports
import './instrumentation';

import { NestFactory } from '@nestjs/core';
import { SwaggerModule, DocumentBuilder } from '@nestjs/swagger';
import helmet from 'helmet';
import express from 'express';
import cookieParser from 'cookie-parser';
import { AppModule } from './app.module';
import { PinoLoggerService } from './logging/pino-logger.service';
import { CorrelationIdMiddleware } from './middleware/correlation-id.middleware';

async function bootstrap() {
  // Use structured Pino logger
  const pinoLogger = new PinoLoggerService();
  pinoLogger.setContext('Bootstrap');

  const app = await NestFactory.create(AppModule, {
    logger: pinoLogger,
  });

  // Apply correlation ID middleware globally
  app.use(
    new CorrelationIdMiddleware().use.bind(new CorrelationIdMiddleware()),
  );

  // Parse cookies (REQUIRED for CSRF protection)
  app.use(cookieParser());

  // Security Headers (Helmet) with CSP Reporting
  app.use(
    helmet({
      contentSecurityPolicy: {
        directives: {
          defaultSrc: ["'self'"],
          scriptSrc: ["'self'"],
          styleSrc: ["'self'", "'unsafe-inline'"],
          imgSrc: ["'self'", 'data:', 'https:'],
          connectSrc: ["'self'"],
          frameSrc: ["'none'"],
          objectSrc: ["'none'"],
          upgradeInsecureRequests: [],
          // CSP Violation Reporting
          reportUri: '/api/v1/security/csp-report',
        },
        reportOnly: process.env.CSP_REPORT_ONLY === 'true', // Start with report-only mode
      },
      strictTransportSecurity: {
        maxAge: 31536000, // 1 year
        includeSubDomains: true,
        preload: true,
      },
      frameguard: { action: 'deny' },
      noSniff: true,
      xssFilter: true,
      hidePoweredBy: true,
      referrerPolicy: { policy: 'strict-origin-when-cross-origin' },
    }),
  );

  // Body Size Limits (DoS Protection)
  app.use(express.json({ limit: '100kb' }));
  app.use(express.urlencoded({ extended: true, limit: '100kb' }));

  // CORS Configuration (Security Hardening)
  // SECURITY: Require explicit CORS_ORIGINS in production to prevent wildcard exposure
  const corsOrigins = process.env.CORS_ORIGINS?.split(',').map((o) => o.trim());
  if (process.env.NODE_ENV === 'production' && !corsOrigins) {
    throw new Error(
      'SECURITY: CORS_ORIGINS must be set in production. ' +
        'Set to comma-separated list of allowed origins (e.g., "https://app.example.com,https://admin.example.com")',
    );
  }

  app.enableCors({
    origin: corsOrigins || (process.env.NODE_ENV === 'production' ? false : '*'),
    methods: ['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'OPTIONS'],
    allowedHeaders: ['Content-Type', 'Authorization', 'X-AgentKernIdentity', 'X-XSRF-TOKEN'],
    exposedHeaders: ['X-AgentKernIdentity'],
    credentials: true,
  });


  // Swagger documentation (disabled in production for security)
  if (process.env.NODE_ENV !== 'production' || process.env.ENABLE_SWAGGER_IN_PROD === 'true') {
    const config = new DocumentBuilder()
      .setTitle('AgentKernIdentity API')
      .setDescription(
        `**Liability Infrastructure for the Agentic Economy**

AgentKernIdentity provides cryptographic Liability Proofs that prove:
- A specific human authorized a specific AI agent action
- The authorization was made via a hardware-bound Passkey
- The authorizer explicitly accepts liability

## Key Features
- **Passkey-Bound**: Only device owner can authorize
- **Self-Verifying**: Target APIs verify locally – no latency
- **Liability Shift**: Cryptographic proof of who accepts responsibility
- **Universal**: Works for payments, data access, cloud ops, anything

## Authentication
Include the \`X-AgentKernIdentity\` header with your liability proof token.`,
      )
      .setVersion('1.0')
      .setContact(
        'AgentKern Team',
        'https://agentkern.io',
        'support@agentkern.io',
      )
      .setLicense('MIT', 'https://opensource.org/licenses/MIT')
      .setExternalDoc('Protocol Specification', '/docs/PROTOCOL_SPEC.md')
      .addTag('Proof', 'Create and verify liability proofs')
      .addTag('DNS', 'Trust resolution and registration')
      .addTag('Mesh', 'Decentralized trust network operations')
      .addTag('Dashboard', 'Analytics, policies, and compliance')
      .addTag('WebAuthn', 'Passkey registration and authentication')
      .addTag('Health', 'System health and status')
      .addApiKey(
        {
          type: 'apiKey',
          name: 'X-AgentKernIdentity',
          in: 'header',
          description: 'Liability proof token',
        },
        'AgentKernIdentity',
      )
      .addServer('http://localhost:3001', 'Local Development')
      .addServer('https://identity.agentkern.io', 'Production')
      .build();

    const document = SwaggerModule.createDocument(app, config);
    SwaggerModule.setup('docs', app, document, {
      swaggerOptions: {
        persistAuthorization: true,
        docExpansion: 'list',
        filter: true,
        showRequestDuration: true,
      },
      customSiteTitle: 'AgentKernIdentity API Documentation',
      customCss: '.swagger-ui .topbar { display: none }',
    });
  }

  // Start server
  const port = process.env.PORT || 3000;
  await app.listen(port);

  pinoLogger.log(
    `🚀 AgentKernIdentity API running on: http://localhost:${port}`,
  );
  pinoLogger.log(`📚 Swagger documentation: http://localhost:${port}/docs`);
  pinoLogger.log(`🔒 Liability Infrastructure for the Agentic Economy`);

  // Log observability status
  const otelEnabled =
    process.env.NODE_ENV === 'production' ||
    process.env.OTEL_ENABLED === 'true';
  if (otelEnabled) {
    pinoLogger.log(`📊 OpenTelemetry tracing enabled`);
  }
}

void bootstrap();
