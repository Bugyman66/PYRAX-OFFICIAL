# PYRAX Deployment Guide

Complete guide for deploying PYRAX infrastructure to Digital Ocean.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Cloudflare (DNS + SSL)                    │
│    pyrax.org | docs.pyrax.org | explorer.pyrax.org | faucet │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                   Digital Ocean Droplet                      │
│  ┌───────────────────────────────────────────────────────┐  │
│  │                    Nginx (80/443)                      │  │
│  └───────────────────────────────────────────────────────┘  │
│         │              │              │              │       │
│         ▼              ▼              ▼              ▼       │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐    │
│  │ Website  │  │   Docs   │  │ Explorer │  │  Faucet  │    │
│  │  :3000   │  │  :3002   │  │  :3003   │  │  :3001   │    │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘    │
└─────────────────────────────────────────────────────────────┘
```

## Services

| Service | Domain | Internal Port | Description |
|---------|--------|---------------|-------------|
| Website | pyrax.org | 3000 | Main marketing website |
| Docs | docs.pyrax.org | 3002 | Docusaurus documentation |
| Explorer | explorer.pyrax.org | 3003 | Block explorer |
| Faucet | faucet.pyrax.org | 3001 | Testnet token faucet |

---

## Quick Start (5 minutes)

### 1. Create Digital Ocean Droplet

- **Image**: Ubuntu 22.04 LTS
- **Size**: Minimum 2GB RAM / 2 vCPU ($18/mo) or 4GB RAM for production
- **Region**: Choose closest to your users
- **Add SSH key**: Required for deployment

### 2. Run Initial Setup

SSH into your droplet and run:

```bash
curl -sSL https://raw.githubusercontent.com/PYRAX-Chain/PYRAX-OFFICIAL/main/deployment/scripts/initial-setup.sh | sudo bash
```

This installs Docker, clones the repo, and configures the firewall.

### 3. Setup Cloudflare SSL

1. Go to **Cloudflare Dashboard → SSL/TLS → Origin Server**
2. Click **Create Certificate**
3. Hostnames: `*.pyrax.org, pyrax.org`
4. Validity: 15 years
5. Save the certificate and key to the droplet:

```bash
nano /opt/pyrax/deployment/ssl/pyrax.org.pem  # Paste certificate
nano /opt/pyrax/deployment/ssl/pyrax.org.key  # Paste private key
chmod 600 /opt/pyrax/deployment/ssl/*
```

### 4. Configure Environment

```bash
cd /opt/pyrax
cp .env.example .env
nano .env
```

Fill in:
- `FAUCET_PRIVATE_KEY` - Private key for faucet wallet
- Other settings as needed

### 5. Deploy!

```bash
docker compose up -d
```

Check status:
```bash
docker compose ps
docker compose logs -f
```

---

## DNS Configuration

Add these records in Cloudflare (orange cloud = proxied):

| Type | Name | Content | Proxy |
|------|------|---------|-------|
| A | @ | YOUR_DROPLET_IP | ✅ Proxied |
| A | www | YOUR_DROPLET_IP | ✅ Proxied |
| A | docs | YOUR_DROPLET_IP | ✅ Proxied |
| A | explorer | YOUR_DROPLET_IP | ✅ Proxied |
| A | faucet | YOUR_DROPLET_IP | ✅ Proxied |

### Cloudflare SSL Settings

In **SSL/TLS** settings:
- **Mode**: Full (Strict)
- **Always Use HTTPS**: On
- **Automatic HTTPS Rewrites**: On
- **Minimum TLS Version**: 1.2

---

## GitHub Actions Auto-Deploy

### Required Secrets

Add these in GitHub → Settings → Secrets and variables → Actions:

| Secret | Description |
|--------|-------------|
| `DROPLET_IP` | Your droplet's IP address |
| `DROPLET_SSH_KEY` | Private SSH key for root access |
| `FAUCET_PRIVATE_KEY` | Faucet wallet private key |

### Required Variables (Optional)

| Variable | Default | Description |
|----------|---------|-------------|
| `NETWORK` | mainnet | Network environment |
| `NEXT_PUBLIC_RPC_URL` | https://rpc.pyrax.org | Public RPC URL |
| `NEXT_PUBLIC_API_URL` | https://api.pyrax.org | Public API URL |
| `FAUCET_AMOUNT` | 10 | Tokens per faucet request |
| `RATE_LIMIT_HOURS` | 24 | Faucet rate limit |

### Generate SSH Key

```bash
# On your local machine
ssh-keygen -t ed25519 -C "github-deploy" -f ~/.ssh/pyrax-deploy

# Copy public key to droplet
ssh-copy-id -i ~/.ssh/pyrax-deploy.pub root@YOUR_DROPLET_IP

# Add private key content to GitHub secret DROPLET_SSH_KEY
cat ~/.ssh/pyrax-deploy
```

### How Auto-Deploy Works

1. Push to `main` branch triggers deployment
2. GitHub Actions detects which services changed
3. Only changed services are rebuilt
4. Docker images are built and deployed
5. Health checks verify deployment

---

## Manual Deployment

### Deploy All Services

```bash
ssh root@YOUR_DROPLET_IP
cd /opt/pyrax
git pull origin main
docker compose build
docker compose up -d
```

### Deploy Single Service

```bash
docker compose build website
docker compose up -d --no-deps website
```

### View Logs

```bash
docker compose logs -f              # All services
docker compose logs -f website      # Single service
```

### Restart Services

```bash
docker compose restart              # All
docker compose restart website      # Single
```

---

## Environment Variables

### .env File

```bash
# Network
NETWORK=mainnet

# RPC Configuration
RPC_URL=http://localhost:8545
NEXT_PUBLIC_RPC_URL=https://rpc.pyrax.org
NEXT_PUBLIC_API_URL=https://api.pyrax.org

# Faucet
FAUCET_PRIVATE_KEY=0x...
FAUCET_AMOUNT=10
RATE_LIMIT_HOURS=24
```

### Per-Service Environment

Services can have their own `.env` files in their directories for development.

---

## SSL Certificates

### Cloudflare Origin Certificate (Recommended)

Free, auto-renewing, 15-year validity. Works only with Cloudflare proxy.

1. Create at Cloudflare → SSL/TLS → Origin Server
2. Save to `/opt/pyrax/deployment/ssl/`
3. Files needed:
   - `pyrax.org.pem` (certificate)
   - `pyrax.org.key` (private key)

### Let's Encrypt (Alternative)

If not using Cloudflare:

```bash
apt install certbot
certbot certonly --standalone -d pyrax.org -d www.pyrax.org -d docs.pyrax.org
```

---

## Troubleshooting

### Container Won't Start

```bash
docker compose logs website  # Check for errors
docker compose down
docker compose up -d
```

### Port Already in Use

```bash
lsof -i :3000  # Find process
kill -9 PID    # Kill it
```

### SSL Certificate Issues

```bash
# Check certificate is valid
openssl x509 -in /opt/pyrax/deployment/ssl/pyrax.org.pem -text -noout

# Check key matches certificate
openssl x509 -noout -modulus -in pyrax.org.pem | md5sum
openssl rsa -noout -modulus -in pyrax.org.key | md5sum
# Both should match
```

### Nginx Configuration Test

```bash
docker compose exec nginx nginx -t
```

### Check Docker Resources

```bash
docker system df           # Disk usage
docker system prune -a     # Clean up (careful!)
```

---

## Scaling & Performance

### Increase Resources

For high-traffic deployments:
- Upgrade droplet to 4GB+ RAM
- Add swap space: `fallocate -l 2G /swapfile`
- Consider load balancer for multiple droplets

### Enable Caching

Cloudflare handles edge caching automatically. For additional caching:
- Enable Cloudflare APO for WordPress-style caching
- Configure cache rules for static assets

---

## Security Checklist

- [ ] SSH key authentication only (disable password)
- [ ] UFW firewall enabled
- [ ] Fail2ban running
- [ ] Cloudflare proxy enabled (hides real IP)
- [ ] SSL/TLS mode set to Full (Strict)
- [ ] Environment variables secured
- [ ] Regular security updates: `apt update && apt upgrade`

---

## Monitoring

### Basic Monitoring

```bash
# Resource usage
htop

# Docker stats
docker stats

# Logs
docker compose logs -f --tail=100
```

### Setup Monitoring (Optional)

Consider adding:
- **Uptime Robot** - Free uptime monitoring
- **Grafana + Prometheus** - Advanced metrics
- **Sentry** - Error tracking

---

## Backup Strategy

### Database Backups (if applicable)

```bash
# Add to crontab
0 2 * * * docker exec postgres pg_dump -U postgres pyrax > /backups/pyrax-$(date +\%Y\%m\%d).sql
```

### Volume Backups

```bash
docker run --rm -v pyrax_data:/data -v /backups:/backup alpine tar cvf /backup/data.tar /data
```

---

## Support

- **Documentation**: https://docs.pyrax.org
- **GitHub Issues**: https://github.com/PYRAX-Chain/PYRAX-OFFICIAL/issues
- **Discord**: https://discord.gg/pyrax
- **Telegram**: https://t.me/pyrax
