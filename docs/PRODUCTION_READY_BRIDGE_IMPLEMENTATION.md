# Production-Ready Bridge Implementation Plan

**Date**: 2026-01-03  
**Status**: Implementation Guide  
**Goal**: Make TypeScript services production-ready by ensuring N-API bridge is always available

---

## Current State Analysis

### ✅ What's Working
- **Bridge Implementation**: Fully implemented in `packages/foundation/bridge/src/lib.rs`
- **Rust Integration**: Bridge correctly calls all Rust pillar implementations
- **TypeScript Services**: Services attempt to load bridge on startup
- **Error Handling**: Services have fallback modes when bridge unavailable

### ❌ What's Missing (Production Gaps)
1. **Build Process**: Bridge not automatically built during app build
2. **Fail-Fast**: Services continue in degraded mode instead of failing
3. **Health Checks**: No verification that bridge is operational
4. **CI/CD Integration**: Bridge build not part of deployment pipeline
5. **Path Resolution**: Bridge path may be incorrect in production builds

---

## Production-Ready Implementation Strategy

### Phase 1: Build Integration (Critical)

#### 1.1 Add Bridge Build to Identity App Build Process

**File**: `apps/identity/package.json`

```json
{
  "scripts": {
    "prebuild": "cd ../../packages/foundation/bridge && pnpm build",
    "build": "nest build",
    "postbuild": "node scripts/verify-bridge.js"
  }
}
```

**Rationale**: Ensures bridge is built before TypeScript compilation.

#### 1.2 Create Bridge Verification Script

**File**: `apps/identity/scripts/verify-bridge.js`

```javascript
const fs = require('fs');
const path = require('path');

const bridgePath = path.resolve(
  __dirname,
  '../../packages/foundation/bridge/index.node'
);

if (!fs.existsSync(bridgePath)) {
  console.error('❌ CRITICAL: N-API bridge not found at:', bridgePath);
  console.error('   Run: cd packages/foundation/bridge && pnpm build');
  process.exit(1);
}

console.log('✅ N-API bridge verified:', bridgePath);
```

**Rationale**: Fail-fast if bridge is missing.

---

### Phase 2: Production Configuration (Critical)

#### 2.1 Make Bridge Loading Mandatory in Production

**File**: `apps/identity/src/services/gate.service.ts`

```typescript
async onModuleInit(): Promise<void> {
  await Promise.resolve();
  
  const isProduction = process.env.NODE_ENV === 'production';
  const bridgePath = this.resolveBridgePath();
  
  try {
    this.bridge = require(bridgePath) as NativeBridge;
    this.bridgeLoaded = true;
    this.logger.log('🌉 N-API Bridge loaded successfully');
    
    // Verify bridge is operational
    await this.verifyBridge();
    
    // Initial policy sync
    this.syncPolicies().catch((e) =>
      this.logger.error(`Initial policy sync failed: ${e}`),
    );
  } catch (error: unknown) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    
    if (isProduction) {
      // PRODUCTION: Fail-fast - bridge is mandatory
      this.logger.error(
        `🚨 CRITICAL: Failed to load N-API bridge in production: ${errorMessage}`,
      );
      this.logger.error('🚨 Application cannot start without bridge');
      throw new Error(
        `N-API bridge is required in production but failed to load: ${errorMessage}`,
      );
    } else {
      // DEVELOPMENT: Allow degraded mode with warnings
      this.logger.error(
        `🚨 SECURITY DEGRADATION: Failed to load N-API bridge: ${errorMessage}`,
      );
      this.logger.warn(
        '⚠️ GateService will operate in FAIL-CLOSED mode (blocking all prompts)',
      );
      this.logger.warn(
        '⚠️ To fix: cd packages/foundation/bridge && pnpm build',
      );
    }
  }
}

/**
 * Resolve bridge path with proper error handling
 */
private resolveBridgePath(): string {
  // Try multiple possible paths (development vs production)
  const possiblePaths = [
    // Development: from source
    path.resolve(__dirname, '../../../../packages/foundation/bridge/index.node'),
    // Production: from dist (after build)
    path.resolve(__dirname, '../../../packages/foundation/bridge/index.node'),
    // Docker/container: absolute path
    '/app/packages/foundation/bridge/index.node',
  ];

  for (const bridgePath of possiblePaths) {
    if (fs.existsSync(bridgePath)) {
      return bridgePath;
    }
  }

  throw new Error(
    `Bridge not found in any expected location: ${possiblePaths.join(', ')}`,
  );
}

/**
 * Verify bridge is operational by calling a test function
 */
private async verifyBridge(): Promise<void> {
  try {
    // Test with a simple call
    const testResult = this.bridge.guardPrompt('test');
    if (!testResult) {
      throw new Error('Bridge returned null for test call');
    }
    this.logger.log('✅ Bridge verification successful');
  } catch (error: unknown) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    throw new Error(`Bridge verification failed: ${errorMessage}`);
  }
}
```

