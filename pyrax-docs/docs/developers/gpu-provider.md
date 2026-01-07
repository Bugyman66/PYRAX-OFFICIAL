# GPU Provider Guide

This guide explains how to become a Crucible GPU provider and earn PYRAX by processing AI jobs.

## Overview

GPU providers earn PYRAX by:
- Processing AI compute jobs
- Maintaining high uptime
- Delivering accurate results
- Building reputation

## Requirements

### Hardware

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| GPU | RTX 3060 (12GB) | RTX 4090 (24GB) |
| CPU | 8 cores | 16+ cores |
| RAM | 16 GB | 32+ GB |
| Storage | 100 GB SSD | 500 GB NVMe |
| Network | 100 Mbps | 1 Gbps |

### Supported GPUs

| GPU | VRAM | Tier | Est. Earnings |
|-----|------|------|---------------|
| RTX 3060 | 12 GB | Entry | 200-500 PYRAX/day |
| RTX 3080 | 10 GB | Standard | 400-800 PYRAX/day |
| RTX 4090 | 24 GB | Premium | 800-1500 PYRAX/day |
| A100 | 40/80 GB | Enterprise | 1500-3000 PYRAX/day |

*Earnings depend on job availability and market conditions*

### Software

- Ubuntu 20.04+ or Windows 10+
- NVIDIA drivers 525+
- Docker 24+
- PYRAX Desktop or Crucible CLI

## Quick Setup

### Using PYRAX Desktop

1. Download [PYRAX Desktop](https://pyrax.org/downloads)
2. Create/import wallet
3. Go to Settings → Crucible
4. Click "Enable as Provider"
5. Complete GPU verification
6. Start earning

### Using CLI

```bash
# Install Crucible CLI
curl -sSL https://crucible.pyrax.org/install.sh | bash

# Configure
crucible init --wallet YOUR_WALLET_ADDRESS

# Verify GPU
crucible verify-gpu

# Start provider
crucible start
```

## Configuration

### Provider Settings

```yaml
# ~/.crucible/config.yaml
provider:
  wallet: "0x..."
  
gpu:
  devices: [0]        # GPU indices
  maxMemory: 90       # % of VRAM to use
  
jobs:
  acceptedTypes:
    - inference
    - fine-tuning
    - processing
  minPayment: 0.01    # Minimum PYRAX per job
  maxDuration: 3600   # Max job duration (seconds)
  
network:
  port: 9090
  publicIp: auto      # or specify IP
```

### Job Preferences

```yaml
jobs:
  # Accept specific models
  acceptedModels:
    - llama-*
    - stable-diffusion-*
    - whisper-*
  
  # Exclude heavy jobs
  excludedModels:
    - llama-70b       # Too large for my GPU
  
  # Scheduling
  schedule:
    enabled: true
    activeHours: "08:00-22:00"  # Local time
```

## Earnings

### Payment Flow

```
┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐
│   Job   │───►│ Execute │───►│ Verify  │───►│  Paid   │
│ Assigned│    │         │    │         │    │         │
└─────────┘    └─────────┘    └─────────┘    └─────────┘
```

1. Job assigned to your GPU
2. You execute the computation
3. Results verified (if required)
4. Payment released to wallet

### Payment Schedule

- Payments accumulate per job
- Minimum withdrawal: 10 PYRAX
- Automatic withdrawal available

### Viewing Earnings

```bash
# CLI
crucible earnings --period month

# Output
Total Earnings: 2,450 PYRAX
Jobs Completed: 1,234
Average per Job: 1.98 PYRAX
Uptime: 98.5%
```

## Reputation System

### Reputation Factors

| Factor | Weight | Description |
|--------|--------|-------------|
| Uptime | 30% | Time available for jobs |
| Success Rate | 30% | Jobs completed successfully |
| Speed | 20% | Faster than estimate = bonus |
| Accuracy | 20% | Verified result quality |

### Reputation Benefits

| Score | Benefits |
|-------|----------|
| 0-50 | Basic jobs only |
| 50-75 | Standard jobs, normal priority |
| 75-90 | Premium jobs, higher priority |
| 90-100 | Enterprise jobs, top priority |

### Improving Reputation

- Maintain high uptime (99%+)
- Don't cancel jobs
- Ensure accurate results
- Upgrade hardware for speed

## Monitoring

### Dashboard

```bash
crucible status

# Output
GPU: NVIDIA RTX 4090
Status: Active
Current Job: job_abc123 (llama-7b inference)
Progress: 45%
Queue: 3 jobs pending
Today: 150 PYRAX earned, 45 jobs
```

### Metrics

```bash
crucible metrics

# Prometheus endpoint
curl http://localhost:9091/metrics
```

### Alerts

```yaml
# config.yaml
alerts:
  enabled: true
  discord:
    webhook: "https://discord.com/api/webhooks/..."
  conditions:
    - type: gpu_temp
      threshold: 85
    - type: job_failure
      count: 3
```

## Troubleshooting

### GPU Not Detected

```bash
# Check NVIDIA driver
nvidia-smi

# Check Docker GPU access
docker run --gpus all nvidia/cuda:12.0-base nvidia-smi
```

### Jobs Failing

- Check GPU memory usage
- Verify model compatibility
- Check logs: `crucible logs`
- Reduce `maxMemory` setting

### Low Earnings

- Check uptime (aim for 99%+)
- Upgrade GPU for more jobs
- Check job preferences aren't too restrictive
- Verify network connectivity

### Network Issues

```bash
# Test connectivity
crucible network-test

# Check port
crucible port-check
```

## Best Practices

1. **Maximize uptime** — Consistent availability earns more
2. **Keep software updated** — Latest versions improve efficiency
3. **Monitor temperatures** — Prevent throttling
4. **Fast internet** — Reduces job transfer time
5. **SSD storage** — Faster model loading

## Staking for Priority

Stake PYRAX to get priority job assignments:

| Stake Amount | Priority Boost |
|--------------|----------------|
| 1,000 | +5% |
| 10,000 | +15% |
| 100,000 | +30% |

```bash
crucible stake --amount 10000
```

---

:::tip Maximize Earnings
1. Run 24/7 with high uptime
2. Upgrade to premium GPUs
3. Stake PYRAX for priority
4. Maintain high reputation
:::
