# Dual-Stream Mining

Dual-Stream Mining is PYRAX's revolutionary approach to cryptocurrency mining that allows your GPU to earn from two sources simultaneously. This page explains how it works and why it matters.

## What is Dual-Stream Mining?

Traditional cryptocurrency mining is single-purpose: your GPU solves cryptographic puzzles to produce blocks and earn rewards. That's it.

PYRAX's Dual-Stream Mining changes this by splitting your GPU's work into two parallel streams:

| Stream | Purpose | Reward Source |
|--------|---------|---------------|
| **Stream A** | Block mining (network security) | Block rewards + fees |
| **Stream B** | AI compute jobs | Job payments |

Both streams run simultaneously, meaning your GPU earns from both activities at the same time.

## How It Works

### Stream A: Block Mining

Stream A operates like traditional GPU mining:

1. **Receive work** — Mining pool sends block template
2. **Compute hashes** — GPU calculates KAWPOW hashes
3. **Find solution** — When nonce produces valid hash, submit it
4. **Earn reward** — Receive portion of block reward

The KAWPOW algorithm is specifically designed for GPUs:
- Memory-intensive to resist ASICs
- Efficient use of GPU architecture
- Fair for all GPU types

### Stream B: AI Compute

Stream B is what makes PYRAX unique:

1. **Opt-in enrollment** — Register your GPU with Crucible
2. **Receive AI jobs** — System matches jobs to your GPU
3. **Process workload** — Run inference, training, or processing
4. **Submit results** — Return completed computation
5. **Receive payment** — Get paid in PYRAX

AI jobs include:
- Machine learning inference
- Neural network training
- Image/video processing
- Natural language processing
- Scientific simulations

## The Technical Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                       Your GPU                              │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────────────┐     ┌─────────────────────┐       │
│  │     Stream A        │     │     Stream B        │       │
│  │   Block Mining      │     │    AI Compute       │       │
│  │                     │     │                     │       │
│  │  ┌───────────────┐  │     │  ┌───────────────┐  │       │
│  │  │ KAWPOW Hashing│  │     │  │  AI Workload  │  │       │
│  │  │    Engine     │  │     │  │   Executor    │  │       │
│  │  └───────────────┘  │     │  └───────────────┘  │       │
│  │         │           │     │         │           │       │
│  │         ▼           │     │         ▼           │       │
│  │  Block Rewards      │     │   Job Payments      │       │
│  └─────────────────────┘     └─────────────────────┘       │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Resource Allocation

Your GPU's resources are intelligently divided:

| Mode | Stream A | Stream B | Best For |
|------|----------|----------|----------|
| **Mining Priority** | 80% | 20% | When block rewards are high |
| **Balanced** | 50% | 50% | Normal operation |
| **AI Priority** | 20% | 80% | When AI demand is high |
| **Auto** | Dynamic | Dynamic | Optimal earnings (recommended) |

The **Auto** mode automatically adjusts allocation based on:
- Current block difficulty
- Available AI jobs and their payment rates
- Your GPU's capabilities
- Network conditions

## Benefits of Dual-Stream Mining

### Higher Total Earnings
By working on two income streams, your GPU potentially earns more than single-purpose mining:

| Mining Type | Block Rewards | AI Payments | Total |
|-------------|---------------|-------------|-------|
| Traditional | $100 | $0 | $100 |
| PYRAX Dual-Stream | $70 | $60 | $130 |

*Example figures for illustration — actual earnings vary*

### Income Stability
When one stream dips, the other often compensates:
- Block rewards drop? AI demand might be high
- AI jobs scarce? Focus on block mining
- Market crash? AI utility maintains some value

### Future-Proof
As AI demand grows exponentially, Stream B becomes increasingly valuable:
- More AI adoption = more jobs
- More jobs = higher competition for GPUs
- Higher competition = better payment rates

### Meaningful Work
Your GPU contributes to real progress:
- Scientific research
- Medical image analysis
- Climate modeling
- Language translation
- And much more

## Getting Started with Dual-Stream

### Requirements

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| GPU | NVIDIA GTX 1060 6GB | RTX 3080+ |
| VRAM | 6 GB | 12+ GB |
| RAM | 8 GB | 16+ GB |
| Internet | 10 Mbps stable | 50+ Mbps |
| OS | Windows 10 / Ubuntu 20.04 | Latest versions |

### Setup Steps

1. **Download PYRAX Desktop**
   - Get the official application from [pyrax.org/downloads](https://pyrax.org/downloads)

2. **Configure Mining**
   - Enter your wallet address
   - Select mining pool or solo mining
   - Set Stream A parameters

3. **Enable AI Compute**
   - Go to Settings → Crucible
   - Click "Enable AI Computing"
   - Complete GPU verification
   - Set Stream B preferences

4. **Start Mining**
   - Click "Start Dual-Stream Mining"
   - Monitor both streams in dashboard
   - Earnings deposit to your wallet

## Stream B Job Types

### Tier 1: Light Jobs
- **Examples**: Text classification, sentiment analysis
- **Duration**: Seconds to minutes
- **Payment**: Low per job, high volume
- **GPU Load**: Light

### Tier 2: Medium Jobs
- **Examples**: Image recognition, audio processing
- **Duration**: Minutes to hours
- **Payment**: Medium
- **GPU Load**: Moderate

### Tier 3: Heavy Jobs
- **Examples**: Model training, video analysis
- **Duration**: Hours to days
- **Payment**: High
- **GPU Load**: Intensive

You can configure which tiers your GPU accepts based on your preferences.

## Earnings Optimization

### Tips for Maximum Earnings

1. **Use Auto Mode** — Let the system optimize allocation
2. **Stay Online** — Consistent uptime earns more jobs
3. **Update Regularly** — New versions improve efficiency
4. **Fast Internet** — Reduces job transfer times
5. **Cool Your GPU** — Prevents thermal throttling

### Earnings Calculator

Estimate your potential earnings:

| Your GPU | Est. Stream A/day | Est. Stream B/day | Total/day |
|----------|-------------------|-------------------|-----------|
| RTX 3060 | ~$2-4 | ~$1-3 | ~$3-7 |
| RTX 3080 | ~$4-8 | ~$3-6 | ~$7-14 |
| RTX 4090 | ~$8-15 | ~$6-12 | ~$14-27 |

*Estimates based on average conditions — actual results vary*

## Frequently Asked Questions

### Does Dual-Stream reduce my block mining hashrate?
Yes, slightly. When Stream B is active, some GPU resources are allocated to AI jobs. However, the total earnings typically exceed what you'd make from 100% block mining.

### Can I disable Stream B?
Yes. You can run Stream A only if you prefer pure block mining. However, you'd miss out on additional earnings.

### Are AI jobs always available?
Job availability varies based on demand. During low periods, more resources automatically shift to Stream A.

### Is my data safe?
AI jobs are processed in isolated environments. Your GPU computes but doesn't store sensitive data.

---

:::tip Start Mining
Ready to try Dual-Stream Mining? Check out the [Mining Setup Guide](./mining-setup) for step-by-step instructions.
:::
