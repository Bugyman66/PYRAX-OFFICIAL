#!/bin/sh

echo "Running database migrations..."
prisma migrate deploy --schema=./prisma/schema.prisma 2>&1 || {
  echo "Migration failed, trying db push..."
  prisma db push --schema=./prisma/schema.prisma --accept-data-loss 2>&1 || {
    echo "WARNING: Database setup failed. App may not work correctly."
    echo "Check DATABASE_URL environment variable."
  }
}

echo "Starting application..."
exec "$@"
