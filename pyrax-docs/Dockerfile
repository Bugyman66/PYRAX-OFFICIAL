# PYRAX Docs Dockerfile
FROM node:20-alpine AS base

# Install dependencies only when needed
FROM base AS deps
RUN apk add --no-cache libc6-compat
WORKDIR /app

COPY package.json package-lock.json* ./
RUN npm ci

# Rebuild the source code only when needed
FROM base AS builder
WORKDIR /app
COPY --from=deps /app/node_modules ./node_modules
COPY . .

ENV NODE_ENV=production

RUN npm run build

# Production image using serve
FROM base AS runner
WORKDIR /app

RUN npm install -g serve

COPY --from=builder /app/build ./build

EXPOSE 3002

CMD ["serve", "-s", "build", "-l", "3002"]
