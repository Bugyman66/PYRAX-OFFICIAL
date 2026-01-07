# PYRAX Tokenomics

> **Document Version:** 1.0  
> **Last Updated:** 2026-01-04  
> **Status:** Final

---

## Executive Summary

PYRAX is the native utility token of the PYRAX blockchain, a TriStream DAG architecture combining ASIC mining, GPU mining, and ZK proof verification. The token powers all network operations including transaction fees, staking, governance, and AI compute marketplace.

---

## Token Specifications

| Property | Value |
|----------|-------|
| **Token Name** | PYRAX |
| **Symbol** | $PYRAX |
| **Total Supply** | 100,000,000,000 (100 Billion) |
| **Decimals** | 8 |
| **Smallest Unit** | 0.00000001 PYRAX (1 Satoshi equivalent) |
| **Token Standard** | Native + ERC-20 (bridged) |
| **Blockchain** | PYRAX TriStream DAG |

---

## Token Allocation

### Overview

| Pool | Allocation | Amount (PYRAX) | Percentage |
|------|------------|----------------|------------|
| Presale | 15,000,000,000 | 15B | 15% |
| BDAG Community | 10,000,000,000 | 10B | 10% |
| Mining Rewards | 35,000,000,000 | 35B | 35% |
| ZK Prover Rewards | 5,000,000,000 | 5B | 5% |
| Team & Founders | 4,000,000,000 | 4B | 4% |
| Advisors | 3,000,000,000 | 3B | 3% |
| Ecosystem | 10,000,000,000 | 10B | 10% |
| Marketing | 5,000,000,000 | 5B | 5% |
| Liquidity | 10,000,000,000 | 10B | 10% |
| Treasury | 2,000,000,000 | 2B | 2% |
| Reserve | 1,000,000,000 | 1B | 1% |
| **Total** | **100,000,000,000** | **100B** | **100%** |

### Allocation Visualization

```
Mining Rewards ████████████████████████████████████  35%
Presale        ███████████████                       15%
BDAG Community ██████████                            10%
Ecosystem      ██████████                            10%
Liquidity      ██████████                            10%
Marketing      █████                                  5%
ZK Prover      █████                                  5%
Team           ████                                   4%
Advisors       ███                                    3%
Treasury       ██                                     2%
Reserve        █                                      1%
```

---

## Detailed Pool Breakdown

### 1. Presale (15% - 15,000,000,000 PYRAX)

**Purpose:** Public token sale to fund development and establish initial token distribution.

**Distribution Phases:**
- Phase 1: Seed Round (aligned with Testnet v1)
- Phase 2: Private Round (aligned with Testnet v2)
- Phase 3: Public Round (aligned with Testnet v3)
- Phase 4: Final Round (aligned with Mainnet launch)

**Vesting Schedule:**
| Component | Details |
|-----------|---------|
| TGE Release | 30% (4,500,000,000 PYRAX) |
| Cliff Period | 1 month |
| Vesting Duration | 12 months (linear) |
| Total Duration | 13 months from TGE |

**Release Timeline:**
```
TGE (Month 0):    ██████                  30% released (4.5B PYRAX)
Month 1:          ██████                  Cliff period (30% total)
Month 2:          ███████                 35.83% total
Month 3:          ████████                41.67% total
Month 4:          █████████               47.50% total
Month 5:          ██████████              53.33% total
Month 6:          ███████████             59.17% total
Month 7:          ████████████            65.00% total
Month 8:          █████████████           70.83% total
Month 9:          ██████████████          76.67% total
Month 10:         ███████████████         82.50% total
Month 11:         ████████████████        88.33% total
Month 12:         █████████████████       94.17% total
Month 13:         █████████████████████   100% fully vested
```

---

### 2. BDAG Community (10% - 10,000,000,000 PYRAX)

**Purpose:** BlockDAG community migration program for existing holders.

**Eligibility:** Verified BlockDAG token holders receive 31.25% of their original investment value in PYRAX tokens.

**Vesting Schedule:**
| Component | Details |
|-----------|---------|
| TGE Release | 0% |
| Cliff Period | 12 months |
| Vesting Duration | 12 months (linear) |
| Total Duration | 24 months from TGE |

**Release Timeline:**
```
Month 0-12:  Cliff period (0% released)
Month 13:    8.33% released
Month 14:    16.67% total
Month 15:    25.00% total
...
Month 24:    100% fully vested
```

---

