# PYRAX Protocol Specification

> **Version:** 1.0.0-draft  
> **Status:** FROZEN FOR PHASE 0  
> **Last Updated:** 2026-01-02

---

## 1. Overview

PYRAX is a Proof-of-Work blockchain implementing the KAWPOW mining algorithm with integrated AI/ML compute marketplace functionality.

### 1.1 Design Goals

- ASIC-resistant mining via KAWPOW
- Fast block times (~60 seconds)
- Native support for AI job execution and verification
- EVM-compatible address format
- Strong replay protection via chain IDs

---

## 2. Cryptographic Primitives

### 2.1 Hash Functions

| Purpose | Algorithm | Output Size |
|---------|-----------|-------------|
| Address derivation | Keccak-256 | 32 bytes |
| Block header hash | KAWPOW | 32 bytes |
| Transaction hash | Keccak-256 | 32 bytes |
| Merkle tree | Keccak-256 | 32 bytes |
| State trie | Keccak-256 | 32 bytes |

### 2.2 Signature Scheme

- **Algorithm:** ECDSA over secp256k1
- **Signature format:** (r, s, v) where:
  - r: 32 bytes
  - s: 32 bytes  
  - v: 1 byte (recovery ID: 0 or 1)

### 2.3 Key Derivation

- **HD Wallets:** BIP-32/BIP-39/BIP-44 compatible
- **Derivation path:** `m/44'/60'/account'/change/index`
- **Mnemonic:** 12-24 word BIP-39 English wordlist

---

## 3. Block Structure

### 3.1 Block Header (112 bytes)

```
+----------+--------+------------------------------------------+
| Field    | Size   | Description                              |
+----------+--------+------------------------------------------+
| version  | 4      | Protocol version (uint32 LE)             |
| parent   | 32     | Parent block hash                        |
| merkle   | 32     | Merkle root of transactions              |
| state    | 32     | State trie root                          |
| time     | 8      | Unix timestamp in seconds (uint64 LE)    |
| diff     | 8      | Difficulty target (uint64 LE)            |
| nonce    | 8      | PoW nonce (uint64 LE)                    |
| height   | 8      | Block number (uint64 LE)                 |
| extra    | 8      | Extra nonce for miners (uint64 LE)       |
| miner    | 20     | Beneficiary address                      |
+----------+--------+------------------------------------------+
Total: 160 bytes
```

### 3.2 Block Validation Rules

1. **Parent exists** (except genesis)
2. **Height = parent.height + 1**
3. **Timestamp > parent.timestamp**
4. **Timestamp <= now + 15 seconds** (future tolerance)
5. **Difficulty matches adjustment algorithm**
6. **Merkle root matches transaction list**
7. **PoW hash < target**
8. **All transactions valid**

### 3.3 Genesis Block

```json
{
  "version": 1,
  "parent_hash": "0x0000000000000000000000000000000000000000000000000000000000000000",
  "merkle_root": "0x0000000000000000000000000000000000000000000000000000000000000000",
  "state_root": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
  "timestamp": 1704067200,
  "difficulty": 1000000,
  "nonce": 0,
  "height": 0,
  "extra_nonce": 0,
  "beneficiary": "0x0000000000000000000000000000000000000000"
}
```

**Genesis Hash:** `0x[TO BE COMPUTED]`

---

## 4. Transaction Structure

### 4.1 Transaction Format

```
+------------+----------+------------------------------------------+
| Field      | Size     | Description                              |
+------------+----------+------------------------------------------+
| version    | 2        | TX format version (uint16 LE)            |
| type       | 1        | Transaction type (uint8)                 |
| chain_id   | 4        | Network identifier (uint32 LE)           |
| nonce      | 8        | Sender's TX sequence (uint64 LE)         |
| gas_price  | 8        | Fee per gas unit (uint64 LE)             |
| gas_limit  | 8        | Maximum gas units (uint64 LE)            |
| to_flag    | 1        | 1 if 'to' present, 0 for deploy          |
| to         | 0 or 20  | Recipient address (if to_flag=1)         |
| value      | 32       | Amount in wei (uint256 LE)               |
| data_len   | 4        | Data length (uint32 LE)                  |
| data       | variable | Payload bytes                            |
| r          | 32       | Signature r component                    |
| s          | 32       | Signature s component                    |
| v          | 1        | Signature recovery ID                    |
+------------+----------+------------------------------------------+
```

### 4.2 Transaction Types

| Type | Value | Description |
|------|-------|-------------|
| Transfer | 0 | Simple value transfer |
| ContractDeploy | 1 | Deploy new contract |
| ContractCall | 2 | Call existing contract |
| AiJobSubmit | 3 | Submit AI job to marketplace |
| AiJobClaim | 4 | Worker claims AI job |
| AiJobComplete | 5 | Worker submits result |
| ModelRegister | 6 | Register AI model |
| Stake | 7 | Stake tokens |
| Unstake | 8 | Unstake tokens |

### 4.3 Gas Costs

| Operation | Gas |
|-----------|-----|
| Base transfer | 21,000 |
| Contract deploy (base) | 32,000 |
| AI job submit | 50,000 |
| AI job claim | 30,000 |
| AI job complete | 40,000 |
| Model register | 75,000 |
| Stake/Unstake | 25,000 |
| Zero byte in data | 4 |
| Non-zero byte in data | 16 |
| Storage write (new) | 20,000 |
| Storage write (existing) | 5,000 |

### 4.4 Transaction Signing

1. Encode transaction fields (excluding signature)
2. Append chain_id for replay protection
3. Compute Keccak-256 hash
4. Sign with ECDSA secp256k1
5. Append (r, s, v) to transaction

---

## 5. KAWPOW Mining Algorithm

