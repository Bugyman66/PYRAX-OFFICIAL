# Rust Contract Development

This guide covers writing smart contracts in Rust for PYRAX.

## Contract Structure

### Basic Contract

```rust
use pyrax_sdk::prelude::*;

/// Contract state stored on-chain
#[pyrax_contract]
pub struct MyContract {
    owner: Address,
    value: u64,
    data: Map<Address, u64>,
}

/// Contract implementation
#[pyrax_contract]
impl MyContract {
    /// Initialize new contract
    #[init]
    pub fn new(initial_value: u64) -> Self {
        Self {
            owner: env::caller(),
            value: initial_value,
            data: Map::new(),
        }
    }

    /// Public function - modifies state
    pub fn set_value(&mut self, new_value: u64) {
        self.value = new_value;
        env::emit_event(ValueChanged { 
            old: self.value, 
            new: new_value 
        });
    }

    /// View function - read only
    #[view]
    pub fn get_value(&self) -> u64 {
        self.value
    }

    /// Restricted function
    pub fn admin_function(&mut self) {
        require!(env::caller() == self.owner, "Not authorized");
        // Admin logic here
    }
}
```

## Core Concepts

### State Management

```rust
#[pyrax_contract]
pub struct State {
    // Simple types
    counter: u64,
    flag: bool,
    name: String,
    
    // Collections
    balances: Map<Address, u128>,
    allowances: Map<(Address, Address), u128>,
    items: Vec<Item>,
    
    // Custom types
    config: Config,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    admin: Address,
    fee_rate: u64,
    paused: bool,
}
```

### Environment Access

```rust
// Caller information
let caller = env::caller();           // Transaction sender
let origin = env::origin();           // Original sender

// Block information
let block_number = env::block_number();
let block_timestamp = env::block_timestamp();
let block_hash = env::block_hash(block_number - 1);

// Contract information
let self_address = env::self_address();
let balance = env::balance();

// Transaction information
let value = env::attached_value();    // PYRAX sent with call
let gas_left = env::gas_left();
```

### Events

```rust
// Define event
#[derive(Event)]
pub struct Transfer {
    #[indexed]
    from: Address,
    #[indexed]
    to: Address,
    amount: u128,
}

// Emit event
env::emit_event(Transfer {
    from: sender,
    to: recipient,
    amount: 1000,
});
```

### Errors

```rust
// Define errors
#[derive(Error, Debug)]
pub enum ContractError {
    #[error("Unauthorized access")]
    Unauthorized,
    
    #[error("Insufficient balance: have {have}, need {need}")]
    InsufficientBalance { have: u128, need: u128 },
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

// Use errors
pub fn transfer(&mut self, to: Address, amount: u128) -> Result<(), ContractError> {
    let balance = self.balances.get(&env::caller()).unwrap_or(0);
    
    if balance < amount {
        return Err(ContractError::InsufficientBalance {
            have: balance,
            need: amount,
        });
    }
    
    // Transfer logic...
    Ok(())
}
```

## Common Patterns

### Access Control

```rust
#[pyrax_contract]
pub struct Ownable {
    owner: Address,
}

#[pyrax_contract]
impl Ownable {
    fn only_owner(&self) {
        require!(
            env::caller() == self.owner,
            "Caller is not the owner"
        );
    }

    pub fn transfer_ownership(&mut self, new_owner: Address) {
        self.only_owner();
        self.owner = new_owner;
    }
}
```

### Pausable

```rust
#[pyrax_contract]
pub struct Pausable {
    paused: bool,
    owner: Address,
}

#[pyrax_contract]
impl Pausable {
    fn when_not_paused(&self) {
        require!(!self.paused, "Contract is paused");
    }

    pub fn pause(&mut self) {
        require!(env::caller() == self.owner, "Not owner");
        self.paused = true;
    }

    pub fn unpause(&mut self) {
        require!(env::caller() == self.owner, "Not owner");
        self.paused = false;
    }

    pub fn do_something(&mut self) {
        self.when_not_paused();
        // Logic here
    }
}
```

### Token Contract

```rust
#[pyrax_contract]
pub struct Token {
    name: String,
    symbol: String,
    decimals: u8,
    total_supply: u128,
    balances: Map<Address, u128>,
    allowances: Map<(Address, Address), u128>,
}

#[pyrax_contract]
impl Token {
    #[init]
    pub fn new(name: String, symbol: String, initial_supply: u128) -> Self {
        let mut balances = Map::new();
        balances.insert(env::caller(), initial_supply);
        
        Self {
            name,
            symbol,
            decimals: 18,
            total_supply: initial_supply,
            balances,
            allowances: Map::new(),
        }
    }

    #[view]
    pub fn balance_of(&self, account: Address) -> u128 {
        self.balances.get(&account).unwrap_or(0)
    }

    pub fn transfer(&mut self, to: Address, amount: u128) -> bool {
        let from = env::caller();
        self.transfer_internal(from, to, amount)
    }

    pub fn approve(&mut self, spender: Address, amount: u128) -> bool {
        let owner = env::caller();
        self.allowances.insert((owner, spender), amount);
        env::emit_event(Approval { owner, spender, amount });
        true
    }

    pub fn transfer_from(&mut self, from: Address, to: Address, amount: u128) -> bool {
        let spender = env::caller();
        let allowance = self.allowances.get(&(from, spender)).unwrap_or(0);
        
        require!(allowance >= amount, "Insufficient allowance");
        
        self.allowances.insert((from, spender), allowance - amount);
        self.transfer_internal(from, to, amount)
    }

    fn transfer_internal(&mut self, from: Address, to: Address, amount: u128) -> bool {
        let from_balance = self.balances.get(&from).unwrap_or(0);
        require!(from_balance >= amount, "Insufficient balance");
        
        self.balances.insert(from, from_balance - amount);
        let to_balance = self.balances.get(&to).unwrap_or(0);
        self.balances.insert(to, to_balance + amount);
        
        env::emit_event(Transfer { from, to, amount });
        true
    }
}
```

## Cross-Contract Calls

```rust
// Define interface
#[interface]
pub trait IToken {
    fn transfer(&mut self, to: Address, amount: u128) -> bool;
    fn balance_of(&self, account: Address) -> u128;
}

// Use interface
pub fn withdraw_tokens(&mut self, token: Address, amount: u128) {
    let token_contract = IToken::at(token);
    
    // Call external contract
    let success = token_contract.transfer(env::caller(), amount);
    require!(success, "Transfer failed");
}
```

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use pyrax_sdk::testing::*;

    #[test]
    fn test_counter() {
        // Set up test environment
        let mut ctx = TestContext::new();
        ctx.set_caller(address!("0x1234..."));
        
        // Deploy contract
        let mut counter = Counter::new();
        
        // Test initial state
        assert_eq!(counter.get(), 0);
        
        // Test increment
        counter.increment();
        assert_eq!(counter.get(), 1);
    }

    #[test]
    fn test_access_control() {
        let mut ctx = TestContext::new();
        let owner = address!("0x1111...");
        let other = address!("0x2222...");
        
        ctx.set_caller(owner);
        let mut contract = MyContract::new(100);
        
        // Owner can call
        contract.admin_function(); // Should succeed
        
        // Non-owner cannot
        ctx.set_caller(other);
        // This should panic
        // contract.admin_function();
    }
}
```

---

:::tip Next Steps
- [Contract Deployment](./rust-contract-deployment) — Deploy your contract
- [Best Practices](./wasm-best-practices) — Optimization tips
:::
