# JavaScript SDK

The PYRAX JavaScript SDK provides easy integration with the PYRAX network.

## Installation

```bash
npm install @pyrax/sdk ethers
```

## Quick Start

```javascript
import { PyraxSDK } from '@pyrax/sdk';

// Initialize
const pyrax = new PyraxSDK({
  network: 'mainnet',
  // or: rpcUrl: 'https://rpc.pyrax.org'
});

// Get balance
const balance = await pyrax.getBalance('0x...');
console.log('Balance:', balance);
```

## Configuration

### Basic Setup

```javascript
import { PyraxSDK } from '@pyrax/sdk';

const pyrax = new PyraxSDK({
  network: 'mainnet',  // or 'testnet'
});
```

### With Custom RPC

```javascript
const pyrax = new PyraxSDK({
  rpcUrl: 'https://your-node.example.com',
  wsUrl: 'wss://your-node.example.com'
});
```

### With Wallet

```javascript
const pyrax = new PyraxSDK({
  network: 'mainnet',
  privateKey: process.env.PRIVATE_KEY
});

// Or connect wallet later
await pyrax.connect(privateKey);
```

## Core Functions

### Account Operations

```javascript
// Get balance
const balance = await pyrax.getBalance(address);

// Get transaction count (nonce)
const nonce = await pyrax.getTransactionCount(address);

// Get token balance
const tokenBalance = await pyrax.getTokenBalance(tokenAddress, walletAddress);
```

### Transactions

```javascript
// Send PYRAX
const tx = await pyrax.send({
  to: recipientAddress,
  value: '1.0'  // PYRAX
});

// Wait for confirmation
const receipt = await tx.wait();
console.log('Confirmed in block:', receipt.blockNumber);

// Send with options
const tx2 = await pyrax.send({
  to: recipient,
  value: '1.0',
  gasLimit: 21000,
  maxFeePerGas: '50',  // Gwei
});
```

### Contract Interaction

```javascript
// Load contract
const contract = pyrax.contract(address, abi);

// Read (no gas)
const value = await contract.getValue();

// Write (requires wallet)
const tx = await contract.setValue(100);
await tx.wait();

// With options
const tx2 = await contract.setValue(100, {
  gasLimit: 100000
});
```

## Staking

```javascript
// Get staking info
const stakingInfo = await pyrax.staking.getInfo(address);
console.log('Staked:', stakingInfo.stakedAmount);
console.log('Rewards:', stakingInfo.pendingRewards);

// Stake tokens
const tx = await pyrax.staking.stake('1000', {
  lockPeriod: 90  // days
});
await tx.wait();

// Claim rewards
const claimTx = await pyrax.staking.claimRewards();

// Unstake
const unstakeTx = await pyrax.staking.unstake('500');
```

## Crucible (AI Compute)

```javascript
// Submit AI job
const job = await pyrax.crucible.submitJob({
  model: 'llama-7b',
  input: { prompt: 'Hello, world!' },
  budget: '10'  // PYRAX
});

// Get job status
const status = await pyrax.crucible.getJobStatus(job.id);

// Get results
const results = await pyrax.crucible.getResults(job.id);
```

## Events

### Listening to Events

```javascript
// Subscribe to contract events
const filter = contract.filters.Transfer();
contract.on(filter, (from, to, value, event) => {
  console.log(`Transfer: ${from} -> ${to}: ${value}`);
});

// Subscribe to blocks
pyrax.on('block', (blockNumber) => {
  console.log('New block:', blockNumber);
});

// Unsubscribe
contract.off(filter);
```

### Query Historical Events

```javascript
const events = await contract.queryFilter(
  contract.filters.Transfer(),
  fromBlock,
  toBlock
);
```

## Utilities

```javascript
import { utils } from '@pyrax/sdk';

// Format/parse values
const wei = utils.parseUnits('1.0', 18);  // 1 PYRAX in wei
const pyrax = utils.formatUnits(wei, 18);  // Back to PYRAX

// Address utilities
const isValid = utils.isAddress(address);
const checksummed = utils.getAddress(address);

// Hashing
const hash = utils.keccak256(data);
const id = utils.id('Transfer(address,address,uint256)');
```

## Error Handling

```javascript
import { PyraxError } from '@pyrax/sdk';

try {
  const tx = await pyrax.send({ to, value });
  await tx.wait();
} catch (error) {
  if (error instanceof PyraxError) {
    switch (error.code) {
      case 'INSUFFICIENT_FUNDS':
        console.log('Not enough PYRAX');
        break;
      case 'NONCE_EXPIRED':
        console.log('Transaction already processed');
        break;
      case 'GAS_LIMIT_EXCEEDED':
        console.log('Transaction would fail');
        break;
      default:
        console.log('Error:', error.message);
    }
  }
}
```

## TypeScript Support

Full TypeScript support included:

```typescript
import { PyraxSDK, TransactionResponse, Balance } from '@pyrax/sdk';

const pyrax = new PyraxSDK({ network: 'mainnet' });

const balance: Balance = await pyrax.getBalance(address);
const tx: TransactionResponse = await pyrax.send({ to, value });
```

## Browser Usage

```html
<script type="module">
  import { PyraxSDK } from 'https://cdn.pyrax.org/sdk/latest.js';
  
  const pyrax = new PyraxSDK({ network: 'mainnet' });
  
  // Connect MetaMask
  await pyrax.connectBrowser();
</script>
```

## Examples

### Token Transfer

```javascript
const token = pyrax.contract(tokenAddress, erc20Abi);

// Check balance
const balance = await token.balanceOf(myAddress);

// Approve spending
const approveTx = await token.approve(spenderAddress, amount);
await approveTx.wait();

// Transfer
const transferTx = await token.transfer(recipient, amount);
await transferTx.wait();
```

### Multi-send

```javascript
const recipients = [
  { address: '0x...', amount: '1.0' },
  { address: '0x...', amount: '2.0' },
];

for (const r of recipients) {
  const tx = await pyrax.send({ to: r.address, value: r.amount });
  await tx.wait();
}
```

---

:::info Resources
- [API Reference](./api-reference)
- [GitHub Repository](https://github.com/PYRAX-Chain/pyrax-sdk)
- [Examples](https://github.com/PYRAX-Chain/pyrax-sdk/tree/main/examples)
:::
