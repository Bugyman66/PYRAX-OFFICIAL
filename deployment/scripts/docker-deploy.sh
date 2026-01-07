#!/bin/bash
# PYRAX Docker Deployment Script
# Usage: ./docker-deploy.sh [service]
# Example: ./docker-deploy.sh all
# Example: ./docker-deploy.sh website

set -e

SERVICE=${1:-all}
VALID_SERVICES="all website faucet docs explorer nginx"

if [[ ! " $VALID_SERVICES " =~ " $SERVICE " ]]; then
    echo "Error: Invalid service '$SERVICE'"
    echo "Usage: ./docker-deploy.sh <all|website|faucet|docs|explorer|nginx>"
    exit 1
fi

cd /opt/pyrax

echo "============================================"
echo "PYRAX Docker Deploy - $SERVICE"
echo "============================================"

# Check if .env exists
if [ ! -f .env ]; then
    echo "Error: .env file not found!"
    echo "Copy .env.example to .env and configure it first."
    exit 1
fi

# Check if SSL certificates exist
if [ ! -f deployment/ssl/pyrax.org.pem ] || [ ! -f deployment/ssl/pyrax.org.key ]; then
    echo "Warning: SSL certificates not found in deployment/ssl/"
    echo "Please add your Cloudflare Origin Certificates:"
    echo "  deployment/ssl/pyrax.org.pem"
    echo "  deployment/ssl/pyrax.org.key"
fi

# Pull latest changes if git repo
if [ -d .git ]; then
    echo "Pulling latest changes..."
    git pull origin main || true
fi

if [ "$SERVICE" = "all" ]; then
    echo "Building all services..."
    docker compose build
    
    echo "Starting all services..."
    docker compose up -d
else
    echo "Building $SERVICE..."
    docker compose build $SERVICE
    
    echo "Restarting $SERVICE..."
    docker compose up -d --no-deps $SERVICE
fi

# Cleanup old images
echo "Cleaning up old images..."
docker image prune -f

# Show status
echo ""
echo "============================================"
echo "Deployment Complete!"
echo "============================================"
docker compose ps

echo ""
echo "View logs with: docker compose logs -f $SERVICE"
