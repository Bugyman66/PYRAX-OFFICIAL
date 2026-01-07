# Python SDK

The PYRAX Python SDK provides easy integration with the PYRAX network for Python developers.

## Installation

```bash
pip install pyrax-sdk
```

## Quick Start

```python
from pyrax import PyraxClient

# Initialize
client = PyraxClient(network='mainnet')

# Get balance
balance = client.get_balance('0x...')
print(f'Balance: {balance} PYRAX')
```

## Configuration

### Basic Setup

```python
from pyrax import PyraxClient

# Mainnet
client = PyraxClient(network='mainnet')

# Testnet
client = PyraxClient(network='testnet')
```

### Custom RPC

```python
client = PyraxClient(
    rpc_url='https://your-node.example.com'
)
```

### With Private Key

```python
import os

client = PyraxClient(
    network='mainnet',
    private_key=os.environ['PRIVATE_KEY']
)
```

## Core Functions

### Account Operations

```python
# Get balance
balance = client.get_balance(address)

# Get balance in wei
balance_wei = client.get_balance(address, unit='wei')

# Get nonce
nonce = client.get_transaction_count(address)
```

### Transactions

```python
# Send PYRAX
tx_hash = client.send_transaction(
    to='0x...',
    value=1.0  # PYRAX
)

# Wait for confirmation
receipt = client.wait_for_transaction(tx_hash)
print(f'Block: {receipt.block_number}')

# With options
tx_hash = client.send_transaction(
    to='0x...',
    value=1.0,
    gas_limit=21000,
    max_fee_per_gas=50_000_000_000  # wei
)
```

### Contract Interaction

```python
# Load contract
contract = client.contract(address, abi)

# Read (view function)
value = contract.functions.getValue().call()

# Write (state-changing)
tx_hash = contract.functions.setValue(100).transact()
receipt = client.wait_for_transaction(tx_hash)
```

## Staking

```python
# Get staking info
info = client.staking.get_info(address)
print(f'Staked: {info.staked_amount}')
print(f'Rewards: {info.pending_rewards}')

# Stake tokens
tx = client.staking.stake(
    amount=1000,
    lock_period_days=90
)

# Claim rewards
tx = client.staking.claim_rewards()

# Unstake
tx = client.staking.unstake(amount=500)
```

## Crucible (AI Compute)

```python
# Submit AI job
job = client.crucible.submit_job(
    model='llama-7b',
    input_data={'prompt': 'Hello, world!'},
    budget=10  # PYRAX
)

# Check status
status = client.crucible.get_job_status(job.id)
print(f'Status: {status}')

# Get results
results = client.crucible.get_results(job.id)
```

## Events

### Query Events

```python
# Get past events
events = contract.events.Transfer.get_logs(
    from_block=0,
    to_block='latest'
)

for event in events:
    print(f'{event.args.from_} -> {event.args.to}: {event.args.value}')
```

### Watch Events

```python
# Real-time event watching
def handle_transfer(event):
    print(f'Transfer: {event}')

event_filter = contract.events.Transfer.create_filter(from_block='latest')
for event in event_filter.get_new_entries():
    handle_transfer(event)
```

## Utilities

```python
from pyrax import utils

# Parse/format values
wei = utils.to_wei(1.0, 'ether')  # 1 PYRAX in wei
pyrax = utils.from_wei(wei, 'ether')  # Back to PYRAX

# Address validation
is_valid = utils.is_address(address)
checksummed = utils.to_checksum_address(address)

# Hashing
hash = utils.keccak256(data)
```

## Error Handling

```python
from pyrax import PyraxError, InsufficientFundsError

try:
    tx = client.send_transaction(to=address, value=1000)
except InsufficientFundsError:
    print('Not enough PYRAX')
except PyraxError as e:
    print(f'Error: {e.message}')
```

## Async Support

```python
import asyncio
from pyrax import AsyncPyraxClient

async def main():
    client = AsyncPyraxClient(network='mainnet')
    
    # Async operations
    balance = await client.get_balance(address)
    
    # Parallel requests
    balances = await asyncio.gather(*[
        client.get_balance(addr) for addr in addresses
    ])

asyncio.run(main())
```

## Examples

### Token Transfer

```python
# Load ERC-20 contract
token = client.contract(token_address, erc20_abi)

# Check balance
balance = token.functions.balanceOf(my_address).call()

# Approve
tx = token.functions.approve(spender, amount).transact()
client.wait_for_transaction(tx)

# Transfer
tx = token.functions.transfer(recipient, amount).transact()
client.wait_for_transaction(tx)
```

### Batch Operations

```python
from pyrax import BatchRequest

batch = BatchRequest(client)
batch.add_balance_request('0x...')
batch.add_balance_request('0x...')
batch.add_block_request('latest')

results = batch.execute()
```

### Data Analysis

```python
import pandas as pd

# Get block range
blocks = []
for i in range(start_block, end_block):
    block = client.get_block(i)
    blocks.append({
        'number': block.number,
        'timestamp': block.timestamp,
        'tx_count': len(block.transactions),
        'gas_used': block.gas_used
    })

df = pd.DataFrame(blocks)
print(df.describe())
```

## CLI Tool

The SDK includes a CLI:

```bash
# Get balance
pyrax balance 0x...

# Send transaction
pyrax send --to 0x... --value 1.0

# Get block
pyrax block latest
```

---

:::info Resources
- [API Reference](./api-reference)
- [PyPI Package](https://pypi.org/project/pyrax-sdk)
- [GitHub Repository](https://github.com/PYRAX-Chain/pyrax-python)
:::
