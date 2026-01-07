# Mining Pools

Mining pools combine the computing power of multiple miners to find blocks more consistently. This guide covers PYRAX mining pools and how to choose the best one for you.

## What is a Mining Pool?

Instead of mining alone (solo mining), pools allow miners to work together:
- Combined hashpower finds blocks faster
- Rewards are split based on contribution
- More consistent, predictable income

**Analogy:** Solo mining is like playing the lottery alone. Pool mining is like a lottery syndicate — you win smaller amounts more often.

## Official PYRAX Pools

### Primary Pool
| Property | Value |
|----------|-------|
| Address | `stratum+tcp://pool.pyrax.org:3333` |
| Fee | 1% |
| Payout | PPLNS |
| Minimum Payout | 10 PYRAX |

### Backup Pool
| Property | Value |
|----------|-------|
| Address | `stratum+tcp://pool2.pyrax.org:3333` |
| Fee | 1% |
| Payout | PPLNS |
| Minimum Payout | 10 PYRAX |

## Community Pools

*Community pools will be listed after mainnet launch. Check Discord for recommendations.*

## Choosing a Pool

### Factors to Consider

| Factor | Why It Matters |
|--------|----------------|
| **Pool hashrate** | Larger = more frequent blocks |
| **Fee** | Lower = more earnings |
| **Minimum payout** | Lower = faster access to funds |
| **Location** | Closer = lower latency |
| **Uptime** | Better = more consistent mining |
| **Payout scheme** | Affects reward distribution |

### Pool Size Trade-offs

**Large Pools:**
- ✅ Frequent, consistent payouts
- ✅ Professional infrastructure
- ❌ Contributes to centralization
- ❌ May have higher fees

**Small Pools:**
- ✅ Supports decentralization
- ✅ May have lower fees
- ❌ Less frequent payouts
- ❌ Higher variance

**Recommendation:** Start with official pools, explore community pools as you learn.

## Payout Schemes

### PPLNS (Pay Per Last N Shares)
- Payment based on shares submitted in recent window
- Rewards loyal, consistent miners
- Discourages pool hopping
- Most common for PYRAX pools

### PPS (Pay Per Share)
- Fixed payment for each share
- Immediate, predictable income
- Pool takes on variance risk
- Usually higher fees

### PROP (Proportional)
- Simple split based on shares per block
- Fair but encourages pool hopping
- Less common now

## Connecting to a Pool

### Configuration

**Pool Address Format:**
```
stratum+tcp://[pool_address]:[port]
```

**Example:**
```
stratum+tcp://pool.pyrax.org:3333
```

### Worker Names

Identify different mining rigs:
```
YOUR_WALLET_ADDRESS.worker_name
```

**Examples:**
```
0x1234...5678.rig1
0x1234...5678.gaming_pc
0x1234...5678.basement
```

### Difficulty Settings

Some pools offer multiple ports for different hashrates:

| Port | Difficulty | Best For |
|------|------------|----------|
| 3333 | Auto-adjust | Most miners |
| 3334 | Low | Single GPU |
| 3335 | High | Mining farms |

## Pool Dashboard

Most pools provide a web dashboard. Access by visiting the pool website and entering your wallet address.

### Dashboard Features

- **Hashrate graph** — Your mining speed over time
- **Shares submitted** — Work units sent to pool
- **Estimated earnings** — Projected rewards
- **Payout history** — Past payments
- **Worker status** — Individual rig statistics

### Key Metrics

| Metric | Meaning |
|--------|---------|
| **Reported Hashrate** | What your miner says |
| **Effective Hashrate** | Calculated from shares |
| **Valid Shares** | Accepted work units |
| **Stale Shares** | Late submissions |
| **Invalid Shares** | Rejected work |

**Note:** Effective hashrate fluctuates. Average over 24 hours is most accurate.

## Optimizing Pool Performance

### Reduce Latency

- Choose geographically close pools
- Use wired internet connection
- Avoid network congestion

### Minimize Stale Shares

- Lower latency connection
- Increase miner aggressiveness (if option exists)
- Avoid overloaded pools

### Worker Configuration

- Use descriptive worker names
- Set appropriate difficulty
- Enable failover pools

## Failover Configuration

Set backup pools in case primary fails:

**T-Rex example:**
```batch
t-rex -a kawpow ^
  -o stratum+tcp://pool.pyrax.org:3333 ^
  -o stratum+tcp://pool2.pyrax.org:3333 ^
  -u YOUR_WALLET_ADDRESS -p x
```

**NBMiner example:**
```batch
nbminer -a kawpow ^
  -o stratum+tcp://pool.pyrax.org:3333 ^
  -o stratum+tcp://pool2.pyrax.org:3333 ^
  -u YOUR_WALLET_ADDRESS
```

## Pool Security

### Wallet Address Only

Pools only need your public wallet address. Never share:
- Private keys
- Seed phrases
- Wallet passwords

### HTTPS Dashboards

- Access pool dashboards via HTTPS
- Verify you're on the official pool website
- Bookmark legitimate URLs

### Payout Addresses

- Verify your payout address is correct
- Some pools allow address locking for security

## Running Your Own Pool

For advanced users with significant hashpower:

### Requirements
- Technical knowledge (Linux, networking)
- Server infrastructure
- PYRAX node running
- Pool software (open source available)

### Considerations
- Significant setup effort
- Need miners to join
- Responsibility for uptime
- Usually only worthwhile for larger operations

## Frequently Asked Questions

### How often will I get paid?
Depends on pool's minimum payout and your hashrate. Could be daily to weekly.

### Why is my effective hashrate different from reported?
Normal variance. Effective hashrate averages out over time.

### Can I mine on multiple pools?
Yes, but splitting hashpower reduces efficiency. Better to pick one.

### What happens if the pool goes down?
Use failover configuration to automatically switch to backup.

### Do I need to register with pools?
Most pools are anonymous — just start mining with your wallet address.

---

:::tip Pool Recommendations
- **Beginners**: Official pool at `pool.pyrax.org:3333`
- **Experienced**: Explore community pools for variety
- **Large miners**: Consider pool diversity for decentralization
:::
