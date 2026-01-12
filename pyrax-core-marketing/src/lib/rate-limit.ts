const rateLimitMap = new Map<string, { count: number; resetTime: number }>();

interface RateLimitConfig {
  windowMs: number;
  maxRequests: number;
}

const DEFAULT_CONFIG: RateLimitConfig = {
  windowMs: 60 * 1000, // 1 minute
  maxRequests: 60,
};

const AUTH_CONFIG: RateLimitConfig = {
  windowMs: 15 * 60 * 1000, // 15 minutes
  maxRequests: 10,
};

const UPLOAD_CONFIG: RateLimitConfig = {
  windowMs: 60 * 1000, // 1 minute
  maxRequests: 10,
};

export function getRateLimitConfig(path: string): RateLimitConfig {
  if (path.includes('/api/auth/')) {
    return AUTH_CONFIG;
  }
  if (path.includes('/api/files') || path.includes('/api/versions')) {
    return UPLOAD_CONFIG;
  }
  return DEFAULT_CONFIG;
}

export function checkRateLimit(
  identifier: string,
  config: RateLimitConfig = DEFAULT_CONFIG
): { allowed: boolean; remaining: number; resetIn: number } {
  const now = Date.now();
  const key = identifier;
  
  const existing = rateLimitMap.get(key);
  
  if (!existing || now > existing.resetTime) {
    rateLimitMap.set(key, {
      count: 1,
      resetTime: now + config.windowMs,
    });
    return {
      allowed: true,
      remaining: config.maxRequests - 1,
      resetIn: config.windowMs,
    };
  }
  
  if (existing.count >= config.maxRequests) {
    return {
      allowed: false,
      remaining: 0,
      resetIn: existing.resetTime - now,
    };
  }
  
  existing.count++;
  return {
    allowed: true,
    remaining: config.maxRequests - existing.count,
    resetIn: existing.resetTime - now,
  };
}

export function getClientIdentifier(request: Request): string {
  const forwarded = request.headers.get('x-forwarded-for');
  const realIp = request.headers.get('x-real-ip');
  
  if (forwarded) {
    return forwarded.split(',')[0].trim();
  }
  
  if (realIp) {
    return realIp;
  }
  
  return 'unknown';
}

// Cleanup old entries periodically
if (typeof setInterval !== 'undefined') {
  setInterval(() => {
    const now = Date.now();
    const keys = Array.from(rateLimitMap.keys());
    for (const key of keys) {
      const value = rateLimitMap.get(key);
      if (value && now > value.resetTime) {
        rateLimitMap.delete(key);
      }
    }
  }, 60 * 1000);
}
