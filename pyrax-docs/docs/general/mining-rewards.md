# Mining Rewards

Understanding how PYRAX mining rewards work helps you maximize your earnings and plan your mining operation.

## Reward Sources

PYRAX miners can earn from multiple sources:

| Source | Description | Availability |
|--------|-------------|--------------|
| **Block Rewards** | New PYRAX for finding blocks | Always |
| **Transaction Fees** | Fees from included transactions | Always |
| **AI Compute** | Payments for Crucible jobs | Stream B enabled |
| **Pool Bonuses** | Some pools offer extra incentives | Varies by pool |

## Block Rewards

### How Block Rewards Work

1. Miners compete to solve cryptographic puzzle
2. Winner adds new block to blockchain
3. Winner receives block reward
4. In pools, reward is split among contributors

### Current Block Reward

| Parameter | Value |
|-----------|-------|
| Block time | ~60 seconds |
| Current reward | *Varies - check network* |
| Daily blocks | ~1,440 |

### Emission Schedule

PYRAX uses a smooth decreasing emission:

| Year | Approximate Daily Emission |
|------|---------------------------|
| 1 | ~27 million PYRAX |
| 2 | ~22 million PYRAX |
| 3 | ~17 million PYRAX |
| 5 | ~11 million PYRAX |
| 10 | ~4.5 million PYRAX |

Unlike Bitcoin's sudden halvings, PYRAX decreases gradually (~20% per year).

## Transaction Fees

Every transaction on PYRAX includes a fee. Miners receive:
- 50% of transaction fees (other 50% burned)
- Fees increase earnings during high network activity

## Pool vs Solo Rewards

### Pool Mining Rewards

**How it works:**
1. Pool finds block
2. Block reward goes to pool
3. Pool distributes to miners based on shares
4. Small fee deducted (typically 1-2%)

**Example:**
- Block reward: 1,000 PYRAX
- Your contribution: 1% of pool hashrate
- Pool fee: 1%
- Your reward: 1,000 × 0.01 × 0.99 = 9.9 PYRAX

### Solo Mining Rewards

**How it works:**
1. You find block alone
2. Entire reward is yours
3. No pool fee
4. But blocks are rare for individual miners

**Reality check:**
With 0.1% of network hashrate, you'd find ~1 block per week on average — but could go months without any.

## Calculating Expected Earnings

### Formula

```
Daily Earnings = (Your Hashrate / Network Hashrate) × Daily Block Rewards
```

### Example Calculation

**Given:**
- Your hashrate: 30 MH/s (RTX 3070)
- Network hashrate: 1,000 GH/s (1,000,000 MH/s)
- Daily emission: 20 million PYRAX

**Calculation:**
```
Daily = (30 / 1,000,000) × 20,000,000
Daily = 0.00003 × 20,000,000
Daily = 600 PYRAX
```

**Note:** This is before electricity costs and assumes constant conditions.

### Profitability Factors

| Factor | Effect |
|--------|--------|
| Hashrate increase | ↑ Your earnings |
| Network hashrate increase | ↓ Your earnings |
| Block reward decrease | ↓ All miner earnings |
| PYRAX price increase | ↑ Fiat value of earnings |
| Electricity cost decrease | ↑ Net profit |
| Pool fee decrease | ↑ Your take-home |

## AI Compute Earnings (Stream B)

### Additional Income Stream

Enable Crucible to earn from AI jobs:
- Process machine learning tasks
- Earn PYRAX per job completed
- Runs alongside block mining

### Stream B Earnings

| Job Type | Typical Payment |
|----------|----------------|
| Light inference | 0.01-0.1 PYRAX |
| Image processing | 0.1-1 PYRAX |
| Model training | 1-100+ PYRAX |

### Enabling Stream B

1. Open PYRAX Desktop
2. Go to Settings → Crucible
3. Enable AI Computing
4. Set job preferences
5. Start Dual-Stream Mining

See [AI Compute Guide](./ai-compute) for details.

## Receiving Payments

### Pool Payouts

Pools send payments when you reach the minimum threshold:

| Pool | Minimum Payout |
|------|----------------|
| Official | 10 PYRAX |
| Others | Varies |

**Payout frequency:**
- Depends on your hashrate and pool settings
- Could be daily, every few days, or weekly

### Solo Mining Payouts

Block rewards go directly to your configured wallet address immediately when you find a block.

## Tracking Earnings

### Pool Dashboard
- Real-time hashrate
- Pending balance
- Payment history
- Estimated daily earnings

### PYRAX Desktop
- Integrated earnings tracker
- Both Stream A and B
- Historical statistics

### Block Explorer
- Verify received payments
- Transaction history
- Wallet balance

## Maximizing Rewards

### Hardware Optimization

1. **Efficient overclocking** — More hashrate per watt
2. **Optimal power settings** — Balance speed and cost
3. **Good cooling** — Prevents throttling

### Pool Selection

1. **Low fees** — More goes to you
2. **Low minimum payout** — Access funds sooner
3. **Reliable uptime** — No lost mining time

### Operational Best Practices

1. **Minimize downtime** — Every hour offline is lost earnings
2. **Monitor continuously** — Catch issues quickly
3. **Update software** — Latest versions often more efficient
4. **Use failover pools** — Keep mining if primary goes down

### Stream B Enrollment

Enable AI compute for additional earnings — often 20-50% extra income potential.

## Tax Considerations

Mining rewards may be taxable income:

- Record date and value when received
- Track for capital gains if sold later
- Electricity may be deductible expense
- Consult a tax professional

:::info Tax Tip
Keep detailed records of:
- Mining start date
- All payouts received (dates and amounts)
- PYRAX price at time of receipt
- Electricity costs
- Hardware purchases
:::

## Reward Projection Examples

### Small Miner (1 GPU)

| Metric | Value |
|--------|-------|
| GPU | RTX 3070 (30 MH/s) |
| Est. daily PYRAX | 40-80 |
| Est. daily at $0.01 | $0.40-0.80 |
| Monthly earnings | $12-24 |
| Power cost (~$10/month) | -$10 |
| Net monthly | $2-14 |

### Medium Miner (4 GPUs)

| Metric | Value |
|--------|-------|
| GPUs | 4× RTX 3080 (168 MH/s) |
| Est. daily PYRAX | 200-400 |
| Est. daily at $0.01 | $2-4 |
| Monthly earnings | $60-120 |
| Power cost (~$60/month) | -$60 |
| Net monthly | $0-60 |

*These are rough estimates. Actual results depend on network conditions, PYRAX price, and electricity rates.*

---

:::tip Maximize Your Earnings
1. Enable both Stream A and Stream B
2. Optimize GPU power settings
3. Choose low-fee pools
4. Maintain high uptime
5. Track and reinvest wisely
:::
