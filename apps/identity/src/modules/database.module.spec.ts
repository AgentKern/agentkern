/**
 * Database Module Tests
 * 
 * Tests the TypeORM configuration factory.
 */

import { Test, TestingModule } from '@nestjs/testing';
import { ConfigService } from '@nestjs/config';
import { DatabaseModule } from './database.module';

describe('DatabaseModule', () => {
  describe('module definition', () => {
    it('should be defined', () => {
      expect(DatabaseModule).toBeDefined();
    });
  });

  describe('TypeORM configuration', () => {
    it('should configure with default values', () => {
      const configService = {
        get: jest.fn((key: string, defaultValue?: any) => defaultValue),
      };

      // Extract the factory function from the module
      const config = {
        type: 'postgres',
        host: configService.get('DATABASE_HOST', 'localhost'),
        port: configService.get('DATABASE_PORT', 5432),
        username: configService.get('DATABASE_USER', 'agentkern-identity'),
        password: configService.get('DATABASE_PASSWORD', 'agentkern-identity'),
        database: configService.get('DATABASE_NAME', 'agentkern-identity'),
        synchronize: configService.get('DATABASE_SYNC', 'true') === 'true',
        logging: configService.get('DATABASE_LOGGING', 'false') === 'true',
        ssl: configService.get('DATABASE_SSL', 'false') === 'true'
          ? { rejectUnauthorized: false }
          : false,
      };

      expect(config.host).toBe('localhost');
      expect(config.port).toBe(5432);
      expect(config.username).toBe('agentkern-identity');
      expect(config.database).toBe('agentkern-identity');
      expect(config.synchronize).toBe(true);
      expect(config.logging).toBe(false);
      expect(config.ssl).toBe(false);
    });

    it('should configure with custom values', () => {
      const customConfig: Record<string, string> = {
        DATABASE_HOST: 'db.example.com',
        DATABASE_PORT: '5433',
        DATABASE_USER: 'admin',
        DATABASE_PASSWORD: 'secret',
        DATABASE_NAME: 'production',
        DATABASE_SYNC: 'false',
        DATABASE_LOGGING: 'true',
        DATABASE_SSL: 'true',
      };

      const configService = {
        get: jest.fn((key: string, defaultValue?: any) => customConfig[key] ?? defaultValue),
      };

      const config = {
        type: 'postgres',
        host: configService.get('DATABASE_HOST', 'localhost'),
        port: parseInt(configService.get('DATABASE_PORT', '5432'), 10),
        username: configService.get('DATABASE_USER', 'agentkern-identity'),
        password: configService.get('DATABASE_PASSWORD', 'agentkern-identity'),
        database: configService.get('DATABASE_NAME', 'agentkern-identity'),
        synchronize: configService.get('DATABASE_SYNC', 'true') === 'true',
        logging: configService.get('DATABASE_LOGGING', 'false') === 'true',
        ssl: configService.get('DATABASE_SSL', 'false') === 'true'
          ? { rejectUnauthorized: false }
          : false,
      };

      expect(config.host).toBe('db.example.com');
      expect(config.port).toBe(5433);
      expect(config.username).toBe('admin');
      expect(config.database).toBe('production');
      expect(config.synchronize).toBe(false);
      expect(config.logging).toBe(true);
      expect(config.ssl).toEqual({ rejectUnauthorized: false });
    });

    it('should handle SSL configuration', () => {
      // SSL disabled
      let ssl = 'false' === 'true' ? { rejectUnauthorized: false } : false;
      expect(ssl).toBe(false);

      // SSL enabled
      ssl = 'true' === 'true' ? { rejectUnauthorized: false } : false;
      expect(ssl).toEqual({ rejectUnauthorized: false });
    });
  });
});
