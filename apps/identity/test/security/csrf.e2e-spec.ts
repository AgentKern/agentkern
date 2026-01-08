
import { Test, TestingModule } from '@nestjs/testing';
import { INestApplication, Controller, Post, Get, MiddlewareConsumer, Module, NestModule } from '@nestjs/common';
import request from 'supertest';
import { App } from 'supertest/types';
import { CsrfMiddleware } from '../../src/middleware/csrf.middleware';
import { getServer } from '../test-types';
import cookieParser from 'cookie-parser';

// ============================================================================
// Test Controller
// ============================================================================
@Controller('test-csrf')
class CsrfTestController {
  @Post()
  sensitiveAction() {
    return { status: 'success' };
  }

  @Get()
  safeAction() {
    return { status: 'safe' };
  }
}

// ============================================================================
// App Module with Middleware
// ============================================================================
@Module({
  controllers: [CsrfTestController],
})
class TestAppModule implements NestModule {
  configure(consumer: MiddlewareConsumer) {
    consumer
      .apply(CsrfMiddleware)
      .forRoutes(CsrfTestController);
  }
}

// ============================================================================
// CSRF Test Suite
// ============================================================================
describe('Security: CSRF (e2e)', () => {
  let app: INestApplication<App>;

  beforeAll(async () => {
    const moduleFixture: TestingModule = await Test.createTestingModule({
      imports: [TestAppModule],
    }).compile();

    app = moduleFixture.createNestApplication();
    app.use(cookieParser()); // Required to parse cookies
    await app.init();
  });

  afterAll(async () => {
    if (app) {
      await app.close();
    }
  });

  it('should allow GET requests without token', () => {
    return request(getServer(app))
      .get('/test-csrf')
      .expect(200);
  });

  it('should set CSRF cookie on GET request if missing', () => {
    return request(getServer(app))
      .get('/test-csrf')
      .expect(200)
      .expect('set-cookie', /XSRF-TOKEN=/);
  });

  it('should reject POST request without token', () => {
    return request(getServer(app))
      .post('/test-csrf')
      .expect(403);
  });

  it('should reject POST request with cookie but missing header', () => {
    return request(getServer(app))
      .post('/test-csrf')
      .set('Cookie', ['XSRF-TOKEN=test-token'])
      .expect(403);
  });

  it('should reject POST request with mismatched tokens', () => {
    return request(getServer(app))
      .post('/test-csrf')
      .set('Cookie', ['XSRF-TOKEN=token-a'])
      .set('X-XSRF-TOKEN', 'token-b')
      .expect(403);
  });

  it('should accept POST request with matching tokens', () => {
    const token = '12345678901234567890123456789012'; // 32 bytes hex length (64 chars actually, but middleware checks string equality)
    // Wait, middleware generates randomBytes(32).toString('hex') -> 64 chars.
    // Length check in safeCompare just checks a.length === b.length.
    // So any matching strings work.
    
    return request(getServer(app))
      .post('/test-csrf')
      .set('Cookie', [`XSRF-TOKEN=${token}`])
      .set('X-XSRF-TOKEN', token)
      .expect(201); // NestJS default POST success is 201
  });
  
  it('should rotate token after successful request', () => {
    const token = '12345678901234567890123456789012';
    
    return request(getServer(app))
      .post('/test-csrf')
      .set('Cookie', [`XSRF-TOKEN=${token}`])
      .set('X-XSRF-TOKEN', token)
      .expect(201)
      .expect('set-cookie', /XSRF-TOKEN=/); // Should set a NEW cookie
  });
});
