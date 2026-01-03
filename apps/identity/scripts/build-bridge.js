#!/usr/bin/env node
/**
 * Build N-API Bridge Script
 * 
 * Ensures the Rust N-API bridge is built before building the Identity app.
 * This is critical for production deployments.
 * 
 * Per MANDATE.md: Zero tolerance for missing dependencies.
 */

const { execSync } = require('child_process');
const path = require('path');
const fs = require('fs');

const bridgePath = path.resolve(__dirname, '../../../packages/foundation/bridge');
const bridgeNodeFile = path.join(bridgePath, 'index.node');

console.log('🔨 Building N-API Bridge...');
console.log(`   Path: ${bridgePath}`);

try {
  // Check if bridge directory exists
  if (!fs.existsSync(bridgePath)) {
    throw new Error(`Bridge directory not found: ${bridgePath}`);
  }

  // Check if already built (skip if exists and not in CI)
  if (fs.existsSync(bridgeNodeFile) && !process.env.CI) {
    console.log('✅ Bridge already built, skipping...');
    return;
  }

  // Build bridge
  console.log('   Running: pnpm build');
  execSync('pnpm build', {
    cwd: bridgePath,
    stdio: 'inherit',
    env: { ...process.env, NODE_ENV: process.env.NODE_ENV || 'production' },
  });

  // Verify build succeeded
  if (!fs.existsSync(bridgeNodeFile)) {
    throw new Error(`Bridge build failed: ${bridgeNodeFile} not found`);
  }

  console.log('✅ N-API Bridge built successfully');
} catch (error) {
  console.error('❌ Failed to build N-API bridge:', error.message);
  console.error('');
  console.error('To build manually:');
  console.error(`  cd ${bridgePath}`);
  console.error('  pnpm install');
  console.error('  pnpm build');
  process.exit(1);
}