### 3. Mining Rewards (35% - 35,000,000,000 PYRAX)

**Purpose:** Incentivize network security through proof-of-work mining.

**Distribution:**
| Stream | Allocation | Percentage | Algorithm |
|--------|------------|------------|-----------|
| Stream A (ASIC) | 21,000,000,000 | 60% | BLAKE3 |
| Stream B (GPU/CPU) | 14,000,000,000 | 40% | KAWPOW |

**Emission Schedule:**
- **Initial Block Reward:** ~1,666 PYRAX per block
- **Block Time:** 6 seconds average
- **Halving Interval:** ~21,000,000 blocks (~4 years)

**Halving Schedule:**
| Epoch | Block Range | Reward/Block | Cumulative |
|-------|-------------|--------------|------------|
| 1 | 0 - 21M | 1,666 PYRAX | ~50% |
| 2 | 21M - 42M | 833 PYRAX | ~75% |
| 3 | 42M - 63M | 416 PYRAX | ~87.5% |
| 4 | 63M - 84M | 208 PYRAX | ~93.75% |
| 5+ | 84M+ | <104 PYRAX | →100% |

**Estimated Mining Timeline:**
- 50% mined: ~4 years
- 75% mined: ~8 years
- 90% mined: ~16 years
- 99% mined: ~40 years

---

### 4. ZK Prover Rewards (5% - 5,000,000,000 PYRAX)

**Purpose:** Incentivize zero-knowledge proof generation and verification (Stream C).

**Emission:**
- Released per attestation/proof verification
- Base reward: ~100 PYRAX per attestation
- Scaled by proof complexity (1-10x multiplier)

**Burn Mechanism:**
| Component | Rate |
|-----------|------|
| Burn Rate | 10% of each reward |
| Net to Prover | 90% of reward |
| Burned | 10% (permanently removed) |

**Deflationary Impact:**
- Maximum potential burn: 500,000,000 PYRAX (10% of pool)
- Creates long-term deflationary pressure
- Reduces circulating supply over time

---

### 5. Team & Founders (4% - 4,000,000,000 PYRAX)

**Purpose:** Core team compensation for development and operations.

**Vesting Schedule:**
| Component | Details |
|-----------|---------|
| TGE Release | 0% |
| Cliff Period | 12 months |
| Vesting Duration | 48 months (linear) |
| Total Duration | 60 months (5 years) from TGE |

**Monthly Release After Cliff:** 83,333,333 PYRAX (~2.08% per month)

---

### 6. Advisors (3% - 3,000,000,000 PYRAX)

**Purpose:** Strategic, technical, and legal advisors providing guidance.

**Vesting Schedule:**
| Component | Details |
|-----------|---------|
| TGE Release | 0% |
| Cliff Period | 12 months |
| Vesting Duration | 48 months (linear) |
| Total Duration | 60 months (5 years) from TGE |

**Monthly Release After Cliff:** 62,500,000 PYRAX (~2.08% per month)

---

### 7. Ecosystem (10% - 10,000,000,000 PYRAX)

**Purpose:** Developer grants, strategic partnerships, ecosystem incentives, and bug bounties.

**Release Mechanism:**
- Milestone-based distribution
- Requires DAO approval for releases
- Transparent proposal and voting process

**Use Cases:**
| Category | Purpose |
|----------|---------|
| Developer Grants | dApp development, tooling, infrastructure |
| Partnerships | Strategic integrations, cross-chain bridges |
| Bug Bounties | Security researchers, vulnerability disclosure |
| Hackathons | Community development events |
| Education | Tutorials, documentation, courses |

---

### 8. Marketing (5% - 5,000,000,000 PYRAX)

**Purpose:** Digital marketing, community rewards, events, influencers, and airdrops.

**Release Mechanism:**
- As-needed basis
- Multi-signature approval required
- Quarterly budget allocation

**Use Cases:**
| Category | Allocation |
|----------|------------|
| Digital Marketing | 30% |
| Community Rewards | 25% |
| Events & Conferences | 20% |
| Influencer Partnerships | 15% |
| Airdrops | 10% |

---

### 9. Liquidity (10% - 10,000,000,000 PYRAX)

**Purpose:** Exchange listings and market liquidity.

**Release:** 100% at TGE

**Distribution:**
| Destination | Allocation | Amount |
|-------------|------------|--------|
| CEX Listings | 60% | 6,000,000,000 PYRAX |
| DEX Pools | 30% | 3,000,000,000 PYRAX |
| Market Making Reserve | 10% | 1,000,000,000 PYRAX |