### 5.1 Parameters

| Parameter | Value |
|-----------|-------|
| EPOCH_LENGTH | 7,500 blocks |
| CACHE_BYTES_INIT | 16 MB |
| CACHE_BYTES_GROWTH | 128 KB/epoch |
| DAG_BYTES_INIT | 1 GB |
| DAG_BYTES_GROWTH | 8 MB/epoch |
| PROGPOW_LANES | 16 |
| PROGPOW_REGS | 32 |
| PROGPOW_CNT_DAG | 64 |
| PROGPOW_CNT_MATH | 18 |

### 5.2 Algorithm Flow

```
1. epoch = height / EPOCH_LENGTH
2. seed = Keccak256^epoch(zeros)
3. cache = generate_cache(seed, cache_size(epoch))
4. dag = generate_dag(cache, dag_size(epoch))
5. mix_seed = Keccak512(header_hash || nonce)
6. For each lane (0..15):
   a. Initialize mix from seed
   b. For 64 rounds:
      - DAG lookups
      - Math operations
      - Lane mixing
7. final_hash = Keccak256(mix_seed || mix_digest)
8. Valid if final_hash < target
```

### 5.3 Test Vectors

```
Header Hash: 0xabcdef...
Nonce: 12345678
Height: 7500 (epoch 1)
Expected Mix: 0x...
Expected Hash: 0x...
```

---

## 6. Difficulty Adjustment

### 6.1 Parameters

| Parameter | Value |
|-----------|-------|
| TARGET_BLOCK_TIME | 60 seconds |
| ADJUSTMENT_WINDOW | 720 blocks |
| MAX_ADJUSTMENT | ±25% |
| MIN_DIFFICULTY | 1 |

### 6.2 Algorithm

```python
def calculate_difficulty(parent, grandparent_time):
    if parent.height < 2:
        return INITIAL_DIFFICULTY
    
    actual_time = parent.timestamp - grandparent_time
    expected_time = TARGET_BLOCK_TIME
    
    ratio = expected_time / actual_time
    ratio = clamp(ratio, 0.75, 1.25)
    
    new_diff = parent.difficulty * ratio
    return max(new_diff, MIN_DIFFICULTY)
```

---

## 7. Network Protocol

### 7.1 Chain IDs

| Network | Chain ID |
|---------|----------|
| Mainnet | 1 |
| Testnet | 2 |
| Devnet | 3 |

### 7.2 Default Ports

| Service | Port |
|---------|------|
| P2P | 30303 |
| RPC | 8545 |
| WebSocket | 8546 |

### 7.3 P2P Message Types

| Type | ID | Description |
|------|-----|-------------|
| Hello | 0x00 | Handshake initiation |
| HelloAck | 0x01 | Handshake response |
| Disconnect | 0x02 | Graceful disconnect |
| GetHeaders | 0x10 | Request block headers |
| Headers | 0x11 | Block headers response |
| GetBlocks | 0x12 | Request full blocks |
| Block | 0x13 | Full block data |
| NewBlock | 0x14 | Announce new block |
| InvTx | 0x20 | Transaction inventory |
| GetTx | 0x21 | Request transactions |
| Tx | 0x22 | Transaction data |
| Ping | 0x30 | Keepalive |
| Pong | 0x31 | Keepalive response |

---

## 8. Economic Model

### 8.1 Token Parameters

| Parameter | Value |
|-----------|-------|
| Total Supply | 21,000,000,000 PYRAX |
| Decimals | 18 |
| Initial Block Reward | 5,000 PYRAX |
| Halving Interval | 2,100,000 blocks |

### 8.2 Distribution

| Allocation | Percentage | Amount |
|------------|------------|--------|
| Mining | 70% | 14.7B |
| AI Compute | 15% | 3.15B |
| Development | 8% | 1.68B |
| Ecosystem | 5% | 1.05B |
| Genesis | 2% | 420M |

### 8.3 Fee Model

- Fee = gas_used × gas_price
- 80% to block miner
- 20% burned

---

## 9. Address Format

### 9.1 Generation

1. Generate secp256k1 keypair
2. Take uncompressed public key (64 bytes, without 0x04 prefix)
3. Keccak-256 hash
4. Take last 20 bytes
5. Encode as `0x` + 40 hex characters

### 9.2 Checksum (EIP-55)

1. Lowercase hex address (without 0x)
2. Keccak-256 hash of lowercase
3. For each character:
   - If hex hash nibble ≥ 8: uppercase
   - Otherwise: lowercase
4. Prepend `0x`

---

## 10. State Model

### 10.1 Account State

```
Account {
    nonce: u64,
    balance: U256,
    code_hash: Option<H256>,
    storage_root: Option<H256>
}
```

### 10.2 State Transitions

- **Transfer:** Decrease sender balance, increase receiver balance
- **Contract deploy:** Create account with code
- **Contract call:** Execute code, modify storage
- **AI job:** Escrow funds, release on completion

---

## Appendix A: Test Vectors

### A.1 Address Generation

```
Private Key: 0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
Public Key (uncompressed): 0x04...
Keccak256: 0x...
Address: 0x...
Checksum Address: 0x...
```

### A.2 Transaction Signing

```
Transaction:
  nonce: 0
  gas_price: 1000000000
  gas_limit: 21000
  to: 0x...
  value: 1000000000000000000
  data: 0x
  chain_id: 1

Signing Hash: 0x...
Signature (r,s,v): ...
TX Hash: 0x...
```

### A.3 Block Hashing

```
Block Header:
  version: 1
  parent: 0x...
  merkle: 0x...
  ...

Header Hash: 0x...
```

---

*This specification is frozen for Phase 0. Changes require version increment and migration plan.*
