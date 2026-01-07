# PYRAX Genesis Block Specification

> **Status:** Canonical  
> **Last Updated:** 2026-01-02

## Overview

The genesis block is the first block in the PYRAX blockchain. Each network (mainnet, testnet, devnet) has a unique, deterministic genesis block with a fixed hash.

---

## Genesis Parameters

### Common Parameters

| Parameter | Value | Description |
|-----------|-------|-------------|
| Version | 1 | Block version |
| Stream | 0 | Stream A (BLAKE3 PoW) |
| Height | 0 | First block |
| Parent Hash | `0x0000...0000` | Zero hash (no parent) |
| Nonce | 0 | No mining required for genesis |
| Extra Nonce | 0 | Not used for genesis |
| Beneficiary | `0x0000...0000` | Zero address |
| Timestamp | 1735689600 | 2025-01-01 00:00:00 UTC |

### Network-Specific Parameters

| Network | Chain ID | Difficulty | Extra Data |
|---------|----------|------------|------------|
| **Mainnet** | 1 | 1,000,000 | "PYRAX Genesis - TriStream DAG Blockchain" |
| **Testnet** | 2 | 1,000 | "PYRAX Testnet Genesis" |
| **Devnet** | 3 | 1 | "PYRAX Devnet Genesis" |

---

## Genesis Block Structure

```json
{
  "header": {
    "version": 1,
    "stream": 0,
    "parent_hash": "0x0000000000000000000000000000000000000000000000000000000000000000",
    "merkle_root": "<coinbase_txid>",
    "utxo_commitment": "0x0000000000000000000000000000000000000000000000000000000000000000",
    "timestamp": 1735689600,
    "difficulty": <network_specific>,
    "nonce": 0,
    "extra_nonce": 0,
    "height": 0,
    "beneficiary": "0x0000000000000000000000000000000000000000"
  },
  "transactions": [
    {
      "version": 1,
      "inputs": [{
        "previous_output": { "txid": "0x00...00", "vout": 4294967295 },
        "script_sig": "<extra_data_bytes>"
      }],
      "outputs": [{
        "value": 0,
        "script_pubkey": ""
      }],
      "lock_time": 0
    }
  ]
}
```

---

## Genesis Hash Computation

The genesis block hash is computed deterministically using BLAKE3:

```rust
use blake3::Hasher;

fn compute_genesis_hash(header: &BlockHeader) -> H256 {
    let mut hasher = Hasher::new();
    hasher.update(&header.version.to_le_bytes());
    hasher.update(&[header.stream]);
    hasher.update(header.parent_hash.as_bytes());
    hasher.update(header.merkle_root.as_bytes());
    hasher.update(header.utxo_commitment.as_bytes());
    hasher.update(&header.timestamp.to_le_bytes());
    hasher.update(&header.difficulty.to_le_bytes());
    hasher.update(&header.nonce.to_le_bytes());
    hasher.update(&header.extra_nonce.to_le_bytes());
    hasher.update(&header.height.to_le_bytes());
    hasher.update(header.beneficiary.as_bytes());
    
    let result = hasher.finalize();
    H256::from_slice(result.as_bytes())
}
```

---

## Verification

### Verify Genesis Locally

```bash
# Build and run genesis verification
cargo run -p pyrax-node --bin pyrax-node -- --network devnet

# Output should show:
# Genesis block stored: 0x<HASH>
```

### Independent Verification

To independently verify the genesis block:

1. **Compute merkle root** from the coinbase transaction
2. **Build header** with all parameters above
3. **Hash with BLAKE3** using the header encoding
4. **Compare** with expected hash

### Validation Rules

A valid genesis block MUST:
- Have height = 0
- Have parent_hash = zero
- Have correct timestamp for network
- Have correct difficulty for network
- Have exactly one coinbase transaction
- Have merkle_root = txid of coinbase transaction
- Hash to expected value for network

---

## Genesis Files

Configuration files are located in `pyrax-node/genesis/`:

```
pyrax-node/genesis/
├── mainnet.json   # Mainnet genesis config
├── testnet.json   # Testnet genesis config
└── devnet.json    # Devnet genesis config
```

---

## Security Considerations

1. **Immutability**: Genesis hash is hardcoded; any change invalidates the chain
2. **Determinism**: Genesis block is computed identically on all nodes
3. **Validation**: Nodes reject chains with wrong genesis hash
4. **No Pre-mine**: Genesis coinbase has zero output value

---

## Implementation

Genesis block generation and validation is implemented in:

- `pyrax-node/src/genesis/mod.rs` - Core genesis module
- `pyrax-node/src/storage/chaindb.rs` - Genesis initialization and validation

### Key Functions

```rust
// Generate genesis block for a network
pub fn genesis_block(network: NetworkId) -> Block

// Validate a block against expected genesis
pub fn validate_genesis(block: &Block, network: NetworkId) -> Result<(), GenesisError>

// Get expected genesis hash for a network
pub fn expected_genesis_hash(network: NetworkId) -> H256
```
