# Crucible Overview

Crucible is PYRAX's decentralized AI computing platform. This guide provides a technical overview for developers.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Crucible Platform                         │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐  │
│  │  Job Queue   │───►│   Matcher    │───►│  Executor    │  │
│  │              │    │   Service    │    │   Network    │  │
│  └──────────────┘    └──────────────┘    └──────────────┘  │
│         ▲                                       │           │
│         │                                       ▼           │
│  ┌──────────────┐                       ┌──────────────┐   │
│  │     API      │                       │  Verifier    │   │
│  │   Gateway    │                       │   Layer      │   │
│  └──────────────┘                       └──────────────┘   │
│         ▲                                       │           │
│         │                                       ▼           │
│  ┌──────────────┐                       ┌──────────────┐   │
│  │    Users     │                       │   Payment    │   │
│  │  (Job Subs)  │                       │   Escrow     │   │
│  └──────────────┘                       └──────────────┘   │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## Core Components

### Job Queue
- Receives job submissions via API
- Validates job specifications
- Prioritizes based on payment/urgency
- Manages job lifecycle

### Matcher Service
- Matches jobs to suitable GPU providers
- Considers: hardware, reputation, price, location
- Optimizes for efficiency and reliability
- Load balances across network

### Executor Network
- GPU providers running Crucible client
- Isolated execution environments
- Supports multiple AI frameworks
- Reports results and metrics

### Verifier Layer
- Validates computation results
- Multi-node consensus for critical jobs
- Zero-knowledge proofs (optional)
- Dispute resolution

### Payment Escrow
- Holds funds during job execution
- Automatic release on verification
- Slashing for misbehavior
- Refunds for failed jobs

## Job Types

| Type | Description | Typical Duration |
|------|-------------|------------------|
| Inference | Run trained models | Seconds-minutes |
| Fine-tuning | Adapt existing models | Hours |
| Training | Train from scratch | Hours-days |
| Processing | General GPU compute | Varies |

## Supported Frameworks

| Framework | Version | Status |
|-----------|---------|--------|
| PyTorch | 2.0+ | Supported |
| TensorFlow | 2.x | Supported |
| ONNX Runtime | 1.15+ | Supported |
| Hugging Face | Latest | Supported |

## Supported Models

### Built-in Models
- LLaMA variants (7B, 13B, 70B)
- Stable Diffusion
- Whisper
- CLIP
- Many more...

### Custom Models
- Upload your own models
- ONNX format preferred
- Size limits apply

## API Overview

### REST API

```
Base URL: https://crucible.pyrax.org/api/v1
```

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/jobs` | POST | Submit new job |
| `/jobs/{id}` | GET | Get job status |
| `/jobs/{id}/results` | GET | Get results |
| `/jobs/{id}/cancel` | POST | Cancel job |
| `/models` | GET | List available models |
| `/providers` | GET | List GPU providers |

### WebSocket

```
wss://crucible.pyrax.org/ws
```

Real-time job status updates.

## Pricing

Jobs are priced based on:
- GPU time (per second)
- GPU type (better = higher)
- Model size
- Priority level

### Example Pricing

| GPU Tier | Price/hour |
|----------|------------|
| Entry (RTX 3060) | 5 PYRAX |
| Standard (RTX 3080) | 10 PYRAX |
| Premium (RTX 4090) | 25 PYRAX |
| Enterprise (A100) | 50 PYRAX |

## Security

### Execution Isolation
- Docker containers
- No persistent storage
- Network isolation
- Resource limits

### Data Protection
- Encrypted in transit
- Deleted after completion
- Optional end-to-end encryption

### Verification
- Result validation
- Reputation system
- Economic incentives

## Integration Options

### SDK Integration
- JavaScript/Python SDKs
- High-level abstractions
- Error handling built-in

### Direct API
- REST API for custom integrations
- WebSocket for real-time updates
- Full control

### Smart Contract
- On-chain job submission
- Programmable compute
- DeFi integrations

## Rate Limits

| Tier | Jobs/hour | Concurrent |
|------|-----------|------------|
| Free | 10 | 2 |
| Basic | 100 | 10 |
| Pro | 1000 | 50 |
| Enterprise | Unlimited | Custom |

---

:::info Next Steps
- [Crucible Integration](./crucible-integration) — Integration guide
- [Job Submission](./job-submission) — Submit AI jobs
- [GPU Provider](./gpu-provider) — Become a provider
:::
