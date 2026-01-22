# PYRAX Devnet Logging & Monitoring Guide

## Server Access
```bash
ssh root@209.38.137.105
```

---

## 🔗 BLOCKCHAIN NODE (TriStream) - All Streams

### View Real-Time Node Logs (All Streams A, B, C)
```bash
# Follow live logs
journalctl -u pyrax-node -f

# Last 100 lines
journalctl -u pyrax-node -n 100

# Logs since last hour
journalctl -u pyrax-node --since "1 hour ago"

# Logs since boot
journalctl -u pyrax-node -b

# Filter by priority (error, warning, info)
journalctl -u pyrax-node -p err
journalctl -u pyrax-node -p warning
```

### Node Service Status
```bash
# Check if node is running
systemctl status pyrax-node

# Restart node
systemctl restart pyrax-node

# Stop node
systemctl stop pyrax-node
```

### Stream-Specific Verification
```bash
# Stream A - RPC (BLAKE3 PoW) - Port 28545
curl -X POST http://localhost:28545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"pyrax_getChainInfo","params":[],"id":1}'

# Stream B - Stratum (KAWPOW GPU Mining) - Port 3333
netstat -tlnp | grep 3333

# Stream C - Staking RPC (ZK Proofs) - Port 28547
curl -X POST http://localhost:28547 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"pyrax_getStakingStats","params":[],"id":1}'

# P2P Network - Port 30303
netstat -tlnp | grep 30303
```

### Network & Peer Information
```bash
# Get connected peers
curl -X POST http://localhost:28545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"pyrax_getNetworkInfo","params":[],"id":1}'

# Get chain info
curl -X POST http://localhost:28545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"pyrax_getChainInfo","params":[],"id":1}'
```

---

## 🐳 DOCKER WEB SERVICES

### View All Container Status
```bash
cd /opt/pyrax
docker compose -f docker-compose.devnet.yml ps
```

### Follow ALL Docker Logs (Combined)
```bash
cd /opt/pyrax
docker compose -f docker-compose.devnet.yml logs -f
```

### Individual Service Logs

```bash
# Website
docker logs pyrax-website -f --tail 100

# Block Explorer
docker logs pyrax-explorer -f --tail 100

# Faucet
docker logs pyrax-faucet -f --tail 100

# Documentation
docker logs pyrax-docs -f --tail 100

# Marketing Hub
docker logs pyrax-marketing -f --tail 100

# Nginx Reverse Proxy
docker logs pyrax-nginx -f --tail 100

# PostgreSQL Database
docker logs pyrax-postgres -f --tail 100

# Redis Cache
docker logs pyrax-redis -f --tail 100
```

### Restart Individual Services
```bash
cd /opt/pyrax
docker compose -f docker-compose.devnet.yml restart website
docker compose -f docker-compose.devnet.yml restart explorer
docker compose -f docker-compose.devnet.yml restart faucet
docker compose -f docker-compose.devnet.yml restart marketing
docker compose -f docker-compose.devnet.yml restart nginx
```

### Restart All Web Services
```bash
cd /opt/pyrax
docker compose -f docker-compose.devnet.yml down
docker compose -f docker-compose.devnet.yml up -d
```

---

## 📊 SYSTEM MONITORING

### Check All Listening Ports
```bash
ss -tlnp | grep -E '(80|443|3000|3001|3002|3003|28545|3333|28547|30303)'
```

### Disk Usage
```bash
df -h
du -sh /var/lib/pyrax/devnet  # Blockchain data
du -sh /opt/pyrax             # Application code
```

### Memory & CPU
```bash
htop
# or
top
free -h
```

### Docker Resource Usage
```bash
docker stats
```

---

## 🔥 QUICK TROUBLESHOOTING

### Full System Status Check
```bash
echo "=== PYRAX NODE ===" && systemctl status pyrax-node --no-pager
echo ""
echo "=== DOCKER SERVICES ===" && cd /opt/pyrax && docker compose -f docker-compose.devnet.yml ps
echo ""
echo "=== LISTENING PORTS ===" && ss -tlnp | grep -E '(80|443|28545|3333|28547|30303)'
```

### Check for Errors in Last Hour
```bash
# Node errors
journalctl -u pyrax-node --since "1 hour ago" -p err

# Docker container errors
docker logs pyrax-nginx --since 1h 2>&1 | grep -i error
docker logs pyrax-explorer --since 1h 2>&1 | grep -i error
```

### View Firewall Status
```bash
ufw status verbose
```

---

## 🌐 SERVICE ENDPOINTS

| Service | Internal Port | External URL |
|---------|--------------|--------------|
| Stream A RPC | 28545 | http://209.38.137.105:28545 |
| Stream B Stratum | 3333 | 209.38.137.105:3333 |
| Stream C Staking | 28547 | http://209.38.137.105:28547 |
| P2P | 30303 | /ip4/209.38.137.105/tcp/30303 |
| Website | 3000 | https://pyrax-devnet.org |
| Explorer | 3003 | https://explorer.pyrax-devnet.org |
| Faucet | 3001 | https://faucet.pyrax-devnet.org |
| Docs | 3002 | https://docs.pyrax-devnet.org |
| Marketing | 3000 | https://marketing.pyrax-devnet.org |

---

## 🔄 DEPLOYMENT COMMANDS

### Trigger GitHub Actions Deployment
Push to `devnet` branch or go to:
https://github.com/PYRAX-Chain/PYRAX-OFFICIAL/actions → "Deploy to Devnet" → "Run workflow"

### Manual Node Rebuild (on server)
```bash
cd /opt/pyrax
source ~/.cargo/env
cargo build --release -p pyrax-node
systemctl stop pyrax-node
cp target/release/pyrax-node /usr/local/bin/
systemctl start pyrax-node
```

### Manual Docker Rebuild (on server)
```bash
cd /opt/pyrax
docker compose -f docker-compose.devnet.yml build --no-cache
docker compose -f docker-compose.devnet.yml up -d
```
