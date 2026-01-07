# Testnet

The PYRAX Testnet is a testing environment for developers to build and test applications before mainnet deployment.

## Network Details

| Parameter | Value |
|-----------|-------|
| Network Name | PYRAX Testnet |
| Chain ID | TBD |
| Currency | PYRAX (test) |
| Block Time | ~60 seconds |

## Endpoints

| Type | URL |
|------|-----|
| HTTP RPC | `https://testnet.pyrax.org` |
| WebSocket | `wss://ws-testnet.pyrax.org` |
| Block Explorer | [testnet-explorer.pyrax.org](https://testnet-explorer.pyrax.org) |
| Faucet | [faucet.pyrax.org](https://faucet.pyrax.org) |

## Adding to Wallet

### MetaMask (Manual)

1. Open MetaMask
2. Click network dropdown → Add Network
3. Enter:
   - Network Name: `PYRAX Testnet`
   - RPC URL: `https://testnet.pyrax.org`
   - Chain ID: `TBD`
   - Currency Symbol: `PYRAX`
   - Explorer: `https://testnet-explorer.pyrax.org`

### Automatic

Visit [testnet.pyrax.org/add-network](https://testnet.pyrax.org/add-network) and click "Add to MetaMask".

## Getting Testnet PYRAX

### Faucet

1. Visit [faucet.pyrax.org](https://faucet.pyrax.org)
2. Enter your wallet address
3. Complete captcha
4. Click "Request Tokens"
5. Receive 100 test PYRAX

**Limits:**
- 100 PYRAX per request
- 24-hour cooldown per address

### Discord Bot

In the [PYRAX Discord](https://discord.gg/sS7kaacRwU):

```
!faucet 0xYourAddress
```

## Development Setup

### Hardhat

```javascript
// hardhat.config.js
module.exports = {
  networks: {
    pyraxTestnet: {
      url: "https://testnet.pyrax.org",
      chainId: 9999,  // Replace with actual
      accounts: [process.env.PRIVATE_KEY]
    }
  }
};
```

### Foundry

```toml
# foundry.toml
[rpc_endpoints]
pyrax_testnet = "https://testnet.pyrax.org"
```

### ethers.js

```javascript
import { ethers } from 'ethers';

const provider = new ethers.JsonRpcProvider('https://testnet.pyrax.org');
const wallet = new ethers.Wallet(privateKey, provider);
```

## Testing Best Practices

### Before Testnet

1. Write comprehensive unit tests
2. Test on local hardhat network
3. Run static analysis

### On Testnet

1. Deploy to testnet first
2. Test all functions
3. Check gas usage
4. Verify contract on explorer
5. Test with multiple accounts

### Before Mainnet

1. Full testnet testing complete
2. Security audit (if applicable)
3. Documentation ready
4. Deployment script verified

## Testnet vs Mainnet

| Aspect | Testnet | Mainnet |
|--------|---------|---------|
| Tokens | Free (faucet) | Real value |
| Risk | None | Financial risk |
| Data | May be reset | Permanent |
| Speed | Sometimes slower | Production speed |

## Known Differences

The testnet aims to match mainnet but may have:
- Different block times during load
- Periodic resets
- Newer features for testing
- Lower gas prices

## Testnet Resets

The testnet may be reset periodically:
- Announced in Discord
- All data cleared
- Re-request faucet tokens after reset

## Debugging on Testnet

### Transaction Failed

```javascript
try {
  const tx = await contract.someFunction();
  await tx.wait();
} catch (error) {
  // Get revert reason
  console.log('Revert reason:', error.reason);
}
```

### Check on Explorer

1. Copy transaction hash
2. Search on [testnet-explorer.pyrax.org](https://testnet-explorer.pyrax.org)
3. View error message in details

### Enable Debug Logs

```javascript
// ethers.js
const tx = await contract.someFunction();
const receipt = await tx.wait();
console.log('Gas used:', receipt.gasUsed.toString());
console.log('Logs:', receipt.logs);
```

## Testnet Support

- **Discord**: #testnet-support channel
- **GitHub**: Open issues for bugs
- **Forum**: Development discussions

---

:::tip Development Flow
1. Write & test locally
2. Deploy to testnet
3. Full testing on testnet
4. Get testnet community feedback
5. Deploy to mainnet
:::
