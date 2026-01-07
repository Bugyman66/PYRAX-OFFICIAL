# GPU Mining Basics

This guide explains the fundamentals of GPU mining for PYRAX. Whether you're new to cryptocurrency mining or an experienced miner, this page covers what you need to know.

## What is GPU Mining?

GPU mining uses your computer's graphics card (GPU) to solve complex mathematical puzzles. When your GPU finds a solution, you earn cryptocurrency rewards.

### Why GPUs?

Graphics cards are ideal for cryptocurrency mining because:
- **Parallel processing** — GPUs have thousands of cores that work simultaneously
- **High throughput** — Process many calculations per second
- **Affordable** — Consumer GPUs are accessible to everyone
- **Versatile** — Can switch between different cryptocurrencies

### How Mining Works (Simple Explanation)

Imagine a lottery where you need to guess a specific number. The only way to win is by trying random guesses very quickly. Your GPU is like a super-fast lottery ticket machine:

1. **Get the puzzle** — Network announces what number range to search
2. **Make guesses** — GPU tries billions of random numbers
3. **Check results** — Each guess is verified against the target
4. **Win reward** — Correct guess earns you the block reward

The faster your GPU can guess (measured in "hashrate"), the more likely you are to win.

## PYRAX Mining Specifics

### The KAWPOW Algorithm

PYRAX uses the KAWPOW mining algorithm, specifically designed for GPU mining:

| Feature | Benefit |
|---------|---------|
| **Memory-hard** | Requires lots of GPU memory, resisting ASICs |
| **Random program execution** | Different each block, favors GPUs |
| **Efficient** | Optimized for modern GPU architectures |
| **ASIC-resistant** | Keeps mining accessible to everyone |

### Mining Parameters

| Parameter | Value |
|-----------|-------|
| **Algorithm** | KAWPOW |
| **Block Time** | ~60 seconds |
| **Block Reward** | Variable (decreasing emission) |
| **Difficulty Adjustment** | Every block |
| **DAG Size** | Grows over time (~4GB minimum) |

## Getting Started

### Step 1: Check Your Hardware

**Minimum Requirements:**
- NVIDIA GTX 1060 6GB or AMD RX 580 8GB
- 8 GB system RAM
- 50 GB free storage
- Stable internet connection

**Recommended:**
- NVIDIA RTX 3070+ or AMD RX 6800+
- 16 GB+ system RAM
- SSD storage
- High-speed internet

### Step 2: Get a Wallet

Before mining, you need a PYRAX wallet address to receive rewards:

