# Faucet

The PYRAX Faucet provides free testnet tokens for development and testing.

## Faucet URL

**[faucet.pyrax.org](https://faucet.pyrax.org)**

## How to Use

### Web Interface

1. Visit [faucet.pyrax.org](https://faucet.pyrax.org)
2. Connect wallet or enter address
3. Complete captcha verification
4. Click "Request Tokens"
5. Wait for transaction confirmation
6. Tokens appear in your wallet

### Discord Bot

In the [PYRAX Discord](https://discord.gg/sS7kaacRwU) #faucet channel:

```
!faucet 0xYourWalletAddress
```

### API

```bash
curl -X POST https://faucet.pyrax.org/api/request \
  -H "Content-Type: application/json" \
  -d '{"address": "0x..."}'
```

Response:
```json
{
  "success": true,
  "txHash": "0x...",
  "amount": "100"
}
```

## Limits

| Limit | Value |
|-------|-------|
| Amount per request | 100 PYRAX |
| Cooldown | 24 hours |
| Daily total | 100 PYRAX per address |

## Troubleshooting

### "Address already requested"

You've already received tokens in the last 24 hours. Wait for the cooldown to expire.

### "Invalid address"

Check that you're entering a valid Ethereum-style address (0x followed by 40 hex characters).

### "Faucet empty"

The faucet may be temporarily out of funds. It's automatically refilled — try again later.

### Tokens not appearing

1. Verify you're on testnet network
2. Check transaction on [testnet-explorer.pyrax.org](https://testnet-explorer.pyrax.org)
3. Wait a few minutes for confirmation

## For Bulk Testing

If you need more tokens for extensive testing:

1. Join Discord
2. Explain your project in #dev-support
3. Request additional testnet allocation

## Programmatic Access

### JavaScript

```javascript
async function requestTestnetTokens(address) {
  const response = await fetch('https://faucet.pyrax.org/api/request', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ address })
  });
  
  const data = await response.json();
  
  if (data.success) {
    console.log(`Received tokens! TX: ${data.txHash}`);
  } else {
    console.log(`Error: ${data.error}`);
  }
}
```

### Python

```python
import requests

def request_testnet_tokens(address):
    response = requests.post(
        'https://faucet.pyrax.org/api/request',
        json={'address': address}
    )
    data = response.json()
    
    if data['success']:
        print(f"Received tokens! TX: {data['txHash']}")
    else:
        print(f"Error: {data['error']}")
```

## Running Your Own Faucet

For private testnets:

```bash
docker run -d \
  -p 3000:3000 \
  -e RPC_URL=http://your-node:8545 \
  -e FAUCET_PRIVATE_KEY=0x... \
  -e AMOUNT=100 \
  -e COOLDOWN=86400 \
  pyrax/faucet:latest
```

---

:::info Need More?
For development requiring large amounts of testnet PYRAX, contact us on [Discord](https://discord.gg/sS7kaacRwU).
:::
