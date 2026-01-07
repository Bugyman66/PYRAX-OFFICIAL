# WASM/Rust Best Practices

Best practices and optimization techniques for WASM smart contracts on PYRAX.

## Code Organization

### Modular Structure

```
src/
├── lib.rs          # Contract entry point
├── state.rs        # State definitions
├── msg.rs          # Message types
├── error.rs        # Error types
├── execute.rs      # State-changing logic
├── query.rs        # Read-only logic
└── helpers.rs      # Utility functions
```

### Separation of Concerns

```rust
// state.rs
#[derive(Serialize, Deserialize)]
pub struct State {
    pub owner: Address,
    pub config: Config,
}

// msg.rs
#[derive(Serialize, Deserialize)]
pub enum ExecuteMsg {
    UpdateConfig { config: Config },
    Transfer { to: Address, amount: u128 },
}

#[derive(Serialize, Deserialize)]
pub enum QueryMsg {
    GetConfig {},
    GetBalance { address: Address },
}

// lib.rs
#[pyrax_contract]
impl Contract {
    pub fn execute(&mut self, msg: ExecuteMsg) -> Result<(), Error> {
        match msg {
            ExecuteMsg::UpdateConfig { config } => self.update_config(config),
            ExecuteMsg::Transfer { to, amount } => self.transfer(to, amount),
        }
    }

    #[view]
    pub fn query(&self, msg: QueryMsg) -> QueryResponse {
        match msg {
            QueryMsg::GetConfig {} => self.get_config(),
            QueryMsg::GetBalance { address } => self.get_balance(address),
        }
    }
}
```

## Gas Optimization

### Storage Efficiency

```rust
// ❌ Inefficient - multiple storage reads
pub fn get_total(&self) -> u128 {
    let a = self.value_a;  // Storage read
    let b = self.value_b;  // Storage read
    let c = self.value_c;  // Storage read
    a + b + c
}

// ✅ Efficient - batch into struct
#[derive(Serialize, Deserialize)]
pub struct Values {
    a: u128,
    b: u128,
    c: u128,
}

pub fn get_total(&self) -> u128 {
    let values = self.values;  // Single storage read
    values.a + values.b + values.c
}
```

### Minimize Storage Writes

```rust
// ❌ Multiple writes
pub fn update(&mut self, a: u128, b: u128) {
    self.value_a = a;  // Storage write
    self.value_b = b;  // Storage write
}

// ✅ Batch writes
pub fn update(&mut self, a: u128, b: u128) {
    self.values = Values { a, b };  // Single write
}
```

### Use References

```rust
// ❌ Cloning data
pub fn process(&self, data: Vec<u8>) {
    let copy = data.clone();  // Expensive
}

// ✅ Use references
pub fn process(&self, data: &[u8]) {
    // Work with reference
}
```

### Efficient Data Structures

```rust
// For frequent lookups
use Map<Address, u128>;  // O(1) lookup

// For ordered iteration
use BTreeMap<Address, u128>;  // O(log n) lookup, ordered

// For small collections (< 10 items)
use Vec<(Address, u128)>;  // Cheaper for small sets
```

## Security Best Practices

### Input Validation

```rust
pub fn transfer(&mut self, to: Address, amount: u128) -> Result<(), Error> {
    // Validate inputs
    require!(to != Address::zero(), "Invalid recipient");
    require!(amount > 0, "Amount must be positive");
    require!(amount <= MAX_TRANSFER, "Amount too large");
    
    // Continue with logic...
    Ok(())
}
```

### Access Control

```rust
// Define roles
const ADMIN_ROLE: u8 = 1;
const OPERATOR_ROLE: u8 = 2;

#[pyrax_contract]
pub struct AccessControl {
    roles: Map<Address, u8>,
}

impl AccessControl {
    fn has_role(&self, account: Address, role: u8) -> bool {
        self.roles.get(&account).map(|r| r & role != 0).unwrap_or(false)
    }

    fn only_role(&self, role: u8) {
        require!(
            self.has_role(env::caller(), role),
            "Missing required role"
        );
    }

    pub fn admin_function(&mut self) {
        self.only_role(ADMIN_ROLE);
        // Admin logic
    }
}
```

