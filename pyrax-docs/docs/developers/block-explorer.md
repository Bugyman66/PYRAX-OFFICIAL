# Block Explorer

The PYRAX Block Explorer provides a web interface for exploring blockchain data.

## Explorer URLs

| Network | URL |
|---------|-----|
| Mainnet | [explorer.pyrax.org](https://explorer.pyrax.org) |
| Testnet | [testnet-explorer.pyrax.org](https://testnet-explorer.pyrax.org) |

## Features

### Block Information
- Block number and hash
- Timestamp
- Miner/validator
- Transactions list
- Gas used
- Uncle blocks

### Transaction Details
- Transaction hash
- From/to addresses
- Value transferred
- Gas price and usage
- Input data
- Event logs

### Address Information
- Balance
- Transaction history
- Token holdings
- Contract interactions
- Internal transactions

### Token Tracking
- ERC-20 tokens
- ERC-721 NFTs
- Token transfers
- Holder lists

### Contract Verification
- Verified source code
- Read/write interface
- ABI download
- Event logs

## API Access

### REST API

```bash
# Get block
curl https://explorer.pyrax.org/api/v1/blocks/12345

# Get transaction
curl https://explorer.pyrax.org/api/v1/txs/0x...

# Get address
curl https://explorer.pyrax.org/api/v1/addresses/0x...
```

### Endpoints

| Endpoint | Description |
|----------|-------------|
| `/blocks/{number}` | Block details |
| `/blocks/latest` | Latest block |
| `/txs/{hash}` | Transaction details |
| `/addresses/{address}` | Address info |
| `/addresses/{address}/txs` | Address transactions |
| `/tokens` | Token list |
| `/tokens/{address}` | Token details |
| `/stats` | Network statistics |

### Rate Limits

| Tier | Requests/min |
|------|--------------|
| Free | 30 |
| Registered | 100 |
| Pro | 1000 |

## Contract Verification

### Via Web Interface

1. Go to contract address page
2. Click "Verify & Publish"
3. Select compiler version
4. Paste source code
5. Enter constructor arguments
6. Submit for verification

### Via API

```bash
curl -X POST https://explorer.pyrax.org/api/v1/contracts/verify \
  -H "Content-Type: application/json" \
  -d '{
    "address": "0x...",
    "sourceCode": "...",
    "compilerVersion": "v0.8.20",
    "optimization": true,
    "runs": 200,
    "constructorArgs": "..."
  }'
```

### Via Hardhat

```javascript
// hardhat.config.js
module.exports = {
  etherscan: {
    apiKey: "YOUR_EXPLORER_API_KEY",
    customChains: [{
      network: "pyrax",
      chainId: 1000,
      urls: {
        apiURL: "https://explorer.pyrax.org/api",
        browserURL: "https://explorer.pyrax.org"
      }
    }]
  }
};

// Verify
// npx hardhat verify --network pyrax CONTRACT_ADDRESS "constructor arg"
```

## Search Features

### Search By
- Transaction hash
- Block number
- Address
- Token name/symbol
- Contract name

### Filters
- Date range
- Transaction type
- Token transfers
- Contract creation

## Analytics

### Network Stats
- Total transactions
- Average block time
- Gas prices
- Active addresses

### Charts
- Transaction volume
- Gas usage
- Block size
- Hash rate

### Token Analytics
- Top tokens
- Trading volume
- Holder distribution

## Embedding

### Transaction Status Widget

```html
<iframe 
  src="https://explorer.pyrax.org/embed/tx/0x..."
  width="400" 
  height="200">
</iframe>
```

### Address Balance Widget

```html
<iframe 
  src="https://explorer.pyrax.org/embed/address/0x..."
  width="300" 
  height="100">
</iframe>
```

## Self-Hosting

For private deployments:

```bash
docker run -d \
  -p 4000:4000 \
  -e RPC_URL=http://your-node:8545 \
  -e DATABASE_URL=postgres://... \
  pyrax/explorer:latest
```

---

:::info API Documentation
For complete API documentation, visit [explorer.pyrax.org/api-docs](https://explorer.pyrax.org/api-docs)
:::
