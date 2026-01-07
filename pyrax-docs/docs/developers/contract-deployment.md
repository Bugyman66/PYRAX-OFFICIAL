# Contract Deployment

This guide covers deploying smart contracts to PYRAX networks.

## Deployment Checklist

Before deploying:
- [ ] All tests passing
- [ ] Code reviewed
- [ ] Gas optimized
- [ ] Security audit (for production)
- [ ] Testnet deployment verified
- [ ] Deployment script tested

## Deployment with Hardhat

### Configure Networks

```javascript
// hardhat.config.js
require("@nomicfoundation/hardhat-toolbox");
require('dotenv').config();

module.exports = {
  solidity: "0.8.20",
  networks: {
    pyraxTestnet: {
      url: "https://testnet.pyrax.org",
      chainId: 9999,
      accounts: [process.env.PRIVATE_KEY]
    },
    pyraxMainnet: {
      url: "https://rpc.pyrax.org",
      chainId: 1000,
      accounts: [process.env.PRIVATE_KEY]
    }
  }
};
```

### Deployment Script

```javascript
// scripts/deploy.js
const hre = require("hardhat");

async function main() {
  console.log("Deploying to", hre.network.name);
  
  // Get deployer
  const [deployer] = await hre.ethers.getSigners();
  console.log("Deployer:", deployer.address);
  console.log("Balance:", await hre.ethers.provider.getBalance(deployer.address));
  
  // Deploy contract
  const MyContract = await hre.ethers.getContractFactory("MyContract");
  const contract = await MyContract.deploy(100); // Constructor args
  
  await contract.waitForDeployment();
  const address = await contract.getAddress();
  
  console.log("Contract deployed to:", address);
  
  // Wait for confirmations
  console.log("Waiting for confirmations...");
  await contract.deploymentTransaction().wait(5);
  
  console.log("Deployment confirmed!");
  
  return address;
}

main()
  .then((address) => {
    console.log("Success! Contract at:", address);
    process.exit(0);
  })
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
```

### Deploy

```bash
# Testnet
npx hardhat run scripts/deploy.js --network pyraxTestnet

# Mainnet
npx hardhat run scripts/deploy.js --network pyraxMainnet
```

## Deployment with Foundry

### Configure

```toml
# foundry.toml
[profile.default]
src = "src"
out = "out"
libs = ["lib"]

[rpc_endpoints]
pyrax_testnet = "https://testnet.pyrax.org"
pyrax_mainnet = "https://rpc.pyrax.org"
```

### Deploy Script

```solidity
// script/Deploy.s.sol
pragma solidity ^0.8.20;

import "forge-std/Script.sol";
import "../src/MyContract.sol";

contract DeployScript is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        
        vm.startBroadcast(deployerPrivateKey);
        
        MyContract myContract = new MyContract(100);
        
        vm.stopBroadcast();
        
        console.log("Deployed to:", address(myContract));
    }
}
```

### Deploy

```bash
forge script script/Deploy.s.sol --rpc-url pyrax_testnet --broadcast
```

## Verification

### Verify on Explorer

```bash
npx hardhat verify --network pyraxMainnet CONTRACT_ADDRESS CONSTRUCTOR_ARGS
```

### Flatten for Manual Verification

```bash
npx hardhat flatten contracts/MyContract.sol > Flattened.sol
```

## Upgradeable Contracts

### Using OpenZeppelin Upgrades

```bash
npm install @openzeppelin/hardhat-upgrades
```

```javascript
// hardhat.config.js
require("@openzeppelin/hardhat-upgrades");
```

### Deploy Proxy

```javascript
const { ethers, upgrades } = require("hardhat");

async function main() {
  const MyContract = await ethers.getContractFactory("MyContract");
  
  const proxy = await upgrades.deployProxy(MyContract, [100], {
    initializer: 'initialize'
  });
  
  await proxy.waitForDeployment();
  console.log("Proxy:", await proxy.getAddress());
}
```

### Upgrade

```javascript
async function upgrade() {
  const MyContractV2 = await ethers.getContractFactory("MyContractV2");
  
  await upgrades.upgradeProxy(PROXY_ADDRESS, MyContractV2);
  console.log("Upgraded!");
}
```

## Multi-Contract Deployment

```javascript
async function deployAll() {
  const addresses = {};
  
  // Deploy Token
  const Token = await ethers.getContractFactory("Token");
  const token = await Token.deploy();
  await token.waitForDeployment();
  addresses.token = await token.getAddress();
  
  // Deploy using token address
  const Staking = await ethers.getContractFactory("Staking");
  const staking = await Staking.deploy(addresses.token);
  await staking.waitForDeployment();
  addresses.staking = await staking.getAddress();
  
  // Save addresses
  const fs = require('fs');
  fs.writeFileSync(
    'deployed-addresses.json',
    JSON.stringify(addresses, null, 2)
  );
  
  return addresses;
}
```

## Gas Estimation

```javascript
async function estimateDeployment() {
  const MyContract = await ethers.getContractFactory("MyContract");
  const deployTx = await MyContract.getDeployTransaction(100);
  
  const gasEstimate = await ethers.provider.estimateGas(deployTx);
  const feeData = await ethers.provider.getFeeData();
  
  const cost = gasEstimate * feeData.gasPrice;
  console.log("Estimated gas:", gasEstimate.toString());
  console.log("Estimated cost:", ethers.formatEther(cost), "PYRAX");
}
```

## Post-Deployment

### Verify Deployment

```javascript
async function verify(address) {
  const contract = await ethers.getContractAt("MyContract", address);
  
  // Check state
  console.log("Value:", await contract.value());
  console.log("Owner:", await contract.owner());
  
  // Test function
  const tx = await contract.setValue(200);
  await tx.wait();
  console.log("New value:", await contract.value());
}
```

### Transfer Ownership

```javascript
async function transferOwnership(contractAddress, newOwner) {
  const contract = await ethers.getContractAt("Ownable", contractAddress);
  const tx = await contract.transferOwnership(newOwner);
  await tx.wait();
  console.log("Ownership transferred to:", newOwner);
}
```

## Best Practices

1. **Always test on testnet first**
2. **Use hardware wallet for mainnet**
3. **Keep deployment records**
4. **Verify contracts on explorer**
5. **Document constructor arguments**
6. **Plan for upgrades if needed**

---

:::info Next Steps
- [Contract Security](./contract-security) — Security review
- [Testnet](./testnet) — Get testnet tokens
:::