1. Download PYRAX Desktop from [pyrax.org/downloads](https://pyrax.org/downloads)
2. Create a new wallet
3. **Securely backup your seed phrase**
4. Copy your wallet address

### Step 3: Choose Mining Software

**Official PYRAX Desktop** (Recommended for beginners)
- Built-in mining with simple interface
- Automatic pool selection
- Dual-Stream Mining support
- Download: [pyrax.org/downloads](https://pyrax.org/downloads)

**Third-Party Miners** (For advanced users)
- T-Rex Miner (NVIDIA)
- TeamRedMiner (AMD)
- NBMiner (Both)

### Step 4: Join a Mining Pool

Solo mining is possible but unlikely to find blocks alone. Pools combine hashpower for consistent rewards:

**Official Pools:**
- `stratum+tcp://pool.pyrax.org:3333` (Primary)
- `stratum+tcp://pool2.pyrax.org:3333` (Backup)

**Community Pools:**
- Various third-party pools available
- Check [Mining Pools](./mining-pools) for full list

### Step 5: Start Mining

**Using PYRAX Desktop:**
1. Open PYRAX Desktop
2. Go to Mining tab
3. Enter your wallet address
4. Click "Start Mining"

**Using T-Rex Miner (example):**
```bash
t-rex -a kawpow -o stratum+tcp://pool.pyrax.org:3333 -u YOUR_WALLET_ADDRESS -p x
```

## Understanding Hashrate

Hashrate measures your mining speed — how many guesses your GPU makes per second.

### Hashrate Units

| Unit | Value | Example |
|------|-------|---------|
| H/s | 1 hash/second | Very slow |
| KH/s | 1,000 H/s | Basic GPU |
| MH/s | 1,000,000 H/s | Standard GPU |
| GH/s | 1,000,000,000 H/s | Mining farm |

### Expected Hashrates

| GPU Model | Approx. Hashrate | Power Draw |
|-----------|------------------|------------|
| GTX 1060 6GB | ~12 MH/s | 80W |
| GTX 1070 | ~18 MH/s | 120W |
| GTX 1080 Ti | ~25 MH/s | 180W |
| RTX 3060 | ~22 MH/s | 115W |
| RTX 3070 | ~30 MH/s | 130W |
| RTX 3080 | ~42 MH/s | 220W |
| RTX 4090 | ~65 MH/s | 320W |
| RX 580 8GB | ~14 MH/s | 135W |
| RX 6800 | ~35 MH/s | 150W |

*Values are approximate and depend on settings and drivers*

## Pool vs Solo Mining

### Pool Mining (Recommended)

**How it works:**
- Join a pool with other miners
- Pool combines everyone's hashpower
- When pool finds a block, reward is split
- Smaller but consistent payouts

**Pros:**
- Regular, predictable income
- Lower variance
- Don't need massive hashpower

**Cons:**
- Pool takes a small fee (usually 1-2%)
- Slightly less total earnings long-term

### Solo Mining

**How it works:**
- Mine independently
- Keep entire block reward when you find one
- Could go long periods without finding blocks

**Pros:**
- No pool fees
- Full block reward
- Complete independence

**Cons:**
- Highly inconsistent income
- Could mine for weeks without reward
- Only viable with significant hashpower

**Recommendation:** Unless you have >1% of network hashrate, use pool mining.

## Optimizing Your Mining

### GPU Settings

1. **Core Clock**
   - KAWPOW is memory-intensive
   - Slight core underclock often helps

2. **Memory Clock**
   - Increase memory clock for better hashrate
   - Be careful of instability

3. **Power Limit**
   - Reduce power limit to improve efficiency
   - 70-80% often optimal

4. **Fan Speed**
   - Keep GPU cool (under 75°C recommended)
   - Consider custom fan curves

### Software Optimization

- **Update drivers** — Latest GPU drivers improve performance
- **Close background apps** — More resources for mining
- **Use mining OS** — HiveOS or similar for dedicated rigs
- **Enable Compute Mode** — For AMD GPUs

### Overclocking Example (RTX 3080)

| Setting | Value | Effect |
|---------|-------|--------|
| Power Limit | 75% | Reduces power, minimal hashrate loss |
| Core Clock | -100 MHz | Lower heat, slight efficiency gain |
| Memory Clock | +1000 MHz | Higher hashrate |
| Fan Speed | 80% | Keeps temperatures low |

**Result:** ~40 MH/s at 200W instead of ~42 MH/s at 320W

:::warning
Overclocking can damage hardware if done incorrectly. Start conservative and test stability.
:::

## Electricity and Profitability

### Calculating Costs

Mining profitability depends on:
- Your hashrate
- Electricity cost
- PYRAX price
- Network difficulty

**Formula:**
```
Daily Profit = (Daily PYRAX Earnings × PYRAX Price) - (Power Draw × 24h × Electricity Rate)
```

**Example:**
- Hashrate: 30 MH/s (RTX 3070)
- Daily PYRAX: 50 PYRAX
- PYRAX Price: $0.01
- Power: 130W
- Electricity: $0.10/kWh

```
Revenue: 50 × $0.01 = $0.50
Cost: 0.13 kW × 24h × $0.10 = $0.31
Profit: $0.50 - $0.31 = $0.19/day
```

### Profitability Tips

1. **Electricity matters** — Find the lowest rates possible
2. **Efficiency over raw power** — Lower power settings often more profitable
3. **Consider heat value** — Mining heat can offset heating costs
4. **Hold or sell** — Market timing affects returns

## Troubleshooting

### Low Hashrate

- Update GPU drivers
- Check thermal throttling (high temps)
- Ensure adequate power supply
- Verify mining software settings

### Rejected Shares

- Check internet stability
- Verify correct pool settings
- Update mining software
- Reduce overclock if unstable

### GPU Crashes

- Reduce memory overclock
- Increase power limit
- Check cooling/thermal paste
- Test in different slot

---

:::tip Ready for More?
Continue to [Mining Setup](./mining-setup) for detailed installation instructions, or check [Mining Pools](./mining-pools) to find the best pool for you.
:::