**Rationale**: 
- Production: Fail-fast if bridge unavailable (security requirement)
- Development: Allow degraded mode with clear warnings
- Verification: Ensure bridge actually works, not just loads

#### 2.2 Apply Same Pattern to All Services

Apply the same production-ready pattern to:
- `synapse.service.ts`
- `arbiter.service.ts`
- `nexus.service.ts`
- `treasury.service.ts`

---

### Phase 3: Health Checks & Monitoring

#### 3.1 Add Bridge Health Endpoint

**File**: `apps/identity/src/controllers/health.controller.ts`

```typescript
@Get('bridge')
@ApiOperation({ summary: 'Check N-API bridge health' })
async checkBridgeHealth(): Promise<{
  status: 'healthy' | 'degraded' | 'unavailable';
  services: {
    gate: boolean;
    synapse: boolean;
    arbiter: boolean;
    nexus: boolean;
    treasury: boolean;
  };
}> {
  return {
    status: this.determineOverallStatus(),
    services: {
      gate: this.gateService.isOperational(),
      synapse: this.synapseService.isOperational(),
      arbiter: this.arbiterService.isOperational(),
      nexus: this.nexusService.isOperational(),
      treasury: this.treasuryService.isOperational(),
    },
  };
}
```

**Rationale**: Allows monitoring/alerting to detect bridge failures.

#### 3.2 Add Startup Health Check

**File**: `apps/identity/src/main.ts`

```typescript
async function bootstrap() {
  const app = await NestFactory.create(AppModule);
  
  // ... existing setup ...
  
  // Verify bridge before starting server
  const gateService = app.get(GateService);
  if (!gateService.isOperational()) {
    if (process.env.NODE_ENV === 'production') {
      throw new Error('Cannot start in production without operational bridge');
    }
    app.get(Logger).warn('⚠️ Starting in degraded mode - bridge unavailable');
  }
  
  await app.listen(3000);
}
```

---

### Phase 4: CI/CD Integration

#### 4.1 Update GitHub Actions Workflow

**File**: `.github/workflows/ci.yml`

```yaml
identity-test:
  steps:
    - name: Build N-API Bridge
      working-directory: packages/foundation/bridge
      run: |
        pnpm install
        pnpm build
    
    - name: Verify Bridge Exists
      run: |
        test -f packages/foundation/bridge/index.node || (echo "Bridge build failed" && exit 1)
    
    - name: Install dependencies
      run: pnpm install --frozen-lockfile
    
    - name: Run tests
      working-directory: apps/identity
      run: pnpm test:cov
```

#### 4.2 Update Dockerfile

**File**: `apps/identity/Dockerfile`

```dockerfile
# Build N-API bridge
WORKDIR /app/packages/foundation/bridge
RUN pnpm install && pnpm build

# Verify bridge exists
RUN test -f index.node || (echo "Bridge build failed" && exit 1)

# Build Identity app
WORKDIR /app/apps/identity
RUN pnpm install && pnpm build

# Verify bridge is accessible from dist
RUN test -f ../../packages/foundation/bridge/index.node || \
    (echo "Bridge not found in expected location" && exit 1)
```

---

### Phase 5: Remove Fallback Modes (Production)

#### 5.1 Update Service Methods

**Pattern**: Remove fallback logic, throw errors instead

**Before**:
```typescript
guardPrompt(prompt: string): PromptAnalysis | null {
  if (!this.bridgeLoaded) {
    this.logger.warn('Bridge not loaded, returning null');
    return null; // Fallback
  }
  // ...
}
```

**After**:
```typescript
guardPrompt(prompt: string): PromptAnalysis {
  if (!this.bridgeLoaded) {
    throw new Error(
      'N-API bridge is required but not loaded. This is a critical failure.',
    );
  }
  // ...
}
```

**Rationale**: Fail-fast is safer than silent degradation.

---

### Phase 6: Integration Testing

#### 6.1 Add Bridge Integration Tests

**File**: `apps/identity/test/bridge.integration.spec.ts`

