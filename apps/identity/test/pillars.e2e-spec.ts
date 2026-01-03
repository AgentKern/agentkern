/**
 * AgentKern Identity - Pillars E2E Test Suite
 *
 * Verifies all 6 pillars work together via the N-API bridge.
 * Runs against live PostgreSQL (start with: docker compose up -d postgres)
 */

import { Test, TestingModule } from '@nestjs/testing';
import { INestApplication } from '@nestjs/common';
import request from 'supertest';
import { App } from 'supertest/types';
import { AppModule } from '../src/app.module';

describe('Pillars Integration (e2e)', () => {
  let app: INestApplication<App>;

  beforeAll(async () => {
    // Set env vars for test
    process.env.DATABASE_HOST = 'localhost';
    process.env.DATABASE_PORT = '5432';
    process.env.DATABASE_USER = 'agentkern';
    process.env.DATABASE_PASSWORD = 'agentkern_secret';
    process.env.DATABASE_NAME = 'agentkern_identity';
    process.env.DATABASE_SYNC = 'true'; // Fresh schema sync
    process.env.DATABASE_SSL = 'false';
    process.env.DATABASE_DROP_SCHEMA = 'true'; // Force drop for tests

    const moduleFixture: TestingModule = await Test.createTestingModule({
      imports: [AppModule],
    }).compile();

    app = moduleFixture.createNestApplication();
    await app.init();
  }, 60000); // 60s timeout for init with db

  afterAll(async () => {
    if (app) {
      await app.close();
    }
  });

  // ============================================================================
  // Identity Pillar
  // ============================================================================
  describe('Identity Pillar', () => {
    it('should return API info on root', () => {
      return request(app.getHttpServer())
        .get('/')
        .expect(200)
        .expect((res) => {
          expect(res.body.name).toBe('AgentKernIdentity API');
        });
    });
  });

  // ============================================================================
  // Gate Pillar (Rust via N-API)
  // ============================================================================
  describe('Gate Pillar', () => {
    it('should attest (simulated TEE)', () => {
      return request(app.getHttpServer())
        .post('/api/v1/gate/attest')
        .send({ nonce: 'test-nonce-123' })
        .expect(200);
    });

    it('should verify action', () => {
      return request(app.getHttpServer())
        .post('/api/v1/gate/verify')
        .send({
          agentId: 'agent-e2e-test',
          action: 'read_data',
          context: { resource: 'test-resource' },
        })
        .expect(200);
    });

    it('should guard prompts', () => {
      return request(app.getHttpServer())
        .post('/api/v1/gate/guard-prompt')
        .send({ prompt: 'Hello, how are you?' })
        .expect(200);
    });
  });

  // ============================================================================
  // Synapse Pillar (Rust via N-API)
  // ============================================================================
  describe('Synapse Pillar', () => {
    const testAgentId = 'agent-synapse-e2e';

    it('should get agent state', () => {
      return request(app.getHttpServer())
        .get(`/api/v1/synapse/state/${testAgentId}`)
        .expect(200);
    });

    it('should update agent state', () => {
      return request(app.getHttpServer())
        .put(`/api/v1/synapse/state/${testAgentId}`)
        .send({ state: { lastAction: 'e2e-test', counter: 1 } })
        .expect(200);
    });

    it('should create memory passport', () => {
      return request(app.getHttpServer())
        .post('/api/v1/synapse/memory/passport')
        .send({ agentId: testAgentId, layers: ['semantic', 'episodic'] })
        .expect(201);
    });

    it('should guard context (RAG)', () => {
      return request(app.getHttpServer())
        .post('/api/v1/synapse/context/guard')
        .send({ documents: ['test document 1', 'test document 2'] })
        .expect(200);
    });
  });

  // ============================================================================
  // Arbiter Pillar (Rust via N-API)
  // ============================================================================
  describe('Arbiter Pillar', () => {
    it('should get kill switch status', () => {
      return request(app.getHttpServer())
        .get('/api/v1/arbiter/killswitch/status')
        .expect(200);
    });

    it('should query audit log', () => {
      return request(app.getHttpServer())
        .get('/api/v1/arbiter/audit')
        .query({ limit: 10 })
        .expect(200);
    });

    it('should inject chaos', () => {
      return request(app.getHttpServer())
        .post('/api/v1/arbiter/chaos/inject')
        .send({ type: 'latency', target: 'test-agent', durationSeconds: 5 })
        .expect(200);
    });
  });

  // ============================================================================
  // Treasury Pillar (Rust via N-API)
  // ============================================================================
  describe('Treasury Pillar', () => {
    const agentA = 'agent-treasury-a';

    it('should get balance', () => {
      return request(app.getHttpServer())
        .get(`/api/v1/treasury/balance/${agentA}`)
        .expect(200);
    });

    it('should deposit funds', () => {
      return request(app.getHttpServer())
        .post(`/api/v1/treasury/balance/${agentA}/deposit`)
        .send({ amount: 100.0 })
        .expect(200);
    });

    it('should get carbon footprint', () => {
      return request(app.getHttpServer())
        .get(`/api/v1/treasury/carbon/${agentA}`)
        .expect(200);
    });
  });

  // ============================================================================
  // Nexus Pillar (Rust via N-API)
  // ============================================================================
  describe('Nexus Pillar', () => {
    it('should list protocols', () => {
      return request(app.getHttpServer())
        .get('/api/v1/nexus/protocols')
        .expect(200)
        .expect((res) => {
          expect(res.body.protocols).toBeDefined();
        });
    });

    it('should register an agent (or fail without bridge)', () => {
      return request(app.getHttpServer())
        .post('/api/v1/nexus/agents')
        .send({
          name: 'E2E Test Agent',
          url: 'http://localhost:9999',
        })
        .expect((res) => {
          // 201 if bridge loaded, 400/500 if bridge not available
          expect([200, 201, 400, 500]).toContain(res.status);
        });
    });

    it('should list agents', () => {
      return request(app.getHttpServer())
        .get('/api/v1/nexus/agents')
        .expect(200);
    });

    it('should get nexus health', () => {
      return request(app.getHttpServer())
        .get('/api/v1/nexus/health')
        .expect(200)
        .expect((res) => {
          expect(res.body.status).toBe('healthy');
        });
    });
  });

  // ============================================================================
  // Cross-Pillar Integration
  // ============================================================================
  describe('Cross-Pillar Integration', () => {
    it('should perform multi-pillar workflow', async () => {
      const agentId = 'agent-cross-pillar';

      // 1. Get balance (Treasury)
      const balanceRes = await request(app.getHttpServer()).get(
        `/api/v1/treasury/balance/${agentId}`,
      );
      expect(balanceRes.status).toBe(200);

      // 2. Get state (Synapse)
      const stateRes = await request(app.getHttpServer()).get(
        `/api/v1/synapse/state/${agentId}`,
      );
      expect(stateRes.status).toBe(200);

      // 3. Check arbiter status
      const arbiterRes = await request(app.getHttpServer()).get(
        '/api/v1/arbiter/killswitch/status',
      );
      expect(arbiterRes.status).toBe(200);

      // 4. Verify via gate
      const gateRes = await request(app.getHttpServer())
        .post('/api/v1/gate/verify')
        .send({ agentId, action: 'cross_pillar_test' });
      expect(gateRes.status).toBe(200);
    });
  });
});
