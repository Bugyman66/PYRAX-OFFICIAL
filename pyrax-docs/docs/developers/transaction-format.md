# Transaction Format

This document describes the structure and types of transactions on PYRAX.

## Transaction Types

PYRAX supports three transaction types:

| Type | ID | Description |
|------|-----|-------------|
| Legacy | 0 | Original Ethereum format |
| Access List | 1 | EIP-2930 with access lists |
| Dynamic Fee | 2 | EIP-1559 with priority fees |

## Type 2 (EIP-1559) Transaction

The recommended transaction type for PYRAX.

### Fields

```javascript
{
  type: 2,
  chainId: 1000,              // PYRAX chain ID
  nonce: 5,                   // Sender's tx count
  maxPriorityFeePerGas: 1e9,  // Tip to miner (wei)
  maxFeePerGas: 50e9,         // Max total fee (wei)
  gasLimit: 21000,            // Max gas units
  to: "0x...",                // Recipient address
  value: 1e18,                // Amount in wei
  data: "0x...",              // Call data
  accessList: []              // Optional access list
}
```

### Signature Fields

```javascript
{
  v: 0 or 1,     // Recovery ID
  r: "0x...",    // ECDSA r
  s: "0x..."     // ECDSA s
}
```

## Legacy Transaction

For backward compatibility.

### Fields

```javascript
{
  nonce: 5,
  gasPrice: 20e9,    // Fixed gas price
  gasLimit: 21000,
  to: "0x...",
  value: 1e18,
  data: "0x...",
  v: 27 or 28,       // Chain-encoded
  r: "0x...",
  s: "0x..."
}
```

## Transaction Lifecycle

```
┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐
│ Create  │───►│  Sign   │───►│  Send   │───►│ Pending │
└─────────┘    └─────────┘    └─────────┘    └────┬────┘
                                                  │
                                                  ▼
┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐
│Confirmed│◄───│ Mined   │◄───│Selected │◄───│  Pool   │
└─────────┘    └─────────┘    └─────────┘    └─────────┘
```

## Gas and Fees

### Gas Costs

| Operation | Gas Cost |
|-----------|----------|
| Transfer (ETH) | 21,000 |
| Transfer (ERC-20) | ~65,000 |
| Contract creation | 32,000 + code cost |
| Storage write (new) | 20,000 |
| Storage write (update) | 5,000 |
| Storage read | 2,100 |

### Fee Calculation

**EIP-1559:**
```
Total Fee = gasUsed * (baseFee + priorityFee)
```

**Legacy:**
```
Total Fee = gasUsed * gasPrice
```

### Estimating Gas

```javascript
const gasEstimate = await provider.estimateGas({
  to: "0x...",
  data: "0x..."
});
```

## Creating Transactions

### Using ethers.js

```javascript
const { ethers } = require('ethers');

// Connect
const provider = new ethers.JsonRpcProvider('https://rpc.pyrax.org');
const wallet = new ethers.Wallet(privateKey, provider);

// Create transaction
const tx = {
  to: "0xRecipientAddress",
  value: ethers.parseEther("1.0"),
  // Gas settings auto-determined
};

// Send
const response = await wallet.sendTransaction(tx);

// Wait for confirmation
const receipt = await response.wait();
console.log('Confirmed in block:', receipt.blockNumber);
```

### Contract Interaction

```javascript
// Load contract
const contract = new ethers.Contract(address, abi, wallet);

// Call (read-only)
const result = await contract.balanceOf(address);

// Send transaction (state-changing)
const tx = await contract.transfer(recipient, amount);
const receipt = await tx.wait();
```

## Transaction Hash

The transaction hash uniquely identifies a transaction:

```javascript
txHash = keccak256(RLP(signedTransaction))
```

## Nonce Management

### Getting Current Nonce

```javascript
const nonce = await provider.getTransactionCount(address);
```

### Nonce Gaps

- Transactions must be sequential
- Gap in nonces blocks subsequent transactions
- Use pending nonce for queued transactions

```javascript
const pendingNonce = await provider.getTransactionCount(address, 'pending');
```

## Transaction Replacement

Replace a pending transaction:

1. Same nonce
2. Higher gas price (>10% increase)
3. Re-sign and submit

```javascript
const replacement = {
  ...originalTx,
  nonce: originalNonce,
  maxFeePerGas: originalFee * 1.1,
  maxPriorityFeePerGas: originalPriority * 1.1
};
```

## Error Handling

### Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `nonce too low` | Nonce already used | Use correct nonce |
| `insufficient funds` | Not enough balance | Add funds |
| `gas too low` | Gas limit too small | Increase gas limit |
| `replacement underpriced` | Replacement fee too low | Increase by >10% |
| `execution reverted` | Contract error | Check contract logic |

### Handling Reverts

```javascript
try {
  const tx = await contract.riskyFunction();
  await tx.wait();
} catch (error) {
  if (error.code === 'CALL_EXCEPTION') {
    console.log('Revert reason:', error.reason);
  }
}
```

## Encoding

### RLP Encoding

Transactions use RLP (Recursive Length Prefix):

```
Type 2: 0x02 || RLP([chainId, nonce, maxPriorityFeePerGas, maxFeePerGas, gasLimit, to, value, data, accessList, v, r, s])
```

### Signing

```javascript
// Sign transaction
const signedTx = await wallet.signTransaction(tx);

// Broadcast
const response = await provider.broadcastTransaction(signedTx);
```

## Querying Transactions

### Get Transaction

```javascript
const tx = await provider.getTransaction(txHash);
```

### Get Receipt

```javascript
const receipt = await provider.getTransactionReceipt(txHash);
```

### Receipt Fields

```javascript
{
  status: 1,                    // 1 = success, 0 = failure
  blockNumber: 12345,
  blockHash: "0x...",
  transactionIndex: 5,
  from: "0x...",
  to: "0x...",
  contractAddress: null,        // If contract creation
  gasUsed: 21000n,
  cumulativeGasUsed: 150000n,
  effectiveGasPrice: 20000000000n,
  logs: [...],
  logsBloom: "0x..."
}
```

---

:::info Related Topics
- [Block Structure](./block-structure)
- [RPC Endpoints](./rpc-endpoints)
- [JavaScript SDK](./javascript-sdk)
:::