```typescript
describe('N-API Bridge Integration', () => {
  let gateService: GateService;
  let app: INestApplication;

  beforeAll(async () => {
    const module = await Test.createTestingModule({
      imports: [AppModule],
    }).compile();

    app = module.createNestApplication();
    await app.init();

    gateService = app.get(GateService);
  });

  it('should load bridge successfully', () => {
    expect(gateService.isOperational()).toBe(true);
  });

  it('should call Rust prompt guard', () => {
    const result = gateService.guardPrompt('test prompt');
    expect(result).toBeDefined();
    expect(result.threat_level).toBeDefined();
  });

  it('should call Rust gate engine', async () => {
    const result = await gateService.verify('agent-1', 'read', {});
    expect(result).toBeDefined();
    expect(result.allowed).toBeDefined();
  });
});
```

---

## Implementation Checklist

### Critical (Must Have for Production)
- [ ] Add bridge build to `package.json` scripts
- [ ] Create bridge verification script
- [ ] Make bridge loading mandatory in production (fail-fast)
- [ ] Update all services with production-ready error handling
- [ ] Add bridge health check endpoint
- [ ] Update CI/CD to build bridge
- [ ] Update Dockerfile to build and verify bridge

### High Priority (Should Have)
- [ ] Add integration tests for bridge
- [ ] Add monitoring/alerting for bridge health
- [ ] Document bridge troubleshooting guide
- [ ] Add bridge version checking

### Medium Priority (Nice to Have)
- [ ] Add bridge hot-reload capability (development)
- [ ] Add bridge performance metrics
- [ ] Add bridge connection pooling (if needed)

---

## Migration Path

### Step 1: Build Integration (This Week)
1. Add build scripts
2. Test locally
3. Verify CI/CD

### Step 2: Production Configuration (This Week)
1. Update service error handling
2. Add health checks
3. Test fail-fast behavior

### Step 3: Remove Fallbacks (Next Week)
1. Update all service methods
2. Add integration tests
3. Deploy to staging

### Step 4: Monitoring (Next Week)
1. Add health check endpoints
2. Set up alerts
3. Document runbooks

---

## Error Handling Strategy

### Production Environment
```typescript
if (!this.bridgeLoaded) {
  // CRITICAL: Fail-fast in production
  throw new Error('N-API bridge is required but unavailable');
}
```

### Development Environment
```typescript
if (!this.bridgeLoaded) {
  // WARNING: Allow degraded mode in development
  this.logger.warn('⚠️ Bridge unavailable - operating in degraded mode');
  this.logger.warn('⚠️ To fix: cd packages/foundation/bridge && pnpm build');
  // Return safe defaults or throw based on criticality
}
```

### Test Environment
```typescript
if (!this.bridgeLoaded) {
  // TESTS: Allow mocks or skip tests
  if (process.env.SKIP_BRIDGE_TESTS === 'true') {
    return; // Skip test
  }
  throw new Error('Bridge required for integration tests');
}
```

---

## Verification

### Local Development
```bash
# 1. Build bridge
cd packages/foundation/bridge && pnpm build

# 2. Verify bridge exists
test -f index.node && echo "✅ Bridge built" || echo "❌ Bridge missing"

# 3. Start identity app
cd ../../apps/identity && pnpm start:dev

# 4. Check logs for bridge loading
# Should see: "🌉 N-API Bridge loaded successfully"
```

### Production Deployment
```bash
# 1. Build bridge in CI/CD
pnpm --filter @agentkern/bridge build

# 2. Verify in Dockerfile
RUN test -f packages/foundation/bridge/index.node

# 3. Health check
curl http://localhost:3000/api/v1/health/bridge
# Should return: {"status": "healthy", "services": {...}}
```

---

## Troubleshooting Guide

### Issue: Bridge Not Found
**Symptoms**: `Cannot find module '../../../../packages/foundation/bridge/index.node'`

**Solutions**:
1. Build bridge: `cd packages/foundation/bridge && pnpm build`
2. Check path resolution in `resolveBridgePath()`
3. Verify bridge is in Docker image

### Issue: Bridge Loads But Methods Fail
**Symptoms**: Bridge loads but calls return errors

**Solutions**:
1. Check Rust compilation: `cargo check --workspace`
2. Verify N-API version compatibility
3. Check bridge logs for Rust errors

### Issue: Bridge Works Locally But Fails in Production
**Symptoms**: Works in dev, fails in Docker/K8s

**Solutions**:
1. Verify bridge is included in Docker image
2. Check file permissions on `index.node`
3. Verify architecture compatibility (x64 vs ARM)

---

## Success Criteria

✅ **Production-Ready When**:
1. Bridge builds automatically in CI/CD
2. Services fail-fast if bridge unavailable in production
3. Health checks verify bridge operational status
4. Integration tests verify bridge functionality
5. Monitoring alerts on bridge failures
6. Zero fallback modes in production code

---

**Next Steps**: Implement Phase 1 and Phase 2 immediately for production readiness.

