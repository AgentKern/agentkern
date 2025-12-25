/**
 * Mesh Controller E2E Tests
 */

import { Test, TestingModule } from '@nestjs/testing';
import { INestApplication, ValidationPipe } from '@nestjs/common';
import request from 'supertest';
import { AppModule } from './../src/app.module';

describe('MeshController (e2e)', () => {
  let app: INestApplication;

  beforeAll(async () => {
    const moduleFixture: TestingModule = await Test.createTestingModule({
      imports: [AppModule],
    }).compile();

    app = moduleFixture.createNestApplication();
    app.useGlobalPipes(new ValidationPipe({
      whitelist: true,
      forbidNonWhitelisted: true,
      transform: true,
    }));
    await app.init();
  });

  afterAll(async () => {
    await app.close();
  });

  describe('GET /api/v1/mesh/node', () => {
    it('should return node info', () => {
      return request(app.getHttpServer())
        .get('/api/v1/mesh/node')
        .expect(200)
        .expect((res) => {
          expect(res.body.id).toBeDefined();
          expect(res.body.publicKey).toBeDefined();
          expect(res.body.type).toBeDefined();
        });
    });
  });

  describe('GET /api/v1/mesh/stats', () => {
    it('should return mesh stats', () => {
      return request(app.getHttpServer())
        .get('/api/v1/mesh/stats')
        .expect(200)
        .expect((res) => {
          expect(res.body.nodeId).toBeDefined();
          expect(res.body.connectedPeers).toBeDefined();
        });
    });
  });

  describe('GET /api/v1/mesh/peers', () => {
    it('should return connected peers', () => {
      return request(app.getHttpServer())
        .get('/api/v1/mesh/peers')
        .expect(200)
        .expect((res) => {
          expect(Array.isArray(res.body)).toBe(true);
        });
    });
  });

  describe('POST /api/v1/mesh/broadcast/trust', () => {
    it('should broadcast trust update', async () => {
      const response = await request(app.getHttpServer())
        .post('/api/v1/mesh/broadcast/trust')
        .send({
          agentId: 'agent-mesh-test',
          principalId: 'principal-mesh-test',
          trustScore: 750,  // number
          event: 'VERIFICATION_SUCCESS',  // string
        })
        .expect(200);

      expect(response.body.success).toBe(true);
    });
  });

  describe('POST /api/v1/mesh/broadcast/revocation', () => {
    it('should broadcast revocation', async () => {
      const response = await request(app.getHttpServer())
        .post('/api/v1/mesh/broadcast/revocation')
        .send({
          agentId: 'agent-revoke-mesh',
          principalId: 'principal-revoke-mesh',
          reason: 'Compromised',
        })
        .expect(200);

      expect(response.body.success).toBe(true);
    });
  });
});
