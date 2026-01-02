/**
 * Dashboard Controller E2E Tests
 * Tests enterprise features and license gating
 */

import { Test, TestingModule } from '@nestjs/testing';
import { INestApplication, ValidationPipe } from '@nestjs/common';
import request from 'supertest';
import { AppModule } from './../src/app.module';

describe('DashboardController (e2e)', () => {
  let app: INestApplication;

  beforeAll(async () => {
    const moduleFixture: TestingModule = await Test.createTestingModule({
      imports: [AppModule],
    }).compile();

    app = moduleFixture.createNestApplication();
    app.useGlobalPipes(
      new ValidationPipe({
        whitelist: true,
        forbidNonWhitelisted: true,
        transform: true,
      }),
    );
    await app.init();
  }, 30000);

  afterAll(async () => {
    await app.close();
  });

  describe('GET /api/v1/dashboard', () => {
    it('should return dashboard endpoints', () => {
      return request(app.getHttpServer())
        .get('/api/v1/dashboard')
        .expect(200)
        .expect((res) => {
          expect(res.body.name).toBe('AgentKernIdentity Dashboard API');
          expect(res.body.endpoints).toBeDefined();
        });
    });
  });

  describe('GET /api/v1/dashboard/stats (Enterprise)', () => {
    it('should return 403 without license', () => {
      return request(app.getHttpServer())
        .get('/api/v1/dashboard/stats')
        .expect(403)
        .expect((res) => {
          expect(res.body.error).toBe('Forbidden');
        });
    });
  });

  describe('GET /api/v1/dashboard/policies', () => {
    it('should return policies (allowed in OSS tier)', () => {
      return request(app.getHttpServer())
        .get('/api/v1/dashboard/policies')
        .expect(200)
        .expect((res) => {
          expect(Array.isArray(res.body)).toBe(true);
        });
    });
  });
});
