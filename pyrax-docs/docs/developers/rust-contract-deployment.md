# Rust Contract Deployment

This guide covers deploying WASM/Rust smart contracts to PYRAX.

## Build for Deployment

### Optimized Build

```bash
# Build release version
cargo build --release --target wasm32-unknown-unknown

# Output location
ls target/wasm32-unknown-unknown/release/*.wasm
```

### Optimize WASM

```bash
# Install wasm-opt (if not installed)
cargo install wasm-opt

# Optimize for size
wasm-opt -Oz \
  target/wasm32-unknown-unknown/release/my_contract.wasm \
  -o my_contract.wasm

# Check size
ls -lh my_contract.wasm
```

### Verify Contract

```bash
# Validate WASM structure
pyrax-cli contract check my_contract.wasm

# Output:
# ✓ Valid WASM module
# ✓ Required exports found
# ✓ Memory limits OK
# ✓ Estimated deployment cost: 0.5 PYRAX
```

## Deployment Methods

### Using PYRAX CLI

```bash
# Deploy to testnet
pyrax-cli contract deploy my_contract.wasm \
  --network testnet \
  --private-key $PRIVATE_KEY \
  --init-args '{"initial_value": 100}'

# Output:
# Deploying contract...
# Transaction: 0xabc123...
# Contract address: 0xdef456...
# Gas used: 150000
```

### Using JavaScript SDK

```javascript
import { PyraxSDK } from '@pyrax/sdk';
import fs from 'fs';

const pyrax = new PyraxSDK({
  network: 'testnet',
  privateKey: process.env.PRIVATE_KEY
});

// Read WASM file
const wasmCode = fs.readFileSync('./my_contract.wasm');

// Deploy
const result = await pyrax.wasm.deploy({
  code: wasmCode,
  initArgs: { initial_value: 100 },
  gasLimit: 500000
});

console.log('Contract deployed at:', result.contractAddress);
```

### Using Python SDK

```python
from pyrax import PyraxClient

client = PyraxClient(
    network='testnet',
    private_key=os.environ['PRIVATE_KEY']
)

# Read WASM file
with open('my_contract.wasm', 'rb') as f:
    wasm_code = f.read()

# Deploy
result = client.wasm.deploy(
    code=wasm_code,
    init_args={'initial_value': 100},
    gas_limit=500000
)

print(f'Contract deployed at: {result.contract_address}')
```

## Deployment Script

Create `scripts/deploy.sh`:

```bash
#!/bin/bash
set -e

# Configuration
NETWORK="${NETWORK:-testnet}"
CONTRACT_NAME="my_contract"

echo "Building contract..."
cargo build --release --target wasm32-unknown-unknown

echo "Optimizing WASM..."
wasm-opt -Oz \
  target/wasm32-unknown-unknown/release/${CONTRACT_NAME}.wasm \
  -o ${CONTRACT_NAME}.wasm

echo "Deploying to ${NETWORK}..."
pyrax-cli contract deploy ${CONTRACT_NAME}.wasm \
  --network $NETWORK \
  --private-key $PRIVATE_KEY \
  --init-args '{}' \
  --output json > deployment.json

echo "Deployment complete!"
cat deployment.json
```

## Interacting with Deployed Contract

### Query (Read-Only)

```bash
# CLI
pyrax-cli contract query 0xCONTRACT_ADDRESS \
  --method "get_value" \
  --network testnet

# Output: 100
```

### Execute (State-Changing)

```bash
# CLI
pyrax-cli contract call 0xCONTRACT_ADDRESS \
  --method "set_value" \
  --args '{"new_value": 200}' \
  --network testnet \
  --private-key $PRIVATE_KEY
```

### JavaScript

```javascript
// Load deployed contract
const contract = pyrax.wasm.contract('0xCONTRACT_ADDRESS');

// Query
const value = await contract.query('get_value');
console.log('Current value:', value);

// Execute
const tx = await contract.call('set_value', { new_value: 200 });
await tx.wait();
console.log('Value updated!');
```

## Upgradeable Contracts

### Proxy Pattern

```rust
#[pyrax_contract]
pub struct Proxy {
    implementation: Address,
    admin: Address,
}

#[pyrax_contract]
impl Proxy {
    #[init]
    pub fn new(implementation: Address) -> Self {
        Self {
            implementation,
            admin: env::caller(),
        }
    }

    pub fn upgrade(&mut self, new_implementation: Address) {
        require!(env::caller() == self.admin, "Not admin");
        self.implementation = new_implementation;
    }

    #[fallback]
    pub fn delegate(&self) {
        // Forward all calls to implementation
        env::delegate_call(self.implementation);
    }
}
```

### Deploy Upgradeable

```bash
# Deploy implementation
pyrax-cli contract deploy my_contract_v1.wasm \
  --network testnet \
  --private-key $PRIVATE_KEY
# Returns: 0xIMPLEMENTATION

# Deploy proxy
pyrax-cli contract deploy proxy.wasm \
  --network testnet \
  --private-key $PRIVATE_KEY \
  --init-args '{"implementation": "0xIMPLEMENTATION"}'
# Returns: 0xPROXY (use this address)
```

## Gas Estimation

```bash
# Estimate deployment cost
pyrax-cli contract estimate my_contract.wasm \
  --network testnet \
  --init-args '{}'

# Output:
# Estimated gas: 250000
# Gas price: 20 gwei
# Total cost: ~0.005 PYRAX
```

## Verification

### Verify Source Code

```bash
# Upload source for verification
pyrax-cli contract verify 0xCONTRACT_ADDRESS \
  --source ./src \
  --network testnet
```

### On Block Explorer

1. Go to [explorer.pyrax.org](https://explorer.pyrax.org)
2. Search for contract address
3. Click "Verify Contract"
4. Upload source files
5. Select Rust version and settings

## Deployment Checklist

- [ ] All tests passing
- [ ] WASM optimized for size
- [ ] Contract verified with `pyrax-cli check`
- [ ] Sufficient PYRAX for gas
- [ ] Init arguments prepared
- [ ] Testnet deployment verified
- [ ] Source code ready for verification

## Common Issues

### "Gas limit exceeded"
Increase gas limit or optimize contract size.

### "Invalid init args"
Check JSON format and required fields.

### "WASM too large"
Run wasm-opt with -Oz flag.

### "Insufficient funds"
Ensure wallet has enough PYRAX for deployment.

---

:::info Next Steps
- [Best Practices](./wasm-best-practices) — Optimize your contracts
- [Contract Security](./contract-security) — Security guidelines
:::
