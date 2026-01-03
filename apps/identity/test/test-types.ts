/**
 * Type-safe response types for E2E tests
 *
 * These types provide proper typing for supertest response bodies,
 * eliminating ESLint @typescript-eslint/no-unsafe-member-access errors.
 */

import { Response } from 'supertest';

/**
 * Type-safe response body accessor
 * Use this to properly type supertest response bodies
 */
export function getBody<T>(res: Response): T {
  return res.body as T;
}

// =========================================================================
// Health Check Response Types
// =========================================================================

export interface HealthResponse {
  name: string;
  version?: string;
  status?: string;
}

export interface BridgeHealthResponse {
  status: 'healthy' | 'degraded' | 'unavailable';
  services: {
    gate: boolean;
    synapse: boolean;
    arbiter: boolean;
    nexus: boolean;
    treasury: boolean;
  };
  timestamp: string;
}

// =========================================================================
// Dashboard Response Types
// =========================================================================

export interface DashboardApiInfo {
  name: string;
  endpoints: string[];
}

export interface DashboardErrorResponse {
  error: string;
  statusCode: number;
}

// =========================================================================
// DNS / Trust Response Types
// =========================================================================

export interface TrustRecordResponse {
  agentId: string;
  principalId: string;
  trusted: boolean;
  trustScore?: number;
  revoked?: boolean;
}

export interface TrustListResponse {
  length: number;
  [index: number]: TrustRecordResponse;
}

// =========================================================================
// Proof Response Types
// =========================================================================

export interface ProofStatusResponse {
  status: string;
}

export interface ProofCreationResponse {
  header: string;
  proofId: string;
  expiresAt: string;
}

export interface ProofSuccessResponse {
  success: boolean;
}

// =========================================================================
// Pillars Response Types
// =========================================================================

export interface PillarsHealthResponse {
  name: string;
}

export interface NexusProtocolsResponse {
  protocols: string[];
}

export interface ArbiterStatusResponse {
  status: string;
}

// =========================================================================
// Generic Error Response
// =========================================================================

export interface ErrorResponse {
  error: string;
  message?: string;
  statusCode?: number;
}

// =========================================================================
// HTTP Server Type Utility
// =========================================================================

import { INestApplication } from '@nestjs/common';
import { Server } from 'http';

/**
 * Type-safe server accessor for supertest
 * Eliminates @typescript-eslint/no-unsafe-argument warnings
 */
export function getServer(app: INestApplication): Server {
  return app.getHttpServer() as Server;
}
