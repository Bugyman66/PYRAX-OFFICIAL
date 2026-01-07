# Quickstart Guide

Get started building on PYRAX in just a few minutes.

## Prerequisites

Ensure you have installed:
- **Node.js** 18+ ([nodejs.org](https://nodejs.org))
- **npm** or **yarn**
- **Git** ([git-scm.com](https://git-scm.com))

## Option 1: Using Hardhat

### Step 1: Create Project

```bash
mkdir my-pyrax-project
cd my-pyrax-project
npm init -y
npm install --save-dev hardhat @nomicfoundation/hardhat-toolbox
npx hardhat init
```

Select "Create a JavaScript project" when prompted.

### Step 2: Configure for PYRAX

Edit `hardhat.config.js`:

```javascript
require("@nomicfoundation/hardhat-toolbox");

/** @type import('hardhat/config').HardhatUserConfig */
module.exports = {
  solidity: "0.8.20",
  networks: {
    pyraxTestnet: {
      url: "https://testnet.pyrax.org",
      chainId: 9999, // Replace with actual chain ID
      accounts: [process.env.PRIVATE_KEY]
    },
    pyraxMainnet: {
      url: "https://rpc.pyrax.org",
      chainId: 1000, // Replace with actual chain ID
      accounts: [process.env.PRIVATE_KEY]
    }
  }
};
```

### Step 3: Create Environment File

Create `.env`:
```bash
PRIVATE_KEY=your_wallet_private_key_here
```

Install dotenv:
```bash
npm install dotenv
```

Add to `hardhat.config.js`:
```javascript
require('dotenv').config();
```

### Step 4: Write Your First Contract

Create `contracts/HelloPYRAX.sol`:

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract HelloPYRAX {
    string public message;
    address public owner;
    
    event MessageUpdated(string newMessage, address updatedBy);
    
    constructor(string memory _message) {
        message = _message;
        owner = msg.sender;
    }
    
    function updateMessage(string memory _newMessage) public {
        message = _newMessage;
        emit MessageUpdated(_newMessage, msg.sender);
    }
    
    function getGreeting() public view returns (string memory) {
        return string(abi.encodePacked("Hello from PYRAX: ", message));
    }
}
```

### Step 5: Compile

```bash
npx hardhat compile
```

### Step 6: Deploy

Create `scripts/deploy.js`:

```javascript
const hre = require("hardhat");

async function main() {
  const HelloPYRAX = await hre.ethers.getContractFactory("HelloPYRAX");
  const contract = await HelloPYRAX.deploy("Welcome to PYRAX!");
  
  await contract.waitForDeployment();
  
  console.log("HelloPYRAX deployed to:", await contract.getAddress());
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
```

Deploy to testnet:
```bash
npx hardhat run scripts/deploy.js --network pyraxTestnet
```

## Option 2: Using Foundry

### Step 1: Install Foundry

```bash
curl -L https://foundry.paradigm.xyz | bash
foundryup
```

### Step 2: Create Project

```bash
forge init my-pyrax-project
cd my-pyrax-project
```

### Step 3: Write Contract

Edit `src/Counter.sol` or create new:

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract HelloPYRAX {
    string public message;
    
    constructor(string memory _message) {
        message = _message;
    }
    
    function updateMessage(string memory _newMessage) public {
        message = _newMessage;
    }
}
```

### Step 4: Build

```bash
forge build
```

### Step 5: Deploy

```bash
forge create --rpc-url https://testnet.pyrax.org \
  --private-key $PRIVATE_KEY \
  src/HelloPYRAX.sol:HelloPYRAX \
  --constructor-args "Welcome to PYRAX!"
```

## Connect Frontend

### Install SDK

```bash
npm install ethers
```

### Connect to PYRAX

```javascript
import { ethers } from 'ethers';

// Connect to PYRAX
const provider = new ethers.JsonRpcProvider('https://rpc.pyrax.org');

// Connect wallet
const signer = new ethers.Wallet(privateKey, provider);

// Load contract
const contractAddress = 'YOUR_CONTRACT_ADDRESS';
const abi = [...]; // Your contract ABI
const contract = new ethers.Contract(contractAddress, abi, signer);

// Interact
const message = await contract.getGreeting();
console.log(message);

// Send transaction
const tx = await contract.updateMessage('New message!');
await tx.wait();
```

## Get Testnet PYRAX

You need testnet PYRAX for gas fees:

1. Visit [faucet.pyrax.org](https://faucet.pyrax.org)
2. Connect wallet or enter address
3. Request testnet tokens
4. Wait for confirmation

## Project Structure

Recommended project structure:

```
my-pyrax-project/
├── contracts/          # Solidity contracts
│   └── MyContract.sol
├── scripts/            # Deployment scripts
│   └── deploy.js
├── test/               # Test files
│   └── MyContract.test.js
├── frontend/           # Web application
│   ├── src/
│   └── package.json
├── hardhat.config.js   # Hardhat configuration
├── .env                # Environment variables
└── package.json
```

## Next Steps

### Smart Contracts
- [Contract Development](./contract-development) — Best practices
- [Contract Security](./contract-security) — Security considerations
- [Contract Deployment](./contract-deployment) — Production deployment

### Integration
- [JavaScript SDK](./javascript-sdk) — Full SDK documentation
- [RPC Endpoints](./rpc-endpoints) — All available endpoints
- [API Reference](./api-reference) — Complete API docs

### AI Integration
- [Crucible Overview](./crucible-overview) — AI computing platform
- [Job Submission](./job-submission) — Submit AI jobs

## Common Issues

### Transaction Failing
- Check you have enough PYRAX for gas
- Verify correct network configuration
- Check contract address is correct

### Connection Issues
- Verify RPC URL is correct
- Check network status
- Try alternative RPC endpoints

### Compilation Errors
- Check Solidity version matches
- Verify all imports are available
- Run `npm install` for dependencies

---

:::tip Need Help?
Join our [Discord developer channel](https://discord.gg/sS7kaacRwU) for real-time support.
:::