### Reentrancy Protection

```rust
#[pyrax_contract]
pub struct ReentrancyGuard {
    locked: bool,
}

impl ReentrancyGuard {
    fn lock(&mut self) {
        require!(!self.locked, "Reentrant call");
        self.locked = true;
    }

    fn unlock(&mut self) {
        self.locked = false;
    }

    pub fn withdraw(&mut self, amount: u128) {
        self.lock();
        
        // Update state BEFORE external call
        self.balances.insert(env::caller(), 0);
        
        // External call
        env::transfer(env::caller(), amount);
        
        self.unlock();
    }
}
```

### Integer Overflow Protection

```rust
// Use checked arithmetic
pub fn add_balance(&mut self, amount: u128) -> Result<(), Error> {
    let current = self.balance;
    let new_balance = current.checked_add(amount)
        .ok_or(Error::Overflow)?;
    self.balance = new_balance;
    Ok(())
}

// Or use saturating arithmetic
pub fn add_balance_safe(&mut self, amount: u128) {
    self.balance = self.balance.saturating_add(amount);
}
```

## Binary Size Optimization

### Cargo.toml Settings

```toml
[profile.release]
opt-level = "z"        # Optimize for size
lto = true             # Link-time optimization
codegen-units = 1      # Single codegen unit
panic = "abort"        # No panic unwinding
strip = true           # Strip symbols
```

### Reduce Dependencies

```rust
// ❌ Heavy dependencies
use serde_json;  // Large

// ✅ Lightweight alternatives
use miniserde;   // Smaller
```

### Feature Flags

```toml
[dependencies]
serde = { version = "1.0", default-features = false, features = ["derive"] }
```

### Remove Debug Code

```rust
// Only include in debug builds
#[cfg(debug_assertions)]
fn debug_log(msg: &str) {
    env::log(msg);
}

#[cfg(not(debug_assertions))]
fn debug_log(_msg: &str) {}
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialization() {
        let contract = MyContract::new(100);
        assert_eq!(contract.get_value(), 100);
    }

    #[test]
    #[should_panic(expected = "Not authorized")]
    fn test_unauthorized_access() {
        let mut contract = MyContract::new(100);
        contract.admin_function(); // Should panic
    }
}
```

### Property-Based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_transfer_preserves_total(
        amount in 0u128..1000000,
        initial in 1000000u128..2000000
    ) {
        let mut contract = Token::new(initial);
        let initial_supply = contract.total_supply();
        
        contract.transfer(other_address(), amount);
        
        assert_eq!(contract.total_supply(), initial_supply);
    }
}
```

### Integration Tests

```rust
#[test]
fn test_full_workflow() {
    let mut ctx = TestContext::new();
    
    // Deploy
    let mut contract = MyContract::new(100);
    
    // Simulate multiple users
    ctx.set_caller(user_1());
    contract.deposit(1000);
    
    ctx.set_caller(user_2());
    contract.deposit(500);
    
    // Verify state
    assert_eq!(contract.total_deposits(), 1500);
}
```

## Documentation

### Inline Documentation

```rust
/// A simple counter contract.
/// 
/// # Examples
/// 
/// ```rust
/// let mut counter = Counter::new();
/// counter.increment();
/// assert_eq!(counter.get(), 1);
/// ```
#[pyrax_contract]
pub struct Counter {
    /// The current counter value
    value: u64,
}

#[pyrax_contract]
impl Counter {
    /// Creates a new counter initialized to zero.
    #[init]
    pub fn new() -> Self {
        Self { value: 0 }
    }

    /// Increments the counter by one.
    /// 
    /// # Panics
    /// 
    /// Panics if the counter would overflow.
    pub fn increment(&mut self) {
        self.value = self.value.checked_add(1)
            .expect("Counter overflow");
    }
}
```

---

:::tip Summary
1. **Organize** code into modules
2. **Optimize** storage access patterns
3. **Validate** all inputs
4. **Protect** against reentrancy
5. **Minimize** binary size
6. **Test** thoroughly
7. **Document** everything
:::
