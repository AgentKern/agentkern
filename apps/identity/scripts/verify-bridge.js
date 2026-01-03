#!/usr/bin/env node
/**
 * Verify N-API Bridge Script
 * 
 * Verifies that the N-API bridge exists and is accessible after build.
 * This ensures production deployments have the required native module.
 * 
 * Per MANDATE.md: Fail-fast on missing critical dependencies.
 */

const fs = require('fs');
const path = require('path');

// Possible bridge locations (development vs production)
const possiblePaths = [
  // Development: from source
  path.resolve(__dirname, '../../../packages/foundation/bridge/index.node'),
  // Production: from dist (after build)
  path.resolve(__dirname, '../../packages/foundation/bridge/index.node'),
  // Docker/container: absolute path
  '/app/packages/foundation/bridge/index.node',
];

console.log('🔍 Verifying N-API Bridge...');

let bridgeFound = false;
let bridgePath = null;

for (const testPath of possiblePaths) {
  if (fs.existsSync(testPath)) {
    bridgeFound = true;
    bridgePath = testPath;
    break;
  }
}

if (!bridgeFound) {
  console.error('❌ CRITICAL: N-API bridge not found in any expected location');
  console.error('');
  console.error('Expected locations:');
  possiblePaths.forEach((p) => console.error(`  - ${p}`));
  console.error('');
  console.error('To fix:');
  console.error('  cd packages/foundation/bridge');
  console.error('  pnpm install');
  console.error('  pnpm build');
  process.exit(1);
}

// Verify file is readable
try {
  fs.accessSync(bridgePath, fs.constants.R_OK);
  const stats = fs.statSync(bridgePath);
  console.log(`✅ N-API bridge verified: ${bridgePath}`);
  console.log(`   Size: ${(stats.size / 1024 / 1024).toFixed(2)} MB`);
} catch (error) {
  console.error(`❌ Bridge file exists but is not readable: ${bridgePath}`);
  console.error(`   Error: ${error.message}`);
  process.exit(1);
}

