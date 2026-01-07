# Job Submission

Complete guide for submitting AI compute jobs to Crucible.

## Job Structure

```javascript
{
  // Required
  model: string,       // Model identifier
  input: object,       // Model-specific input

  // Optional
  options: {
    maxBudget: number,      // Max PYRAX to spend
    priority: string,       // low, normal, high
    timeout: number,        // Seconds before timeout
    gpuType: string,        // Preferred GPU type
    gpuMemory: number,      // Minimum VRAM (GB)
    region: string,         // Preferred region
    verification: string,   // none, single, multi
    webhook: string,        // Callback URL
    metadata: object        // Custom metadata
  }
}
```

## Model Types

### Text Generation

```javascript
await crucible.submitJob({
  model: 'llama-7b',  // or llama-13b, llama-70b
  input: {
    prompt: 'Write a story about a robot',
    max_tokens: 1000,
    temperature: 0.7,
    top_p: 0.9,
    stop: ['\n\n']
  }
});
```

### Image Generation

```javascript
await crucible.submitJob({
  model: 'stable-diffusion-xl',
  input: {
    prompt: 'A futuristic city at night',
    negative_prompt: 'blurry, low quality',
    width: 1024,
    height: 1024,
    steps: 30,
    guidance_scale: 7.5,
    seed: 12345  // Optional, for reproducibility
  }
});
```

### Speech Recognition

```javascript
await crucible.submitJob({
  model: 'whisper-large',
  input: {
    audio: audioBase64,  // Base64 encoded audio
    language: 'en',
    task: 'transcribe'   // or 'translate'
  }
});
```

### Image Understanding

```javascript
await crucible.submitJob({
  model: 'clip',
  input: {
    image: imageBase64,
    texts: ['a cat', 'a dog', 'a bird']
  }
});
```

### Custom Models

```javascript
// First upload your model
const modelId = await crucible.uploadModel({
  file: modelBuffer,
  format: 'onnx',      // onnx, pytorch, tensorflow
  name: 'my-classifier',
  inputSchema: {...},
  outputSchema: {...}
});

// Then use it
await crucible.submitJob({
  model: modelId,
  input: {
    data: inputTensor
  }
});
```

## Priority Levels

| Priority | Queue Position | Price Multiplier |
|----------|---------------|------------------|
| low | Back of queue | 0.8x |
| normal | Standard | 1.0x |
| high | Front of queue | 1.5x |

## GPU Selection

### By Type

```javascript
{
  options: {
    gpuType: 'rtx-4090'  // Specific GPU
  }
}
```

### By Memory

```javascript
{
  options: {
    gpuMemory: 24  // Minimum 24GB VRAM
  }
}
```

### Available GPUs

| GPU | VRAM | Identifier |
|-----|------|------------|
| RTX 3060 | 12 GB | rtx-3060 |
| RTX 3080 | 10 GB | rtx-3080 |
| RTX 4090 | 24 GB | rtx-4090 |
| A100 | 40/80 GB | a100 |

## Verification Modes

### None
- Fastest, cheapest
- No result verification
- Best for non-critical jobs

### Single
- One provider executes
- Basic sanity check
- Good balance

### Multi
- Multiple providers execute
- Results compared
- Highest reliability
- 2-3x cost

## Handling Large Inputs

### File Upload

```javascript
// Upload file first
const fileId = await crucible.uploadFile(largeBuffer);

// Reference in job
await crucible.submitJob({
  model: 'whisper-large',
  input: {
    audioFile: fileId
  }
});
```

### Chunking

```javascript
// For very long text
const chunks = splitIntoChunks(longText, 4000);

const jobs = await Promise.all(
  chunks.map(chunk =>
    crucible.submitJob({
      model: 'llama-7b',
      input: { prompt: chunk }
    })
  )
);

const results = await Promise.all(
  jobs.map(j => j.waitForCompletion())
);
```

## Job Management

### Cancel Job

```javascript
await crucible.cancelJob(jobId);
```

### Retry Failed Job

```javascript
const originalJob = await crucible.getJob(jobId);
const newJob = await crucible.retryJob(jobId);
```

### List Jobs

```javascript
const jobs = await crucible.listJobs({
  status: 'completed',
  from: '2024-01-01',
  limit: 100
});
```

## Results Format

### Text Generation

```javascript
{
  output: {
    text: "Generated text here...",
    tokens_used: 150,
    finish_reason: "stop"
  },
  usage: {
    compute_time: 2.5,
    cost: 0.05
  }
}
```

### Image Generation

```javascript
{
  output: {
    images: [
      "data:image/png;base64,..."
    ],
    seed: 12345
  },
  usage: {
    compute_time: 8.2,
    cost: 0.25
  }
}
```

## REST API

### Submit Job

```bash
POST /api/v1/jobs
Content-Type: application/json
Authorization: Bearer YOUR_API_KEY

{
  "model": "llama-7b",
  "input": {
    "prompt": "Hello world"
  }
}
```

### Response

```json
{
  "id": "job_abc123...",
  "status": "queued",
  "createdAt": "2024-01-15T10:00:00Z",
  "estimatedCost": 0.05
}
```

### Get Status

```bash
GET /api/v1/jobs/job_abc123
```

### Get Results

```bash
GET /api/v1/jobs/job_abc123/results
```

---

:::info Related
- [Crucible Integration](./crucible-integration) — SDK usage
- [GPU Provider](./gpu-provider) — Become a provider
:::
