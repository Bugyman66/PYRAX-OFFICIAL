# Smart Contracts Introduction

PYRAX supports EVM-compatible smart contracts. This guide introduces smart contract development on PYRAX.

## What are Smart Contracts?

Smart contracts are self-executing programs stored on the blockchain:
- Code runs automatically when conditions are met
- Immutable once deployed
- Transparent and verifiable
- Trustless execution

## EVM Compatibility

PYRAX uses the Ethereum Virtual Machine (EVM):

**Supported:**
- Solidity 0.8+
- Vyper
- All EVM opcodes
- Standard precompiles

**Benefits:**
- Existing Ethereum code works
- Standard tooling (Hardhat, Foundry)
- OpenZeppelin libraries
- Battle-tested patterns

## Your First Contract

### Simple Storage Contract

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract SimpleStorage {
    uint256 private value;
    address public owner;
    
    event ValueChanged(uint256 newValue, address changedBy);
    
    constructor(uint256 _initialValue) {
        value = _initialValue;
        owner = msg.sender;
    }
    
    function setValue(uint256 _value) public {
        value = _value;
        emit ValueChanged(_value, msg.sender);
    }
    
    function getValue() public view returns (uint256) {
        return value;
    }
}
```

### Key Components

| Component | Purpose |
|-----------|---------|
| `pragma` | Compiler version |
| `contract` | Contract definition |
| `state variables` | Persistent storage |
| `constructor` | Initialization |
| `functions` | Contract logic |
| `events` | Logging/notifications |
| `modifiers` | Access control |

## Contract Types

### Token Contracts

**ERC-20 (Fungible Tokens):**
```solidity
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";

contract MyToken is ERC20 {
    constructor() ERC20("MyToken", "MTK") {
        _mint(msg.sender, 1000000 * 10**decimals());
    }
}
```

**ERC-721 (NFTs):**
```solidity
import "@openzeppelin/contracts/token/ERC721/ERC721.sol";

contract MyNFT is ERC721 {
    uint256 private _tokenIds;
    
    constructor() ERC721("MyNFT", "MNFT") {}
    
    function mint(address to) public returns (uint256) {
        _tokenIds++;
        _safeMint(to, _tokenIds);
        return _tokenIds;
    }
}
```

### DeFi Contracts

- Lending protocols
- DEX/AMM
- Staking pools
- Yield farming

### Governance Contracts

- Voting mechanisms
- Proposal systems
- Timelock controllers

## Solidity Basics

### Data Types

```solidity
// Value types
bool isActive = true;
uint256 amount = 100;
int256 balance = -50;
address wallet = 0x123...;
bytes32 hash = keccak256("data");

// Reference types
string name = "PYRAX";
bytes data = hex"1234";
uint256[] numbers;
mapping(address => uint256) balances;
```

### Visibility

| Visibility | Access |
|------------|--------|
| `public` | Anyone |
| `private` | Contract only |
| `internal` | Contract + children |
| `external` | External calls only |

### Function Modifiers

```solidity
modifier onlyOwner() {
    require(msg.sender == owner, "Not owner");
    _;
}

function adminFunction() public onlyOwner {
    // Only owner can call
}
```

### View vs Pure

```solidity
// view: reads state
function getBalance() public view returns (uint256) {
    return balances[msg.sender];
}

// pure: no state access
function add(uint a, uint b) public pure returns (uint) {
    return a + b;
}
```

## Gas Optimization

### Tips

1. **Use appropriate types**: `uint256` is often cheaper than `uint8`
2. **Pack storage**: Group small types together
3. **Use memory for temporary data**
4. **Avoid loops over unbounded arrays**
5. **Use events instead of storage for logs**

### Storage Packing

```solidity
// Bad: 3 storage slots
contract Bad {
    uint256 a;  // slot 0
    uint8 b;    // slot 1
    uint256 c;  // slot 2
}

// Good: 2 storage slots
contract Good {
    uint256 a;  // slot 0
    uint256 c;  // slot 1
    uint8 b;    // slot 1 (packed)
}
```

## Testing Contracts

### Using Hardhat

```javascript
const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("SimpleStorage", function () {
  let storage;
  
  beforeEach(async function () {
    const SimpleStorage = await ethers.getContractFactory("SimpleStorage");
    storage = await SimpleStorage.deploy(100);
  });
  
  it("should return initial value", async function () {
    expect(await storage.getValue()).to.equal(100);
  });
  
  it("should update value", async function () {
    await storage.setValue(200);
    expect(await storage.getValue()).to.equal(200);
  });
});
```

### Run Tests

```bash
npx hardhat test
```

## Security Considerations

### Common Vulnerabilities

| Vulnerability | Description |
|---------------|-------------|
| Reentrancy | Recursive calls exploit |
| Integer overflow | Math errors (pre-0.8) |
| Access control | Missing permissions |
| Front-running | Transaction ordering |

### Best Practices

1. Use OpenZeppelin libraries
2. Follow checks-effects-interactions
3. Implement access control
4. Get security audits
5. Start with small amounts

See [Contract Security](./contract-security) for details.

## Development Workflow

```
1. Design    → Plan contract architecture
2. Develop   → Write Solidity code
3. Test      → Unit and integration tests
4. Audit     → Security review
5. Deploy    → Testnet first, then mainnet
6. Verify    → Publish source code
7. Monitor   → Watch for issues
```

## Resources

### Learning
- [Solidity Documentation](https://docs.soliditylang.org)
- [OpenZeppelin Learn](https://docs.openzeppelin.com/learn)
- [CryptoZombies](https://cryptozombies.io)

### Libraries
- [OpenZeppelin Contracts](https://github.com/OpenZeppelin/openzeppelin-contracts)
- [Solmate](https://github.com/transmissions11/solmate)

### Tools
- [Hardhat](https://hardhat.org)
- [Foundry](https://book.getfoundry.sh)
- [Remix IDE](https://remix.ethereum.org)

---

:::tip Next Steps
- [Contract Development](./contract-development) — Detailed development guide
- [Contract Deployment](./contract-deployment) — Deploy to PYRAX
- [Contract Security](./contract-security) — Security best practices
:::
