# Crucible Integration

Guide for integrating Crucible AI computing into your applications.

## Quick Start

### JavaScript

```javascript
import { CrucibleClient } from '@pyrax/sdk';

const crucible = new CrucibleClient({
  apiKey: 'your-api-key',
  network: 'mainnet'
});

// Submit a job
const job = await crucible.submitJob({
  model: 'llama-7b',
  input: { prompt: 'Explain quantum computing' },
  maxBudget: 10 // PYRAX
});

// Wait for completion
const result = await job.waitForCompletion();
console.log(result.output);
```

### Python

```python
from pyrax import CrucibleClient

crucible = CrucibleClient(api_key='your-api-key')

job = crucible.submit_job(
    model='llama-7b',
    input={'prompt': 'Explain quantum computing'},
    max_budget=10
)

result = job.wait_for_completion()
print(result.output)
```

## Authentication

### API Key

Get your API key from [crucible.pyrax.org/dashboard](https://crucible.pyrax.org/dashboard).

```javascript
const crucible = new CrucibleClient({
  apiKey: process.env.CRUCIBLE_API_KEY
});
```

### Wallet Signature

For decentralized auth:

```javascript
const crucible = new CrucibleClient({
  wallet: yourWallet // ethers.js wallet
});

await crucible.authenticate();
```

## Job Lifecycle

```
┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐
│ Created │───►│ Queued  │───►│ Running │───►│Complete │
└─────────┘    └─────────┘    └─────────┘    └─────────┘
                    │              │              │
                    ▼              ▼              ▼
               ┌─────────┐   ┌─────────┐   ┌─────────┐
               │Cancelled│   │ Failed  │   │ Expired │
               └─────────┘   └─────────┘   └─────────┘
```

## Job Configuration

### Basic Job

```javascript
const job = await crucible.submitJob({
  model: 'stable-diffusion',
  input: {
    prompt: 'A mountain landscape at sunset',
    negative_prompt: 'blurry, low quality',
    steps: 30
  }
});
```

### Advanced Options

```javascript
const job = await crucible.submitJob({
  model: 'llama-13b',
  input: {
    prompt: 'Write a poem about the ocean',
    max_tokens: 500,
    temperature: 0.7
  },
  options: {
    maxBudget: 50,           // Max PYRAX to spend
    priority: 'high',        // low, normal, high
    timeout: 300,            // Seconds
    gpuType: 'rtx-4090',     // Preferred GPU
    region: 'us-east',       // Preferred region
    verification: 'multi'    // none, single, multi
  }
});
```

### Custom Model

```javascript
// Upload model first
const modelId = await crucible.uploadModel({
  file: modelBuffer,
  format: 'onnx',
  name: 'my-custom-model'
});

// Use custom model
const job = await crucible.submitJob({
  model: modelId,
  input: { data: inputData }
});
```

## Handling Results

### Polling

```javascript
const job = await crucible.submitJob({...});

// Poll for status
let status = await crucible.getJobStatus(job.id);
while (status.state !== 'completed') {
  await sleep(1000);
  status = await crucible.getJobStatus(job.id);
}

const results = await crucible.getJobResults(job.id);
```

### Waiting

```javascript
const job = await crucible.submitJob({...});
const result = await job.waitForCompletion({
  timeout: 300000, // 5 minutes
  pollInterval: 1000
});
```

### WebSocket

```javascript
const crucible = new CrucibleClient({...});

crucible.on('job:update', (event) => {
  console.log(`Job ${event.jobId}: ${event.status}`);
});

crucible.on('job:complete', (event) => {
  console.log('Result:', event.result);
});

const job = await crucible.submitJob({...});
```

### Webhooks

```javascript
const job = await crucible.submitJob({
  model: 'llama-7b',
  input: {...},
  webhook: 'https://your-server.com/crucible-callback'
});
```

Your webhook receives:
```json
{
  "event": "job.completed",
  "jobId": "job_123...",
  "status": "completed",
  "result": {...}
}
```

## Error Handling

```javascript
try {
  const job = await crucible.submitJob({...});
  const result = await job.waitForCompletion();
} catch (error) {
  if (error.code === 'INSUFFICIENT_FUNDS') {
    console.log('Add more PYRAX to your account');
  } else if (error.code === 'JOB_TIMEOUT') {
    console.log('Job timed out, try with longer timeout');
  } else if (error.code === 'MODEL_NOT_FOUND') {
    console.log('Check model name');
  } else {
    console.log('Error:', error.message);
  }
}
```

## Batch Processing

```javascript
const jobs = await Promise.all([
  crucible.submitJob({ model: 'llama-7b', input: { prompt: 'Q1' } }),
  crucible.submitJob({ model: 'llama-7b', input: { prompt: 'Q2' } }),
  crucible.submitJob({ model: 'llama-7b', input: { prompt: 'Q3' } })
]);

const results = await Promise.all(
  jobs.map(job => job.waitForCompletion())
);
```

## Cost Management

### Budget Limits

```javascript
const job = await crucible.submitJob({
  model: 'llama-70b',
  input: {...},
  options: {
    maxBudget: 100 // Will fail if cost exceeds
  }
});
```

### Cost Estimation

```javascript
const estimate = await crucible.estimateCost({
  model: 'llama-7b',
  input: { prompt: '...' }
});

console.log(`Estimated: ${estimate.min} - ${estimate.max} PYRAX`);
```

### Usage Tracking

```javascript
const usage = await crucible.getUsage({
  from: '2024-01-01',
  to: '2024-01-31'
});

console.log(`Total spent: ${usage.totalCost} PYRAX`);
console.log(`Jobs run: ${usage.jobCount}`);
```

---

:::info More Details
- [Job Submission](./job-submission) — Complete job options
- [API Reference](./api-reference) — Full API documentation
:::
