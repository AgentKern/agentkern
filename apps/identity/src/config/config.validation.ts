/**
 * Configuration Validation Schema
 * 
 * Fail-fast validation for environment variables.
 * Prevents application startup with invalid or missing critical configuration.
 */

import * as Joi from 'joi';

export const configValidationSchema = Joi.object({
  // Server
  NODE_ENV: Joi.string()
    .valid('development', 'production', 'test')
    .default('development'),
  PORT: Joi.number().port().default(3000),

  // Database (required in production)
  DATABASE_URL: Joi.string()
    .uri({ scheme: ['postgres', 'postgresql'] })
    .when('NODE_ENV', {
      is: 'production',
      then: Joi.required(),
      otherwise: Joi.optional(),
    }),

  // Security
  CORS_ORIGINS: Joi.string().optional(),
  CSP_REPORT_ONLY: Joi.boolean().default(true),

  // Enterprise License (optional)
  AGENTKERN_LICENSE_KEY: Joi.string().optional(),

  // WebAuthn
  WEBAUTHN_RP_ID: Joi.string().default('localhost'),
  WEBAUTHN_RP_NAME: Joi.string().default('AgentKern Identity'),
  WEBAUTHN_ORIGIN: Joi.string().uri().default('http://localhost:3000'),
});

/**
 * Validation options for ConfigModule
 */
export const configValidationOptions = {
  abortEarly: false, // Show all validation errors, not just the first
  allowUnknown: true, // Allow env vars not in schema
};
