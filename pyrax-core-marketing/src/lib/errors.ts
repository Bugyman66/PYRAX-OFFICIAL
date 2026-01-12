export class AppError extends Error {
  constructor(
    message: string,
    public code: string,
    public statusCode: number = 400,
    public details?: Record<string, unknown>
  ) {
    super(message);
    this.name = 'AppError';
  }
}

export class AuthenticationError extends AppError {
  constructor(message = 'Authentication required') {
    super(message, 'UNAUTHENTICATED', 401);
    this.name = 'AuthenticationError';
  }
}

export class AuthorizationError extends AppError {
  constructor(message = 'You do not have permission to perform this action') {
    super(message, 'FORBIDDEN', 403);
    this.name = 'AuthorizationError';
  }
}

export class NotFoundError extends AppError {
  constructor(resource = 'Resource', message?: string) {
    super(message || `${resource} not found`, 'NOT_FOUND', 404);
    this.name = 'NotFoundError';
  }
}

export class ValidationError extends AppError {
  constructor(message: string, details?: Record<string, string[]>) {
    super(message, 'VALIDATION_ERROR', 400, details);
    this.name = 'ValidationError';
  }
}

export class RateLimitError extends AppError {
  constructor(retryAfter: number) {
    super('Too many requests', 'RATE_LIMITED', 429, { retryAfter });
    this.name = 'RateLimitError';
  }
}

export class ConflictError extends AppError {
  constructor(message: string) {
    super(message, 'CONFLICT', 409);
    this.name = 'ConflictError';
  }
}

export function isAppError(error: unknown): error is AppError {
  return error instanceof AppError;
}

export function handleApiError(error: unknown): { message: string; code: string; statusCode: number } {
  console.error('API Error:', error);

  if (isAppError(error)) {
    return {
      message: error.message,
      code: error.code,
      statusCode: error.statusCode,
    };
  }

  if (error instanceof Error) {
    return {
      message: process.env.NODE_ENV === 'development' ? error.message : 'An unexpected error occurred',
      code: 'INTERNAL_ERROR',
      statusCode: 500,
    };
  }

  return {
    message: 'An unexpected error occurred',
    code: 'INTERNAL_ERROR',
    statusCode: 500,
  };
}

export function validateRequired(
  data: Record<string, unknown>,
  fields: string[]
): void {
  const missing: string[] = [];
  
  for (const field of fields) {
    const value = data[field];
    if (value === undefined || value === null || value === '') {
      missing.push(field);
    }
  }
  
  if (missing.length > 0) {
    throw new ValidationError(
      `Missing required fields: ${missing.join(', ')}`,
      { missing }
    );
  }
}

export function validateEmail(email: string): boolean {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return emailRegex.test(email);
}

export function validateSlug(slug: string): boolean {
  const slugRegex = /^[a-z0-9]+(?:-[a-z0-9]+)*$/;
  return slugRegex.test(slug) && slug.length >= 3 && slug.length <= 100;
}

export function sanitizeHtml(input: string): string {
  return input
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
}

export function sanitizeInput(input: string, maxLength = 1000): string {
  if (typeof input !== 'string') return '';
  return input.trim().substring(0, maxLength);
}
