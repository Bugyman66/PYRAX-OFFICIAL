import { Queue, Worker, Job } from 'bullmq';
import Redis from 'ioredis';

const REDIS_URL = process.env.REDIS_URL || 'redis://localhost:6379';

const connection = new Redis(REDIS_URL, {
  maxRetriesPerRequest: null,
});

// Queue names
export const QUEUE_NAMES = {
  EMAIL: 'email-queue',
  NOTIFICATION: 'notification-queue',
  FILE_PROCESSING: 'file-processing-queue',
  CLEANUP: 'cleanup-queue',
} as const;

// Email queue for sending magic links, notifications, etc.
export const emailQueue = new Queue(QUEUE_NAMES.EMAIL, { connection });

// Notification queue for async notification processing
export const notificationQueue = new Queue(QUEUE_NAMES.NOTIFICATION, { connection });

// File processing queue for thumbnails, conversions, etc.
export const fileProcessingQueue = new Queue(QUEUE_NAMES.FILE_PROCESSING, { connection });

// Cleanup queue for expired sessions, old files, etc.
export const cleanupQueue = new Queue(QUEUE_NAMES.CLEANUP, { connection });

// Job types
export type EmailJobData = {
  type: 'magic_link' | 'notification' | 'workflow_update' | 'mention';
  to: string;
  subject: string;
  htmlContent: string;
  textContent?: string;
};

export type NotificationJobData = {
  userId: string;
  type: string;
  title: string;
  message: string;
  entityType?: string;
  entityId?: string;
  metadata?: Record<string, unknown>;
};

export type FileProcessingJobData = {
  fileId: string;
  operation: 'thumbnail' | 'preview' | 'convert';
  source: string;
  options?: Record<string, unknown>;
};

export type CleanupJobData = {
  type: 'expired_sessions' | 'old_magic_links' | 'temp_files';
  olderThan?: Date;
};

// Add jobs to queues
export async function addEmailJob(data: EmailJobData): Promise<Job<EmailJobData>> {
  return emailQueue.add('send-email', data, {
    attempts: 3,
    backoff: {
      type: 'exponential',
      delay: 1000,
    },
    removeOnComplete: 100,
    removeOnFail: 1000,
  });
}

export async function addNotificationJob(data: NotificationJobData): Promise<Job<NotificationJobData>> {
  return notificationQueue.add('send-notification', data, {
    attempts: 3,
    backoff: {
      type: 'exponential',
      delay: 500,
    },
    removeOnComplete: 100,
    removeOnFail: 1000,
  });
}

export async function addFileProcessingJob(data: FileProcessingJobData): Promise<Job<FileProcessingJobData>> {
  return fileProcessingQueue.add('process-file', data, {
    attempts: 2,
    backoff: {
      type: 'fixed',
      delay: 5000,
    },
    removeOnComplete: 50,
    removeOnFail: 500,
  });
}

export async function addCleanupJob(data: CleanupJobData): Promise<Job<CleanupJobData>> {
  return cleanupQueue.add('cleanup', data, {
    attempts: 1,
    removeOnComplete: true,
    removeOnFail: 100,
  });
}

// Schedule recurring cleanup jobs
export async function scheduleCleanupJobs(): Promise<void> {
  // Clean expired sessions every hour
  await cleanupQueue.add(
    'cleanup-sessions',
    { type: 'expired_sessions' },
    {
      repeat: {
        pattern: '0 * * * *', // Every hour
      },
      removeOnComplete: true,
    }
  );

  // Clean old magic links every 6 hours
  await cleanupQueue.add(
    'cleanup-magic-links',
    { type: 'old_magic_links' },
    {
      repeat: {
        pattern: '0 */6 * * *', // Every 6 hours
      },
      removeOnComplete: true,
    }
  );
}

// Get queue stats
export async function getQueueStats(): Promise<Record<string, unknown>> {
  const [emailStats, notificationStats, fileStats, cleanupStats] = await Promise.all([
    emailQueue.getJobCounts(),
    notificationQueue.getJobCounts(),
    fileProcessingQueue.getJobCounts(),
    cleanupQueue.getJobCounts(),
  ]);

  return {
    email: emailStats,
    notification: notificationStats,
    fileProcessing: fileStats,
    cleanup: cleanupStats,
  };
}

export { connection as queueConnection };
