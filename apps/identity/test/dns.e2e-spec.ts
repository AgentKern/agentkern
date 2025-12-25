/**
 * DNS Controller E2E Tests
 */

import { Test, TestingModule } from '@nestjs/testing';
import { INestApplication, ValidationPipe } from '@nestjs/common';
import request from 'supertest';
import { AppModule } from './../src/app.module';

describe('DnsController (e2e)', () => {
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

  describe('POST /api/v1/dns/register', () => {
    it('should register trust relationship', async () => {
      const response = await request(app.getHttpServer())
        .post('/api/v1/dns/register')
        .send({
          agentId: 'agent-dns-test',
          principalId: 'principal-dns-test',
          agentName: 'DNS Test Agent',
          agentVersion: '1.0.0',
        })
        .expect(201);

      expect(response.body.agentId).toBe('agent-dns-test');
      expect(response.body.principalId).toBe('principal-dns-test');
      expect(response.body.trusted).toBe(true);
    });
  });

  describe('GET /api/v1/dns/resolve', () => {
    it('should resolve registered trust', async () => {
      // First register
      await request(app.getHttpServer())
        .post('/api/v1/dns/register')
        .send({
          agentId: 'agent-resolve-test',
          principalId: 'principal-resolve-test',
        });

      // Then resolve
      const response = await request(app.getHttpServer())
        .get('/api/v1/dns/resolve')
        .query({ agentId: 'agent-resolve-test', principalId: 'principal-resolve-test' })
        .expect(200);

      expect(response.body.trusted).toBe(true);
      expect(response.body.trustScore).toBeGreaterThan(0);
    });
  });

  describe('POST /api/v1/dns/revoke', () => {
    it('should revoke trust', async () => {
      // Register first
      await request(app.getHttpServer())
        .post('/api/v1/dns/register')
        .send({
          agentId: 'agent-revoke-test2',
          principalId: 'principal-revoke-test2',
        });

      // Revoke
      const response = await request(app.getHttpServer())
        .post('/api/v1/dns/revoke')
        .send({
          agentId: 'agent-revoke-test2',
          principalId: 'principal-revoke-test2',
          reason: 'Test revocation',
        })
        .expect(200);

      expect(response.body.revoked).toBe(true);
    });
  });

  describe('POST /api/v1/dns/reinstate', () => {
    it('should reinstate revoked trust', async () => {
      // Register and revoke
      await request(app.getHttpServer())
        .post('/api/v1/dns/register')
        .send({ agentId: 'agent-reinstate2', principalId: 'principal-reinstate2' });
      
      await request(app.getHttpServer())
        .post('/api/v1/dns/revoke')
        .send({
          agentId: 'agent-reinstate2',
          principalId: 'principal-reinstate2',
          reason: 'Temporary',
        });

      // Reinstate
      const response = await request(app.getHttpServer())
        .post('/api/v1/dns/reinstate')
        .send({
          agentId: 'agent-reinstate2',
          principalId: 'principal-reinstate2',
        })
        .expect(200);

      expect(response.body.revoked).toBe(false);
    });
  });

  describe('POST /api/v1/dns/resolve/batch', () => {
    it('should batch resolve multiple queries', async () => {
      // Register some agents
      await request(app.getHttpServer())
        .post('/api/v1/dns/register')
        .send({ agentId: 'batch-agent-1b', principalId: 'batch-principal-b' });
      await request(app.getHttpServer())
        .post('/api/v1/dns/register')
        .send({ agentId: 'batch-agent-2b', principalId: 'batch-principal-b' });

      const response = await request(app.getHttpServer())
        .post('/api/v1/dns/resolve/batch')
        .send({
          queries: [
            { agentId: 'batch-agent-1b', principalId: 'batch-principal-b' },
            { agentId: 'batch-agent-2b', principalId: 'batch-principal-b' },
          ],
        })
        .expect(200);

      expect(response.body.length).toBe(2);
      expect(response.body[0].trusted).toBe(true);
      expect(response.body[1].trusted).toBe(true);
    });
  });
});
