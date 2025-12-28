/**
 * AgentKern Identity - Main Entry Point
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

  // Security Headers (Helmet)
  app.use(helmet());

  // Body Size Limits (DoS Protection)
  app.use(require('express').json({ limit: '100kb' }));
  app.use(require('express').urlencoded({ extended: true, limit: '100kb' }));

  // Enable CORS for cross-origin requests
  app.enableCors({
    origin: process.env.CORS_ORIGINS?.split(',') || '*', // Restrict CORS origins to authorized domains from env
    methods: ['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'OPTIONS'],
    allowedHeaders: ['Content-Type', 'Authorization', 'X-AgentKern Identity'],
    exposedHeaders: ['X-AgentKern Identity'],
  });

  // Swagger documentation
  const config = new DocumentBuilder()
    .setTitle('AgentKern Identity API')
    .setDescription(
      `**Liability Infrastructure for the Agentic Economy**

AgentKern Identity provides cryptographic Liability Proofs that prove:
- A specific human authorized a specific AI agent action
- The authorization was made via a hardware-bound Passkey
- The authorizer explicitly accepts liability

## Key Features
- **Passkey-Bound**: Only device owner can authorize
- **Self-Verifying**: Target APIs verify locally – no latency
- **Liability Shift**: Cryptographic proof of who accepts responsibility
- **Universal**: Works for payments, data access, cloud ops, anything

## Authentication
Include the \`X-AgentKern Identity\` header with your liability proof token.`,
    )
    .setVersion('1.0')
    .setContact('AgentKern Identity Team', 'https://agentkern-identity.dev', 'support@agentkern-identity.dev')
    .setLicense('MIT', 'https://opensource.org/licenses/MIT')
    .setExternalDoc('Protocol Specification', '/docs/PROTOCOL_SPEC.md')
    .addTag('Proof', 'Create and verify liability proofs')
    .addTag('DNS', 'Trust resolution and registration')
    .addTag('Mesh', 'Decentralized trust network operations')
    .addTag('Dashboard', 'Analytics, policies, and compliance')
    .addTag('WebAuthn', 'Passkey registration and authentication')
    .addTag('Health', 'System health and status')
    .addApiKey(
      { type: 'apiKey', name: 'X-AgentKern Identity', in: 'header', description: 'Liability proof token' },
      'AgentKern Identity',
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
    customSiteTitle: 'AgentKern Identity API Documentation',
    customCss: '.swagger-ui .topbar { display: none }',
  });

  // Start server
  const port = process.env.PORT || 3000;
  await app.listen(port);

  logger.log(`🚀 AgentKern Identity API running on: http://localhost:${port}`);
  logger.log(`📚 Swagger documentation: http://localhost:${port}/docs`);
  logger.log(`🔒 Liability Infrastructure for the Agentic Economy`);
}

bootstrap();
