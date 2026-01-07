# Contract Development

This guide covers best practices for developing smart contracts on PYRAX.

## Project Setup

### Using Hardhat

```bash
# Create project
mkdir my-contract
cd my-contract
npm init -y

# Install dependencies
npm install --save-dev hardhat @nomicfoundation/hardhat-toolbox
npm install @openzeppelin/contracts

# Initialize
npx hardhat init
```

### Project Structure

```
my-contract/
├── contracts/           # Solidity files
│   ├── MyContract.sol
│   └── interfaces/
├── scripts/            # Deployment scripts
│   └── deploy.js
├── test/               # Test files
│   └── MyContract.test.js
├── hardhat.config.js   # Configuration
└── package.json
```

## Writing Contracts

### Contract Template

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";

/**
 * @title MyContract
 * @dev Example contract with best practices
 */
contract MyContract is Ownable, ReentrancyGuard {
    // State variables
    uint256 public value;
    mapping(address => uint256) public balances;
    
    // Events
    event ValueUpdated(uint256 indexed newValue, address indexed updatedBy);
    event Deposited(address indexed user, uint256 amount);
    
    // Errors (gas efficient)
    error InsufficientBalance(uint256 requested, uint256 available);
    error InvalidAmount();
    
    // Constructor
    constructor(uint256 _initialValue) Ownable(msg.sender) {
        value = _initialValue;
    }
    
    // External functions
    function deposit() external payable nonReentrant {
        if (msg.value == 0) revert InvalidAmount();
        balances[msg.sender] += msg.value;
        emit Deposited(msg.sender, msg.value);
    }
    
    function withdraw(uint256 amount) external nonReentrant {
        uint256 balance = balances[msg.sender];
        if (amount > balance) {
            revert InsufficientBalance(amount, balance);
        }
        
        balances[msg.sender] -= amount;
        
        (bool success, ) = msg.sender.call{value: amount}("");
        require(success, "Transfer failed");
    }
    
    // Admin functions
    function setValue(uint256 _value) external onlyOwner {
        value = _value;
        emit ValueUpdated(_value, msg.sender);
    }
    
    // View functions
    function getBalance(address user) external view returns (uint256) {
        return balances[user];
    }
}
```

### Using Interfaces

```solidity
// interfaces/IMyContract.sol
interface IMyContract {
    function deposit() external payable;
    function withdraw(uint256 amount) external;
    function getBalance(address user) external view returns (uint256);
    
    event Deposited(address indexed user, uint256 amount);
}
```

## Testing

### Comprehensive Tests

```javascript
const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("MyContract", function () {
  let contract;
  let owner;
  let user1;
  let user2;
  
  beforeEach(async function () {
    [owner, user1, user2] = await ethers.getSigners();
    
    const MyContract = await ethers.getContractFactory("MyContract");
    contract = await MyContract.deploy(100);
  });
  
  describe("Deployment", function () {
    it("should set initial value", async function () {
      expect(await contract.value()).to.equal(100);
    });
    
    it("should set owner", async function () {
      expect(await contract.owner()).to.equal(owner.address);
    });
  });
  
  describe("Deposits", function () {
    it("should accept deposits", async function () {
      const amount = ethers.parseEther("1.0");
      
      await expect(contract.connect(user1).deposit({ value: amount }))
        .to.emit(contract, "Deposited")
        .withArgs(user1.address, amount);
      
      expect(await contract.getBalance(user1.address)).to.equal(amount);
    });
    
    it("should reject zero deposits", async function () {
      await expect(
        contract.connect(user1).deposit({ value: 0 })
      ).to.be.revertedWithCustomError(contract, "InvalidAmount");
    });
  });
  
  describe("Withdrawals", function () {
    beforeEach(async function () {
      await contract.connect(user1).deposit({ 
        value: ethers.parseEther("2.0") 
      });
    });
    
    it("should allow withdrawal", async function () {
      const amount = ethers.parseEther("1.0");
      const balanceBefore = await ethers.provider.getBalance(user1.address);
      
      const tx = await contract.connect(user1).withdraw(amount);
      const receipt = await tx.wait();
      const gasCost = receipt.gasUsed * receipt.gasPrice;
      
      const balanceAfter = await ethers.provider.getBalance(user1.address);
      expect(balanceAfter).to.equal(balanceBefore + amount - gasCost);
    });
    
    it("should reject over-withdrawal", async function () {
      const amount = ethers.parseEther("3.0");
      
      await expect(
        contract.connect(user1).withdraw(amount)
      ).to.be.revertedWithCustomError(contract, "InsufficientBalance");
    });
  });
  
  describe("Admin functions", function () {
    it("should allow owner to set value", async function () {
      await contract.setValue(200);
      expect(await contract.value()).to.equal(200);
    });
    
    it("should reject non-owner", async function () {
      await expect(
        contract.connect(user1).setValue(200)
      ).to.be.revertedWithCustomError(contract, "OwnableUnauthorizedAccount");
    });
  });
});
```

### Running Tests

```bash
# Run all tests
npx hardhat test

