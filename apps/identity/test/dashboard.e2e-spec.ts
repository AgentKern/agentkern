/**
 * Dashboard Controller E2E Tests
 * Tests enterprise features and license gating
 */

import { Test, TestingModule } from '@nestjs/testing';
import { INestApplication, ValidationPipe } from '@nestjs/common';
import request from 'supertest';
import { AppModule } from './../src/app.module';
import {
  getBody,
  DashboardApiInfo,
  DashboardErrorResponse,
  getServer,
} from './test-types';

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
      return request(getServer(app))
        .get('/api/v1/dashboard')
        .expect(200)
        .expect((res) => {
          const body = getBody<DashboardApiInfo>(res);
          expect(body.name).toBe('AgentKernIdentity Dashboard API');
          expect(body.endpoints).toBeDefined();
        });
    });
  });

  describe('GET /api/v1/dashboard/stats (Enterprise)', () => {
    it('should return 403 without license', () => {
      return request(getServer(app))
        .get('/api/v1/dashboard/stats')
        .expect(403)
        .expect((res) => {
          const body = getBody<DashboardErrorResponse>(res);
          expect(body.error).toBe('Forbidden');
        });
    });
  });

  describe('GET /api/v1/dashboard/policies', () => {
    it('should return policies (allowed in OSS tier)', () => {
      return request(getServer(app))
        .get('/api/v1/dashboard/policies')
        .expect(200)
        .expect((res) => {
          const body = getBody<unknown[]>(res);
          expect(Array.isArray(body)).toBe(true);
        });
    });
  });
});
