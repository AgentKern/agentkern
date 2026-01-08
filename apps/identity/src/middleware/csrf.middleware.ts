/**
 * CSRF Protection Middleware
 * 
 * Implements double-submit cookie pattern for CSRF protection.
 * Required for session-based flows where cookies are used.
 * 
 * Uses cryptographically random tokens stored in cookies and
 * validated against request headers.
 * 
 * @see https://cheatsheetseries.owasp.org/cheatsheets/Cross-Site_Request_Forgery_Prevention_Cheat_Sheet.html
 */
import { Injectable, NestMiddleware, HttpException, HttpStatus, Logger } from '@nestjs/common';
import { Request, Response, NextFunction } from 'express';
import { randomBytes } from 'crypto';

/** CSRF cookie name */
const CSRF_COOKIE_NAME = 'XSRF-TOKEN';

/** CSRF header name (Angular convention) */
const CSRF_HEADER_NAME = 'X-XSRF-TOKEN';

/** Token length in bytes */
const TOKEN_LENGTH = 32;

/** Methods that require CSRF validation */
const PROTECTED_METHODS = ['POST', 'PUT', 'DELETE', 'PATCH'];

/** Paths exempt from CSRF (webhooks, public APIs) */
const EXEMPT_PATHS = [
  '/api/v1/proof', // Public proof verification
  '/api/v1/proof/verify',
  '/api/v1/health',
  '/api/v1/security/csp-report', // CSP reports
  '/docs', // Swagger
];

@Injectable()
export class CsrfMiddleware implements NestMiddleware {
  private readonly logger = new Logger(CsrfMiddleware.name);

  use(req: Request, res: Response, next: NextFunction): void {
    // Check if path is exempt
    if (this.isExemptPath(req.path)) {
      return next();
    }

    // For safe methods, set CSRF token cookie if not present
    if (!PROTECTED_METHODS.includes(req.method)) {
      this.setTokenCookie(req, res);
      return next();
    }

    // For state-changing methods, validate CSRF token
    const cookieToken = req.cookies?.[CSRF_COOKIE_NAME];
    const headerToken = req.headers[CSRF_HEADER_NAME.toLowerCase()] as string;

    if (!cookieToken || !headerToken) {
      this.logger.warn('CSRF validation failed: missing tokens', {
        path: req.path,
        method: req.method,
        hasCookie: !!cookieToken,
        hasHeader: !!headerToken,
      });
      throw new HttpException('CSRF token missing', HttpStatus.FORBIDDEN);
    }

    // Constant-time comparison to prevent timing attacks
    if (!this.safeCompare(cookieToken, headerToken)) {
      this.logger.warn('CSRF validation failed: token mismatch', {
        path: req.path,
        method: req.method,
      });
      throw new HttpException('CSRF token invalid', HttpStatus.FORBIDDEN);
    }

    // Rotate token after successful validation
    this.setTokenCookie(req, res, true);
    
    next();
  }

  /**
   * Set CSRF token cookie if not present or forced.
   */
  private setTokenCookie(req: Request, res: Response, force = false): void {
    if (force || !req.cookies?.[CSRF_COOKIE_NAME]) {
      const token = randomBytes(TOKEN_LENGTH).toString('hex');
      res.cookie(CSRF_COOKIE_NAME, token, {
        httpOnly: false, // Must be readable by JavaScript
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'strict',
        path: '/',
        maxAge: 24 * 60 * 60 * 1000, // 24 hours
      });
    }
  }

  /**
   * Check if path is exempt from CSRF protection.
   */
  private isExemptPath(path: string): boolean {
    return EXEMPT_PATHS.some((exempt) => path.startsWith(exempt));
  }

  /**
   * Constant-time string comparison to prevent timing attacks.
   */
  private safeCompare(a: string, b: string): boolean {
    if (a.length !== b.length) {
      return false;
    }

    let result = 0;
    for (let i = 0; i < a.length; i++) {
      result |= a.charCodeAt(i) ^ b.charCodeAt(i);
    }

    return result === 0;
  }
}

/**
 * Decorator to mark endpoint as CSRF-exempt.
 * Use sparingly - only for webhook endpoints and public APIs.
 */
export const CsrfExempt = () => {
  return (target: any, propertyKey?: string, descriptor?: PropertyDescriptor) => {
    if (descriptor) {
      Reflect.defineMetadata('csrf:exempt', true, descriptor.value);
    }
  };
};
