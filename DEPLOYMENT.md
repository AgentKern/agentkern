# Deployment Guide

This guide covers deploying AgentKern to production.

---

## Prerequisites

You'll need accounts with:
- [Vercel](https://vercel.com) — For the Playground
- [npm](https://www.npmjs.com) — For the SDK
- [Fly.io](https://fly.io) — For Rust services (or Docker-compatible host)

---

## 1. GitHub Secrets Setup

Add these secrets to your GitHub repository:

### For Vercel (Playground)

1. Go to [Vercel Dashboard](https://vercel.com/dashboard) → Settings → Tokens
2. Create a new token
3. In GitHub: Settings → Secrets → Actions, add:
   - `VERCEL_TOKEN` — Your Vercel token
   - `VERCEL_ORG_ID` — From `.vercel/project.json` after `vercel link`
   - `VERCEL_PROJECT_ID` — From `.vercel/project.json` after `vercel link`

### For npm (SDK)

1. Go to [npmjs.com](https://www.npmjs.com) → Access Tokens → Generate New Token
2. Choose "Automation" type for CI/CD
3. In GitHub: Settings → Secrets → Actions, add:
   - `NPM_TOKEN` — Your npm automation token

### For Fly.io (Rust Services)

1. Install Fly CLI: `curl -L https://fly.io/install.sh | sh`
2. Run: `fly auth token`
3. In GitHub: Settings → Secrets → Actions, add:
   - `FLY_API_TOKEN` — Your Fly token

---

## 2. Deploy the Playground

### Automatic (via GitHub Actions)

Push to `main` and the workflow deploys automatically.

### Manual (first time)

```bash
cd apps/playground
npm install
npm run build

# Link to Vercel
npx vercel link

# Deploy
npx vercel --prod
```

---

## 3. Publish SDK to npm

### Automatic (via GitHub Actions)

1. Create a GitHub Release
2. The `publish-sdk.yml` workflow triggers automatically

### Manual

```bash
cd sdks/typescript/agentkern
npm install
npm run build
npm test
npm publish --access public
```

---

## 4. Deploy Rust Services

### Using Fly.io

```bash
# Gate
cd packages/pillars/gate
fly launch --name agentkern-gate
fly deploy

# Synapse
cd packages/pillars/synapse
fly launch --name agentkern-synapse
fly deploy

# Arbiter
cd packages/pillars/arbiter
fly launch --name agentkern-arbiter
fly deploy
```

### Using Docker

```bash
# Build all images
docker build -t agentkern-gate packages/pillars/gate
docker build -t agentkern-synapse packages/pillars/synapse
docker build -t agentkern-arbiter packages/pillars/arbiter

# Run locally
docker run -p 3001:3001 agentkern-gate
docker run -p 3002:3002 agentkern-synapse
docker run -p 3003:3003 agentkern-arbiter
```

### Using Docker Compose

```yaml
version: '3.8'
services:
  gate:
    build: ./packages/pillars/gate
    ports:
      - "3001:3001"
    environment:
      - PORT=3001

  synapse:
    build: ./packages/pillars/synapse
    ports:
      - "3002:3002"
    environment:
      - PORT=3002

  arbiter:
    build: ./packages/pillars/arbiter
    ports:
      - "3003:3003"
    environment:
      - PORT=3003
```

---

## 5. Environment Variables

### Gate (port 3001)
```
PORT=3001
```

### Synapse (port 3002)
```
PORT=3002
```

### Arbiter (port 3003)
```
PORT=3003
```

---

## 6. Health Checks

Each service exposes a `/health` endpoint:

```bash
curl http://localhost:3001/health  # Gate
curl http://localhost:3002/health  # Synapse
curl http://localhost:3003/health  # Arbiter
```

---

## 7. Service URLs (After Deployment)

| Service | Local | Fly.io |
|---------|-------|--------|
| Gate | http://localhost:3001 | https://agentkern-gate.fly.dev |
| Synapse | http://localhost:3002 | https://agentkern-synapse.fly.dev |
| Arbiter | http://localhost:3003 | https://agentkern-arbiter.fly.dev |
| Playground | http://localhost:5173 | https://agentkern-playground.vercel.app |
