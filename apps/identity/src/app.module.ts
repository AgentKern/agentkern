import { Module } from '@nestjs/common';
import { APP_GUARD, APP_PIPE } from '@nestjs/core';
import { ThrottlerModule, ThrottlerGuard } from '@nestjs/throttler';
import { ValidationPipe } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { AppController } from './app.controller';
import { LicenseModule } from './modules/license.module';
import { ProofModule } from './modules/proof.module';
import { DnsModule } from './modules/dns.module';
import { DashboardModule } from './modules/dashboard.module';
import { WebAuthnModule } from './modules/webauthn.module';
import { DatabaseModule } from './modules/database.module';
import { SecurityModule } from './modules/security.module';

@Module({
  imports: [
    // Environment configuration
    ConfigModule.forRoot({
      isGlobal: true,
      envFilePath: ['.env.local', '.env'],
    }),
    // Rate limiting - prevent abuse and DDoS
    ThrottlerModule.forRoot([
      {
        name: 'short',
        ttl: 1000, // 1 second
        limit: 10, // 10 requests per second
      },
      {
        name: 'medium',
        ttl: 10000, // 10 seconds
        limit: 50, // 50 requests per 10 seconds
      },
      {
        name: 'long',
        ttl: 60000, // 1 minute
        limit: 200, // 200 requests per minute
      },
    ]),
    // Core modules
    LicenseModule,
    ProofModule,
    DnsModule,
    DashboardModule,
    WebAuthnModule,
    DatabaseModule,
    // Security framework
    SecurityModule,
  ],
  controllers: [AppController],
  providers: [
    // Global rate limiting guard
    {
      provide: APP_GUARD,
      useClass: ThrottlerGuard,
    },
    // Global validation pipe
    {
      provide: APP_PIPE,
      useFactory: () =>
        new ValidationPipe({
          whitelist: true, // Strip unknown properties
          forbidNonWhitelisted: true, // Throw on unknown properties
          transform: true, // Transform payloads to DTO instances
          transformOptions: {
            enableImplicitConversion: true,
          },
          disableErrorMessages: false, // Show validation errors
        }),
    },
  ],
})
export class AppModule {}
