import Redis from 'ioredis';

const REDIS_URL = process.env.REDIS_URL || 'redis://localhost:6379';

const globalForRedis = globalThis as unknown as {
  redis: Redis | undefined;
  publisher: Redis | undefined;
  subscriber: Redis | undefined;
};

function createRedisClient(): Redis {
  const client = new Redis(REDIS_URL, {
    maxRetriesPerRequest: 3,
    retryDelayOnFailover: 100,
    lazyConnect: true,
  });

  client.on('error', (err: Error) => {
    console.error('Redis Client Error:', err);
  });

  client.on('connect', () => {
    console.log('Redis connected');
  });

  return client;
}

export const redis = globalForRedis.redis ?? createRedisClient();
export const publisher = globalForRedis.publisher ?? createRedisClient();
export const subscriber = globalForRedis.subscriber ?? createRedisClient();

if (process.env.NODE_ENV !== 'production') {
  globalForRedis.redis = redis;
  globalForRedis.publisher = publisher;
  globalForRedis.subscriber = subscriber;
}

// Cache utilities
const DEFAULT_TTL = 3600; // 1 hour

export async function getCache<T>(key: string): Promise<T | null> {
  try {
    const data = await redis.get(key);
    return data ? JSON.parse(data) : null;
  } catch (error) {
    console.error('Redis get error:', error);
    return null;
  }
}

export async function setCache<T>(
  key: string,
  value: T,
  ttl: number = DEFAULT_TTL
): Promise<void> {
  try {
    await redis.setex(key, ttl, JSON.stringify(value));
  } catch (error) {
    console.error('Redis set error:', error);
  }
}

export async function deleteCache(key: string): Promise<void> {
  try {
    await redis.del(key);
  } catch (error) {
    console.error('Redis delete error:', error);
  }
}

export async function deleteCachePattern(pattern: string): Promise<void> {
  try {
    const keys = await redis.keys(pattern);
    if (keys.length > 0) {
      await redis.del(...keys);
    }
  } catch (error) {
    console.error('Redis delete pattern error:', error);
  }
}

// Session utilities
const SESSION_PREFIX = 'session:';
const SESSION_TTL = 60 * 60 * 24 * 30; // 30 days

export async function getSession(token: string): Promise<string | null> {
  return redis.get(`${SESSION_PREFIX}${token}`);
}

export async function setSession(
  token: string,
  userId: string,
  ttl: number = SESSION_TTL
): Promise<void> {
  await redis.setex(`${SESSION_PREFIX}${token}`, ttl, userId);
}

export async function deleteSession(token: string): Promise<void> {
  await redis.del(`${SESSION_PREFIX}${token}`);
}

// Pub/Sub utilities
export type NotificationPayload = {
  type: 'proof_updated' | 'comment_added' | 'decision_made' | 'workflow_changed' | 'mention';
  entityId: string;
  entityType: string;
  userId?: string;
  data: Record<string, unknown>;
};

export async function publishNotification(
  channel: string,
  payload: NotificationPayload
): Promise<void> {
  try {
    await publisher.publish(channel, JSON.stringify(payload));
  } catch (error) {
    console.error('Redis publish error:', error);
  }
}

export function subscribeToChannel(
  channel: string,
  callback: (payload: NotificationPayload) => void
): void {
  subscriber.subscribe(channel, (err: Error | null) => {
    if (err) {
      console.error('Redis subscribe error:', err);
    }
  });

  subscriber.on('message', (ch: string, message: string) => {
    if (ch === channel) {
      try {
        const payload = JSON.parse(message) as NotificationPayload;
        callback(payload);
      } catch (error) {
        console.error('Redis message parse error:', error);
      }
    }
  });
}

export default redis;
