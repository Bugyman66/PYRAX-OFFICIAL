# AI Compute & Crucible

Crucible is PYRAX's decentralized AI computing platform. It connects people who need GPU computing power with miners who have GPUs to spare. This page explains how it works and why it matters.

## What is Crucible?

Crucible is a marketplace for AI computing power. Think of it like Uber, but instead of rides, people are buying and selling GPU time for artificial intelligence tasks.

### The Traditional Problem

Running AI workloads is expensive:
- **AWS GPU instances**: $3-30+ per hour
- **Google Cloud TPUs**: $4-8+ per hour
- **Azure GPU VMs**: $2-15+ per hour

Small businesses, researchers, and developers often can't afford these costs. They're priced out of the AI revolution.

### The Crucible Solution

Crucible connects:
- **Job Submitters** — People who need computing done
- **GPU Providers** — Miners willing to do the computing
- **The Network** — Ensures fair, accurate transactions

**Result**: 40-70% lower costs than centralized cloud providers

## How Crucible Works

### For Job Submitters

1. **Create a job** — Specify what you need computed
2. **Set a budget** — How much PYRAX you'll pay
3. **Submit to Crucible** — Job enters the queue
4. **Matching** — System finds suitable GPUs
5. **Processing** — GPU providers compute your job
6. **Verification** — Results are validated
7. **Delivery** — You receive results, miners get paid

### For GPU Providers (Miners)

1. **Enable Crucible** — Opt-in through PYRAX Desktop
2. **Set preferences** — Which job types you'll accept
3. **Receive jobs** — System sends matching work
4. **Process** — Your GPU runs the computation
5. **Submit results** — Return completed work
6. **Get paid** — PYRAX deposited to your wallet

## Job Types

Crucible supports various AI workloads:

### Inference Jobs
Running trained AI models on new data:
- Image classification ("What's in this photo?")
- Text generation (ChatGPT-style responses)
- Object detection (Finding items in videos)
- Speech recognition (Converting audio to text)

**Typical Payment**: 0.01-0.10 PYRAX per job

### Training Jobs
Teaching AI models from data:
- Fine-tuning language models
- Training image classifiers
- Creating custom models

**Typical Payment**: 1-100+ PYRAX per job (longer, more complex)

### Processing Jobs
General GPU-accelerated tasks:
- Video encoding/transcoding
- Scientific simulations
- Rendering
- Data analysis

**Typical Payment**: Varies by complexity

## The Economics

### Pricing Model

Jobs are priced based on:
- GPU time required
- GPU type needed (better GPU = higher price)
- Priority level (faster = costs more)
- Complexity (memory/compute requirements)

### Example Pricing

| Job Type | Traditional Cloud | Crucible | Savings |
|----------|------------------|----------|---------|
| Image inference (1000 images) | $5-10 | $2-4 | 50-60% |
| Small model fine-tuning | $50-100 | $20-40 | 60% |
| Video processing (1 hour) | $15-30 | $5-12 | 55-65% |

### How Miners Earn

GPU providers earn based on:
- Time their GPU is working
- GPU capability (better = more)
- Job complexity
- Reliability score (consistent = bonus)

**Example monthly earnings (RTX 3080):**
- Light activity: 500-1,000 PYRAX
- Moderate activity: 1,000-2,000 PYRAX
- High activity: 2,000-4,000 PYRAX

*Earnings depend on job availability and market conditions*

## Verification & Trust

How do we ensure honest computation?

### Multi-Node Consensus
Critical jobs run on multiple GPUs:
- Same job → 3+ providers
- Results compared
- Majority result accepted
- Outliers investigated

### Proof of Compute
GPUs generate cryptographic proofs:
- Proves work was actually done
- Verifiable on-chain
- Can't fake results

### Reputation System
Providers build reputation over time:
- Successful jobs increase score
- Failed/incorrect results decrease score
- Higher reputation = more job access

### Slashing
Bad actors face penalties:
- Wrong results = stake slashed
- Repeated issues = removal
- Stakes locked as collateral

## Getting Started

### As a Job Submitter

**Requirements:**
- PYRAX tokens (for payment)
- Job specification (what you need computed)

**Steps:**
1. Visit [crucible.pyrax.org](https://crucible.pyrax.org)
2. Connect your wallet
3. Create new job
4. Upload data/model
5. Set parameters and budget
6. Submit and wait for results

### As a GPU Provider

**Requirements:**
- Compatible GPU (6GB+ VRAM)
- PYRAX Desktop application
- Stable internet connection

**Steps:**
1. Open PYRAX Desktop
2. Go to Settings → Crucible
3. Click "Enable as Provider"
4. Complete verification
5. Set preferences (job types, schedule)
6. Start earning

## Supported Models & Frameworks

Crucible supports popular AI frameworks:

### Machine Learning
- PyTorch
- TensorFlow
- ONNX Runtime
- Hugging Face Transformers

### Specific Models
- Stable Diffusion (image generation)
- Whisper (speech recognition)
- LLAMA variants (text generation)
- CLIP (image understanding)
- Custom models (user-uploaded)

## Privacy & Security

### Data Protection
- Jobs run in isolated containers
- Data deleted after completion
- No persistent storage on provider GPUs
- Optional encryption for sensitive data

### Code Security
- Sandboxed execution environment
- Limited system access
- Network isolation during processing
- Verified container images only

## Use Cases

### Researchers
Academic researchers use Crucible for:
- Training models on limited budgets
- Running large-scale experiments
- Processing research datasets

### Startups
Small companies use Crucible for:
- Prototyping AI features
- Processing user requests
- Scaling without infrastructure

### Artists & Creators
Creative professionals use Crucible for:
- AI image generation
- Video enhancement
- Audio processing
- Content creation

### Developers
Software developers use Crucible for:
- Testing AI integrations
- Running inference APIs
- Model experimentation

## Crucible vs. Cloud Providers

| Feature | AWS/GCP/Azure | Crucible |
|---------|---------------|----------|
| Cost | $$$ | $ |
| Setup | Complex | Simple |
| Minimum commitment | Often hourly | Per-job |
| Decentralized | No | Yes |
| Privacy | Provider sees data | Distributed |
| Geographic diversity | Limited regions | Global |

## Future Roadmap

Crucible continues to evolve:

### Near-term
- More supported model types
- Improved job matching
- Mobile GPU support

### Medium-term
- Confidential computing (encrypted processing)
- Cross-chain payment support
- Enterprise features

### Long-term
- Federated learning support
- Real-time inference
- Hardware-specific optimization

---

:::tip Start Using Crucible
- **Job Submitters**: Visit [crucible.pyrax.org](https://crucible.pyrax.org)
- **GPU Providers**: Enable in [PYRAX Desktop](https://pyrax.org/downloads)
- **Developers**: Check [Crucible API Docs](/developers/crucible-overview)
:::