---

### 10. Treasury (2% - 2,000,000,000 PYRAX)

**Purpose:** Emergency fund and long-term protocol sustainability.

**Governance:**
- DAO controlled
- 75% approval threshold required for any withdrawal
- Multi-signature wallet security

**Use Cases:**
- Protocol upgrades
- Emergency security responses
- Long-term sustainability initiatives

---

### 11. Reserve (1% - 1,000,000,000 PYRAX)

**Purpose:** Protocol buffer for unforeseen needs.

**Governance:**
- DAO governance controlled
- Reserved for critical protocol needs
- Acts as final safety net

---

## Vesting Summary

### Vesting Comparison Table

| Pool | TGE % | Cliff | Vesting | Total Duration |
|------|-------|-------|---------|----------------|
| Presale | 30% | 1 month | 12 months | 13 months |
| BDAG Community | 0% | 12 months | 12 months | 24 months |
| Team & Founders | 0% | 12 months | 48 months | 60 months |
| Advisors | 0% | 12 months | 48 months | 60 months |
| Liquidity | 100% | - | - | Immediate |

### Circulating Supply Projection

| Milestone | Circulating Supply | % of Total |
|-----------|-------------------|------------|
| TGE | 14,500,000,000 | 14.5% |
| Month 3 | 13,000,000,000 | 13% |
| Month 6 | 17,000,000,000 | 17% |
| Month 12 | 22,000,000,000 | 22% |
| Month 24 | 30,000,000,000 | 30% |
| Month 36 | 38,000,000,000 | 38% |
| Month 48 | 45,000,000,000 | 45% |
| Month 60 | 52,000,000,000 | 52% |

*Note: Excludes mining rewards which are released per block*

---

## Token Utility

### Primary Use Cases

| Use Case | Description |
|----------|-------------|
| **Transaction Fees** | Pay for on-chain transactions and smart contract execution |
| **Staking** | Stake tokens to become a validator and earn rewards |
| **Governance** | Vote on protocol upgrades and DAO proposals |
| **AI Compute** | Pay for AI/ML inference jobs on the network |
| **ZK Proofs** | Stake to become a ZK prover and earn attestation rewards |
| **Bridge Collateral** | Lock tokens for L1↔L2 bridge operations |

### Fee Distribution

| Recipient | Share | Description |
|-----------|-------|-------------|
| Stream A Miners | 20% | ASIC miners (BLAKE3) |
| Stream B Miners | 40% | GPU/CPU miners (KAWPOW) |
| Stream C Provers | 30% | ZK proof verifiers |
| Protocol Treasury | 10% | DAO-controlled fund |

---

## Deflationary Mechanisms

### 1. ZK Prover Burn
- 10% of all ZK prover rewards are burned
- Maximum potential burn: 500M PYRAX

### 2. Transaction Fee Burn (Future)
- Proposed: Burn portion of transaction fees
- Requires governance approval

### 3. Buyback & Burn (Future)
- Protocol may use treasury funds for buybacks
- Requires 75% DAO approval

---

## Governance

### Token-Weighted Voting
- 1 PYRAX = 1 vote
- Staked tokens receive 1.5x voting power
- Delegation supported

### Proposal Thresholds
| Action | Required Approval |
|--------|-------------------|
| Standard Proposals | 51% |
| Parameter Changes | 51% |
| Treasury Withdrawals | 75% |
| Protocol Upgrades | 67% |
| Emergency Actions | 67% + Multi-sig |

---

## Security

### Smart Contract Audits
- All token contracts audited by reputable firms
- Continuous security monitoring
- Bug bounty program active

### Multi-Signature Wallets
| Wallet | Signers Required |
|--------|------------------|
| Treasury | 4 of 7 |
| Marketing | 3 of 5 |
| Team Vesting | 3 of 5 |

---

## Legal Disclaimer

This document is for informational purposes only and does not constitute financial advice, an offer to sell, or a solicitation of an offer to buy any tokens. PYRAX tokens are utility tokens and should not be considered securities. Token holders should conduct their own research and consult with qualified professionals before making any investment decisions.

---

## Contact

- **Website:** https://pyrax.org
- **Documentation:** https://docs.pyrax.org
- **GitHub:** https://github.com/pyrax-official
- **Twitter:** @pyrax_official
- **Discord:** discord.gg/pyrax

---

*Document Version 1.0 - January 2026*
