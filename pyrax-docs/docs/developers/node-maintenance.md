# Node Maintenance

Guide for maintaining and operating PYRAX nodes.

## Regular Tasks

### Daily
- Check node is synced
- Monitor disk space
- Review logs for errors

### Weekly
- Check for software updates
- Review peer connections
- Backup configuration

### Monthly
- Update node software
- Clean old logs
- Review performance metrics

## Monitoring

### Sync Status

```bash
# Check if synced
curl -s localhost:8545 -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_syncing","params":[],"id":1}'

# Result: false = synced
# Result: {...} = still syncing
```

### Block Number

```bash
curl -s localhost:8545 -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
```

### Peer Count

```bash
curl -s localhost:8545 -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"net_peerCount","params":[],"id":1}'
```

### Health Check Script

```bash
#!/bin/bash
# health-check.sh

RPC="http://localhost:8545"

# Check if responding
if ! curl -s "$RPC" -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' > /dev/null; then
  echo "ERROR: Node not responding"
  exit 1
fi

# Check sync status
SYNCING=$(curl -s "$RPC" -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_syncing","params":[],"id":1}' | jq -r '.result')

if [ "$SYNCING" != "false" ]; then
  echo "WARNING: Node is syncing"
  exit 1
fi

echo "OK: Node is healthy"
exit 0
```

## Disk Management

### Check Usage

```bash
du -sh /data/pyrax/
df -h /data/pyrax/
```

### Pruning

For non-archive nodes, pruning happens automatically. For archive nodes, no pruning occurs.

### Cleanup

```bash
# Remove old logs
find /data/pyrax/logs -name "*.log" -mtime +30 -delete

# Remove old backups
find /backups -name "*.tar.gz" -mtime +90 -delete
```

## Updating

### Check Version

```bash
pyrax-node --version
```

### Update Process

```bash
# 1. Stop the node
sudo systemctl stop pyrax-node

# 2. Backup current binary
sudo cp /usr/local/bin/pyrax-node /usr/local/bin/pyrax-node.backup

# 3. Download new version
wget https://github.com/PYRAX-Chain/pyrax-core/releases/latest/download/pyrax-node-linux-amd64.tar.gz
tar -xzf pyrax-node-linux-amd64.tar.gz
sudo mv pyrax-node /usr/local/bin/

# 4. Verify
pyrax-node --version

# 5. Start
sudo systemctl start pyrax-node

# 6. Check logs
journalctl -u pyrax-node -f
```

## Backup & Recovery

### What to Backup

| Item | Location | Frequency |
|------|----------|-----------|
| Keystore | `datadir/keystore/` | On change |
| Config | `/etc/pyrax/` | On change |
| Node key | `datadir/geth/nodekey` | Once |

### Backup Script

```bash
#!/bin/bash
DATADIR=/data/pyrax
BACKUP_DIR=/backups
DATE=$(date +%Y%m%d)

# Backup keystore
tar -czf "$BACKUP_DIR/keystore-$DATE.tar.gz" "$DATADIR/keystore/"

# Backup config
cp /etc/pyrax/config.toml "$BACKUP_DIR/config-$DATE.toml"

# Backup node key
cp "$DATADIR/geth/nodekey" "$BACKUP_DIR/nodekey-$DATE"
```

### Recovery

```bash
# Stop node
sudo systemctl stop pyrax-node

# Restore keystore
tar -xzf /backups/keystore-YYYYMMDD.tar.gz -C /data/pyrax/

# Restore config
cp /backups/config-YYYYMMDD.toml /etc/pyrax/config.toml

# Restart
sudo systemctl start pyrax-node
```

## Troubleshooting

### Node Not Syncing

1. Check internet connectivity
2. Verify peers are connecting
3. Check disk space
4. Review logs for errors
5. Try adding manual peers

```bash
# Add peer manually
pyrax-node attach --exec "admin.addPeer('enode://...')"
```

### High Resource Usage

**High CPU:**
- Check if syncing (normal during sync)
- Reduce max peers
- Check for attack (many requests)

**High Memory:**
- Reduce cache size
- Check for memory leaks (update software)

**High Disk I/O:**
- Use SSD
- Check for corruption
- Consider faster storage

### RPC Not Responding

1. Check node is running
2. Verify port is open
3. Check bind address
4. Review firewall rules

### Corrupted Database

```bash
# Stop node
sudo systemctl stop pyrax-node

# Remove corrupt data
rm -rf /data/pyrax/geth/chaindata

# Restart (will resync)
sudo systemctl start pyrax-node
```

## Alerts Setup

### Prometheus + Grafana

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'pyrax-node'
    static_configs:
      - targets: ['localhost:6060']
```

### Simple Alert Script

```bash
#!/bin/bash
# Check every minute via cron

if ! /path/to/health-check.sh; then
  # Send alert
  curl -X POST "https://hooks.slack.com/..." \
    -d '{"text": "PYRAX Node Alert: Node unhealthy"}'
fi
```

---

:::tip Maintenance Tips
1. Automate monitoring and alerts
2. Keep detailed runbooks
3. Test recovery procedures
4. Document all changes
5. Join operator community for updates
:::
