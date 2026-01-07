# PYRAX Genesis Block Documentation

> **Network:** Mainnet  
> **Status:** DRAFT - NOT FINALIZED

---

## Genesis Block Specification

### Header Fields

| Field | Value |
|-------|-------|
| Version | 1 |
| Parent Hash | `0x0000000000000000000000000000000000000000000000000000000000000000` |
| Merkle Root | `0x0000000000000000000000000000000000000000000000000000000000000000` |
| State Root | `0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421` |
| Timestamp | 1704067200 (2024-01-01 00:00:00 UTC) |
| Difficulty | 1,000,000 |
| Nonce | 0 |
| Height | 0 |
| Extra Nonce | 0 |
| Beneficiary | `0x0000000000000000000000000000000000000000` |

### Genesis Hash

```
MAINNET:  0x[TO BE COMPUTED AT LAUNCH]
TESTNET:  0x[TO BE COMPUTED AT LAUNCH]
DEVNET:   0x[TO BE COMPUTED]
```

---

## Initial State

### Genesis Allocations

| Address | Amount (PYRAX) | Purpose |
|---------|----------------|---------|
| `0x...` | 420,000,000 | Genesis allocation (2%) |

### Allocation Breakdown

```
Total Genesis Allocation: 420,000,000 PYRAX (2% of total supply)

- Development Fund:    168,000,000 PYRAX (locked, vesting schedule)
- Ecosystem Fund:      126,000,000 PYRAX (locked, community governed)
- Initial Liquidity:    84,000,000 PYRAX (DEX bootstrapping)
- Bug Bounty Reserve:   42,000,000 PYRAX (security fund)
```

---

## Verification Steps

### Step 1: Build from Source

```bash
git clone https://github.com/pyrax-official/pyrax.git
cd pyrax
git checkout v1.0.0
cargo build --release
```

### Step 2: Verify Binary Hash

```bash
# Windows
certutil -hashfile target/release/pyrax-node.exe SHA256

# Expected hash (release v1.0.0):
# [TO BE PUBLISHED]
```

### Step 3: Verify Genesis Hash

```bash
./pyrax-node --verify-genesis
```

Expected output:
```
Genesis block verification:
  Chain ID: 1 (mainnet)
  Genesis Hash: 0x...
  State Root: 0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421
  Initial Difficulty: 1000000
  Timestamp: 2024-01-01T00:00:00Z
  
Status: VALID ✓
```

### Step 4: Independent Calculation

You can independently verify the genesis hash:

```python
import hashlib
from eth_utils import keccak

def calculate_genesis_hash():
    header = b''
    header += (1).to_bytes(4, 'little')  # version
    header += bytes(32)  # parent_hash (zeros)
    header += bytes(32)  # merkle_root (zeros)
    header += bytes.fromhex('56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421')  # state_root
    header += (1704067200).to_bytes(8, 'little')  # timestamp
    header += (1000000).to_bytes(8, 'little')  # difficulty
    header += (0).to_bytes(8, 'little')  # nonce
    header += (0).to_bytes(8, 'little')  # height
    header += (0).to_bytes(8, 'little')  # extra_nonce
    header += bytes(20)  # beneficiary (zeros)
    
    return keccak(header).hex()

print(f"Genesis Hash: 0x{calculate_genesis_hash()}")
```

---

## Network Configuration

### Mainnet

```toml
[network]
chain_id = 1
genesis_hash = "0x..."

[bootnodes]
nodes = [
    "/ip4/seed1.pyrax.org/tcp/30303/p2p/...",
    "/ip4/seed2.pyrax.org/tcp/30303/p2p/...",
    "/ip4/seed3.pyrax.org/tcp/30303/p2p/..."
]
```

### Testnet

```toml
[network]
chain_id = 2
genesis_hash = "0x..."

[bootnodes]
nodes = [
    "/ip4/testnet-seed1.pyrax.org/tcp/30303/p2p/...",
    "/ip4/testnet-seed2.pyrax.org/tcp/30303/p2p/..."
]
```

### Devnet (Local)

```toml
[network]
chain_id = 3
genesis_hash = "0x..."

[bootnodes]
nodes = []  # Local discovery only
```

---

## Genesis File (genesis.json)

```json
{
  "config": {
    "chainId": 1,
    "kawpowBlock": 0,
    "targetBlockTime": 60,
    "difficultyAdjustmentWindow": 720,
    "maxDifficultyChange": 0.25,
    "initialDifficulty": 1000000,
    "blockReward": "5000000000000000000000",
    "halvingInterval": 2100000
  },
  "timestamp": "0x65900a00",
  "extraData": "0x5059524158204d41494e4e4554",
  "gasLimit": "0x1c9c380",
  "difficulty": "0xf4240",
  "alloc": {
    "0x0000000000000000000000000000000000000001": {
      "balance": "420000000000000000000000000000"
    }
  }
}
```

---

## Historical Record

| Event | Date | Details |
|-------|------|---------|
| Specification Finalized | TBD | Genesis parameters locked |
| Genesis File Published | TBD | Official genesis.json released |
| Testnet Launch | TBD | Testnet genesis created |
| Mainnet Launch | TBD | Mainnet goes live |

---

## Checkpoints

Checkpoints are periodically published to prevent long-range attacks:

| Height | Hash | Date |
|--------|------|------|
| 0 | `0x...` | Genesis |
| TBD | TBD | TBD |

---

## Security Considerations

1. **Never trust unofficial genesis files** - Always verify against this document
2. **Verify binary hashes** - Ensure you're running official releases
3. **Check chain ID** - Prevents replay attacks across networks
4. **Monitor for forks** - Nodes should alert on unexpected reorganizations

---

## Contact

For questions about genesis configuration:
- GitHub Issues: https://github.com/pyrax-official/pyrax/issues
- Discord: https://discord.gg/pyrax
- Email: dev@pyrax.org

---

*This document will be finalized before mainnet launch. All values marked "TBD" will be filled in during the launch process.*
