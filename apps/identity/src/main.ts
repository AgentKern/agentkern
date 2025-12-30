/**
 * AgentKernIdentity - Main Entry Point
 * 
 * Bootstrap the NestJS application with:
 * - Swagger documentation
 * - CORS configuration
 * - Security headers
 * - Global validation
 * 
 * Follows mandate: documentation, security, production-ready.
 */

import { NestFactory } from '@nestjs/core';
import { ValidationPipe, Logger } from '@nestjs/common';
import { SwaggerModule, DocumentBuilder } from '@nestjs/swagger';
import helmet from 'helmet';
import { AppModule } from './app.module';

async function bootstrap() {
  const logger = new Logger('Bootstrap');
  const app = await NestFactory.create(AppModule);

  // Security Headers (Helmet) with CSP Reporting
  app.use(helmet({
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
  }));

  // Body Size Limits (DoS Protection)
  app.use(require('express').json({ limit: '100kb' }));
  app.use(require('express').urlencoded({ extended: true, limit: '100kb' }));

  // Enable CORS for cross-origin requests
  app.enableCors({
    origin: process.env.CORS_ORIGINS?.split(',') || '*', // Restrict CORS origins to authorized domains from env
    methods: ['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'OPTIONS'],
    allowedHeaders: ['Content-Type', 'Authorization', 'X-AgentKernIdentity'],
    exposedHeaders: ['X-AgentKernIdentity'],
  });

  // Swagger documentation
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
    .setContact('AgentKernIdentity Team', 'https://agentkern-identity.dev', 'support@agentkern-identity.dev')
    .setLicense('MIT', 'https://opensource.org/licenses/MIT')
    .setExternalDoc('Protocol Specification', '/docs/PROTOCOL_SPEC.md')
    .addTag('Proof', 'Create and verify liability proofs')
    .addTag('DNS', 'Trust resolution and registration')
    .addTag('Mesh', 'Decentralized trust network operations')
    .addTag('Dashboard', 'Analytics, policies, and compliance')
    .addTag('WebAuthn', 'Passkey registration and authentication')
    .addTag('Health', 'System health and status')
    .addApiKey(
      { type: 'apiKey', name: 'X-AgentKernIdentity', in: 'header', description: 'Liability proof token' },
      'AgentKernIdentity',
    )
    .addServer('http://localhost:3000', 'Local Development')
    .addServer('https://api.agentkern-identity.dev', 'Production')
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

  // Start server
  const port = process.env.PORT || 3000;
  await app.listen(port);

  logger.log(`🚀 AgentKernIdentity API running on: http://localhost:${port}`);
  logger.log(`📚 Swagger documentation: http://localhost:${port}/docs`);
  logger.log(`🔒 Liability Infrastructure for the Agentic Economy`);
}

bootstrap();
