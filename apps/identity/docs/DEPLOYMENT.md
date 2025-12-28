# Deployment Guide

## Quick Start

### Local Development
```bash
npm install
npm run start:dev
```

### Staging Deployment (Docker)
```bash
# Start all services
docker compose -f docker-compose.staging.yml up -d --build

# Check status
docker compose -f docker-compose.staging.yml ps

# View logs
docker compose -f docker-compose.staging.yml logs -f api

# Stop services
docker compose -f docker-compose.staging.yml down
```

### Production Deployment
```bash
# Use production compose file
docker compose -f docker-compose.yml up -d --build
```

---

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `NODE_ENV` | Environment mode | `development` |
| `PORT` | API port | `3000` |
| `DATABASE_HOST` | PostgreSQL host | `localhost` |
| `DATABASE_PORT` | PostgreSQL port | `5432` |
| `DATABASE_USER` | Database user | `agentkern-identity` |
| `DATABASE_PASSWORD` | Database password | - |
| `DATABASE_NAME` | Database name | `agentkern-identity` |
| `DATABASE_SYNC` | Auto-sync schema | `false` |
| `DATABASE_SSL` | Enable SSL | `false` |
| `WEBAUTHN_RP_NAME` | Relying party name | `AgentKern Identity` |
| `WEBAUTHN_RP_ID` | Relying party ID | `localhost` |
| `WEBAUTHN_ORIGIN` | WebAuthn origin | `http://localhost:3000` |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Docker Network                           │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │   API       │  │  PostgreSQL │  │   Redis     │         │
│  │  :3000      │  │   :5432     │  │   :6379     │         │
│  │             │──│             │  │             │         │
│  │  NestJS     │  │   v16       │  │   v7        │         │
│  └─────────────┘  └─────────────┘  └─────────────┘         │
└─────────────────────────────────────────────────────────────┘
```

---

## Health Checks

### API Health
```bash
curl http://localhost:3000/health
# Returns: {"status":"healthy","timestamp":"..."}
```

### Database Health
```bash
docker exec agentkern-identity-postgres-1 pg_isready -U agentkern-identity
```

### Redis Health
```bash
docker exec agentkern-identity-redis-1 redis-cli ping
```

---

## Scaling

### Horizontal Scaling (Kubernetes)
See `k8s/` directory for Kubernetes manifests.

### Vertical Scaling (Docker)
```bash
# Increase API replicas
docker compose -f docker-compose.staging.yml up -d --scale api=3
```

---

## Troubleshooting

### API not starting
```bash
# Check logs
docker compose -f docker-compose.staging.yml logs api

# Common issues:
# - Database not ready: Wait for postgres health check
# - Port conflict: Change PORT in .env.staging
```

### Database connection failed
```bash
# Verify postgres is running
docker compose -f docker-compose.staging.yml ps postgres

# Check credentials in .env.staging
```

---

## Security Notes

1. **Never commit `.env` files** with real secrets
2. **Use strong passwords** in production
3. **Enable SSL** for database in production (`DATABASE_SSL=true`)
4. **Review rate limits** in `app.module.ts` for your use case