# Run with gas reporting
REPORT_GAS=true npx hardhat test

# Run specific test
npx hardhat test test/MyContract.test.js

# Run with coverage
npx hardhat coverage
```

## Design Patterns

### Factory Pattern

```solidity
contract TokenFactory {
    address[] public tokens;
    
    event TokenCreated(address indexed token, string name);
    
    function createToken(
        string memory name,
        string memory symbol,
        uint256 supply
    ) external returns (address) {
        Token token = new Token(name, symbol, supply, msg.sender);
        tokens.push(address(token));
        emit TokenCreated(address(token), name);
        return address(token);
    }
}
```

### Proxy Pattern

```solidity
import "@openzeppelin/contracts/proxy/transparent/TransparentUpgradeableProxy.sol";

// Implementation
contract MyContractV1 {
    uint256 public value;
    
    function initialize(uint256 _value) external {
        value = _value;
    }
}

// Deploy proxy pointing to implementation
```

### Access Control

```solidity
import "@openzeppelin/contracts/access/AccessControl.sol";

contract Managed is AccessControl {
    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");
    bytes32 public constant OPERATOR_ROLE = keccak256("OPERATOR_ROLE");
    
    constructor() {
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _grantRole(ADMIN_ROLE, msg.sender);
    }
    
    function adminFunction() external onlyRole(ADMIN_ROLE) {
        // Admin only
    }
    
    function operatorFunction() external onlyRole(OPERATOR_ROLE) {
        // Operator only
    }
}
```

## Gas Optimization

### Efficient Storage

```solidity
// Pack structs
struct User {
    uint128 balance;     // 16 bytes
    uint64 lastAction;   // 8 bytes
    uint64 actionCount;  // 8 bytes
    // Total: 32 bytes (1 slot)
}

// Use bytes32 for small strings
bytes32 public constant NAME = "MyContract";
```

### Efficient Operations

```solidity
// Use unchecked for safe math
function increment() external {
    unchecked {
        counter++;  // Safe if counter won't overflow
    }
}

// Cache storage reads
function process() external {
    uint256 _value = value;  // Cache
    // Use _value multiple times
}
```

## Debugging

### Console Logging

```solidity
import "hardhat/console.sol";

function myFunction() public {
    console.log("Value:", value);
    console.log("Sender:", msg.sender);
}
```

### Error Messages

```solidity
// Custom errors (gas efficient)
error Unauthorized(address caller);
error InvalidState(uint256 current, uint256 expected);

// Revert with error
if (msg.sender != owner) {
    revert Unauthorized(msg.sender);
}
```

---

:::tip Next Steps
- [Contract Deployment](./contract-deployment) — Deploy your contract
- [Contract Security](./contract-security) — Security review
:::
