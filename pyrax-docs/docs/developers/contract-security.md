# Contract Security

Security is critical for smart contracts. This guide covers common vulnerabilities and best practices.

## Common Vulnerabilities

### Reentrancy

**Problem:** External calls can re-enter your contract before state updates.

```solidity
// VULNERABLE
function withdraw() external {
    uint256 amount = balances[msg.sender];
    (bool success,) = msg.sender.call{value: amount}("");
    require(success);
    balances[msg.sender] = 0;  // Updated AFTER external call
}
```

**Solution:** Use checks-effects-interactions pattern or ReentrancyGuard.

```solidity
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";

// SAFE
function withdraw() external nonReentrant {
    uint256 amount = balances[msg.sender];
    balances[msg.sender] = 0;  // Update BEFORE external call
    (bool success,) = msg.sender.call{value: amount}("");
    require(success);
}
```

### Access Control

**Problem:** Missing or incorrect access restrictions.

```solidity
// VULNERABLE
function setAdmin(address newAdmin) external {
    admin = newAdmin;  // Anyone can call!
}
```

**Solution:** Implement proper access control.

```solidity
import "@openzeppelin/contracts/access/Ownable.sol";

// SAFE
function setAdmin(address newAdmin) external onlyOwner {
    admin = newAdmin;
}
```

### Integer Overflow/Underflow

**Problem:** Math operations wrap around (pre-Solidity 0.8).

```solidity
// Pre-0.8: uint8 max = 255, 255 + 1 = 0
```

**Solution:** Solidity 0.8+ has built-in checks. Use `unchecked` carefully.

```solidity
// Solidity 0.8+ is safe by default
uint256 result = a + b;  // Reverts on overflow

// Only use unchecked when CERTAIN it's safe
unchecked {
    counter++;  // Only if counter won't overflow
}
```

### Front-Running

**Problem:** Miners/validators can see and reorder transactions.

**Solutions:**
- Commit-reveal schemes
- Flashbots-style private transactions
- Slippage protection

```solidity
// Slippage protection example
function swap(uint256 amountIn, uint256 minAmountOut) external {
    uint256 amountOut = calculateOutput(amountIn);
    require(amountOut >= minAmountOut, "Slippage exceeded");
    // Proceed with swap
}
```

### Denial of Service

**Problem:** Contracts can be made unusable.

```solidity
// VULNERABLE: Unbounded loop
function distributeRewards() external {
    for (uint i = 0; i < users.length; i++) {  // Could run out of gas
        users[i].transfer(rewards[i]);
    }
}
```

**Solution:** Use pull patterns instead of push.

```solidity
// SAFE: Pull pattern
mapping(address => uint256) public pendingRewards;

function claimReward() external {
    uint256 amount = pendingRewards[msg.sender];
    pendingRewards[msg.sender] = 0;
    payable(msg.sender).transfer(amount);
}
```

## Security Patterns

### Checks-Effects-Interactions

Always follow this order:
1. **Checks**: Validate inputs and conditions
2. **Effects**: Update state
3. **Interactions**: External calls

```solidity
function transfer(address to, uint256 amount) external {
    // CHECKS
    require(balances[msg.sender] >= amount, "Insufficient");
    require(to != address(0), "Invalid address");
    
    // EFFECTS
    balances[msg.sender] -= amount;
    balances[to] += amount;
    
    // INTERACTIONS
    emit Transfer(msg.sender, to, amount);
}
```

### Pull Over Push

Let users withdraw rather than pushing funds.

```solidity
// Push (risky)
function distribute() external {
    for (uint i = 0; i < recipients.length; i++) {
        recipients[i].transfer(amounts[i]);
    }
}

// Pull (safer)
function claim() external {
    uint256 amount = pendingClaims[msg.sender];
    delete pendingClaims[msg.sender];
    payable(msg.sender).transfer(amount);
}
```

### Emergency Stop

Include pause functionality for emergencies.

```solidity
import "@openzeppelin/contracts/security/Pausable.sol";

contract MyContract is Pausable {
    function criticalFunction() external whenNotPaused {
        // Only works when not paused
    }
    
    function pause() external onlyOwner {
        _pause();
    }
    
    function unpause() external onlyOwner {
        _unpause();
    }
}
```

## Audit Checklist

### Code Quality
- [ ] Compiler version locked
- [ ] No compiler warnings
- [ ] Consistent naming conventions
- [ ] Adequate documentation

### Access Control
- [ ] All admin functions protected
- [ ] Role hierarchy documented
- [ ] Ownership transfer is two-step
- [ ] Renounce ownership considered

### External Calls
- [ ] Reentrancy protection
- [ ] Return values checked
- [ ] Gas limits considered
- [ ] Fallback behavior handled

### Math Operations
- [ ] No overflow/underflow risks
- [ ] Division by zero prevented
- [ ] Precision loss minimized
- [ ] Rounding documented

### Token Handling
- [ ] ERC-20 quirks handled (USDT, etc.)
- [ ] SafeERC20 used
- [ ] Approve race condition mitigated

## Testing for Security

### Fuzz Testing

```javascript
it("should handle random inputs", async function () {
    for (let i = 0; i < 100; i++) {
        const randomValue = Math.floor(Math.random() * 1000000);
        await contract.setValue(randomValue);
        expect(await contract.getValue()).to.equal(randomValue);
    }
});
```

### Invariant Testing (Foundry)

```solidity
function invariant_totalSupplyConstant() public {
    assertEq(token.totalSupply(), INITIAL_SUPPLY);
}
```

### Symbolic Execution

Tools like Mythril and Manticore can find vulnerabilities automatically.

## Recommended Tools

| Tool | Purpose |
|------|---------|
| Slither | Static analysis |
| Mythril | Symbolic execution |
| Echidna | Fuzzing |
| Foundry | Fuzz + invariant testing |
| OpenZeppelin | Audited libraries |

## Audit Process

1. **Internal Review**: Team reviews code
2. **Static Analysis**: Run automated tools
3. **External Audit**: Professional security firm
4. **Bug Bounty**: Ongoing community review

### Audit Firms
- OpenZeppelin
- Trail of Bits
- Consensys Diligence
- Halborn

---

:::warning Before Mainnet
Never deploy to mainnet without:
1. Comprehensive testing
2. Static analysis (clean)
3. Professional audit (for value-holding contracts)
4. Bug bounty program
:::
