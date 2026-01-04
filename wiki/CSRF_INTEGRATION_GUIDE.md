# CSRF Protection Integration Guide

This guide explains how to integrate CSRF protection in the AgentKern Identity service.

## Overview

The Identity service uses a **double-submit cookie** pattern for CSRF protection:
1. A random token is set in a cookie (`XSRF-TOKEN`)
2. The client must include this token in the `X-XSRF-TOKEN` header
3. The server validates that both match

## Configuration

### Enable CSRF Middleware

Add `CsrfMiddleware` to your NestJS app:

```typescript
// main.ts
import { CsrfMiddleware } from './middleware/csrf.middleware';

async function bootstrap() {
  const app = await NestFactory.create(AppModule);
  
  // Apply CSRF middleware globally
  app.use(new CsrfMiddleware().use);
  
  await app.listen(3000);
}
```

### Exempt Paths

The following paths are automatically exempt:
- `/api/v1/proof` - Public proof verification
- `/api/v1/health` - Health checks
- `/api/v1/security/csp-report` - CSP reports
- `/docs` - Swagger documentation

### Custom Exemptions

Use the `@CsrfExempt()` decorator for webhook endpoints:

```typescript
import { CsrfExempt } from '../middleware/csrf.middleware';

@Post('webhook')
@CsrfExempt()
handleWebhook() {
  // CSRF not required for webhooks
}
```

## Client Integration

### Angular (Automatic)

Angular's HttpClient automatically reads `XSRF-TOKEN` cookie and sends `X-XSRF-TOKEN` header.

### React/Fetch

```typescript
async function makeRequest(url: string, method: string, body?: object) {
  // Get CSRF token from cookie
  const xsrfToken = document.cookie
    .split('; ')
    .find(row => row.startsWith('XSRF-TOKEN='))
    ?.split('=')[1];

  return fetch(url, {
    method,
    credentials: 'include',
    headers: {
      'Content-Type': 'application/json',
      'X-XSRF-TOKEN': xsrfToken || '',
    },
    body: body ? JSON.stringify(body) : undefined,
  });
}
```

### Python

```python
import requests

session = requests.Session()

# First GET to receive CSRF cookie
session.get('https://identity.agentkern.io/api/v1/health')

# POST with CSRF token
csrf_token = session.cookies.get('XSRF-TOKEN')
response = session.post(
    'https://identity.agentkern.io/api/v1/agents',
    json={'name': 'my-agent'},
    headers={'X-XSRF-TOKEN': csrf_token}
)
```

## Security Notes

1. **Cookie settings**: `SameSite=Strict`, `Secure` in production
2. **Token rotation**: Token rotates after each successful validation
3. **Constant-time comparison**: Prevents timing attacks
4. **HTTPS required**: Always use HTTPS in production

## Error Responses

| Code | Message | Cause |
|------|---------|-------|
| 403 | CSRF token missing | Missing cookie or header |
| 403 | CSRF token invalid | Token mismatch |
