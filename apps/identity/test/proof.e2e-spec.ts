/**
 * Proof Controller E2E Tests
 */

import { Test, TestingModule } from '@nestjs/testing';
import { INestApplication, ValidationPipe } from '@nestjs/common';
import request from 'supertest';
import { AppModule } from './../src/app.module';

describe('ProofController (e2e)', () => {
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
  });

  afterAll(async () => {
    await app.close();
  });

  describe('GET /api/v1/proof/health', () => {
    it('should return health status', () => {
      return request(app.getHttpServer())
        .get('/api/v1/proof/health')
        .expect(200)
        .expect((res) => {
          expect(res.body.status).toBe('healthy');
        });
    });
  });

  describe('POST /api/v1/proof/create', () => {
    it('should create a signed proof', async () => {
      const response = await request(app.getHttpServer())
        .post('/api/v1/proof/create')
        .send({
          principal: {
            id: 'principal-e2e-test',
            credentialId: 'cred-e2e-test',
          },
          agent: {
            id: 'agent-e2e-test',
            name: 'E2E Test Agent',
            version: '1.0.0',
          },
          intent: {
            action: 'transfer',
            target: {
              service: 'bank',
              endpoint: '/transfer',
              method: 'POST',
            },
            parameters: { amount: 100 },
          },
        })
        .expect(201);

      expect(response.body.header).toBeDefined();
      expect(response.body.proofId).toBeDefined();
      expect(response.body.expiresAt).toBeDefined();
    });
  });

  describe('POST /api/v1/proof/register-key', () => {
    it('should register a public key', async () => {
      const response = await request(app.getHttpServer())
        .post('/api/v1/proof/register-key')
        .send({
          principalId: 'principal-key-test',
          publicKey: 'MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAE...',
          credentialId: 'cred-key-test',
        })
        .expect(201);

      expect(response.body.success).toBe(true);
    });
  });
});
