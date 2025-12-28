import { Test, TestingModule } from '@nestjs/testing';
import { AppController } from './app.controller';

describe('AppController', () => {
  let appController: AppController;

  beforeEach(async () => {
    const app: TestingModule = await Test.createTestingModule({
      controllers: [AppController],
    }).compile();

    appController = app.get<AppController>(AppController);
  });

  describe('root', () => {
    it('should return API info', () => {
      const result = appController.getRoot();
      expect(result.name).toBe('AgentKernIdentity API');
      expect(result.version).toBe('1.0.0');
      expect(result.docs).toBe('/docs');
      expect(result.endpoints).toBeDefined();
    });
  });

  describe('health', () => {
    it('should return healthy status', () => {
      const result = appController.getHealth();
      expect(result.status).toBe('healthy');
      expect(result.timestamp).toBeDefined();
    });
  });
});
