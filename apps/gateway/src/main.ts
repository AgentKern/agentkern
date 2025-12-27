/**
 * AgentKern Gateway - Main Entry Point
 * 
 * Per ARCHITECTURE.md: The Gateway routes requests through the Four Pillars.
 * Using Fastify for high-performance HTTP (per Hyper-Loop design).
 */

import { NestFactory } from '@nestjs/core';
import { FastifyAdapter, NestFastifyApplication } from '@nestjs/platform-fastify';
import { AppModule } from './app.module';

async function bootstrap() {
  const app = await NestFactory.create<NestFastifyApplication>(
    AppModule,
    new FastifyAdapter({ logger: true }),
  );

  // Global prefix for all routes
  app.setGlobalPrefix('api/v1');

  // Enable CORS for SDK clients
  app.enableCors({
    origin: true,
    credentials: true,
  });

  const port = process.env.PORT ?? 3000;
  await app.listen(port, '0.0.0.0');

  console.log(`🚀 AgentKern Gateway running on http://localhost:${port}`);
  console.log(`📖 API Docs: http://localhost:${port}/api/v1`);
}

bootstrap();
