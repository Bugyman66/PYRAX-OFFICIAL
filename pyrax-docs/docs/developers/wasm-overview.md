# WASM/Rust Overview

PYRAX supports WebAssembly (WASM) smart contracts written in Rust, providing an alternative to Solidity for developers who prefer systems programming languages.

## Why WASM on PYRAX?

### Benefits

| Benefit | Description |
|---------|-------------|
| **Performance** | Near-native execution speed |
| **Memory Safety** | Rust's ownership model prevents common bugs |
| **Smaller Binaries** | Optimized WASM output |
| **Language Choice** | Use Rust instead of Solidity |
| **Tooling** | Leverage Rust's mature ecosystem |

### Use Cases

- **High-performance contracts** — Computationally intensive operations
- **Complex logic** — Easier to express in Rust
- **Existing Rust codebases** — Port existing libraries
- **AI/ML integration** — Native integration with Crucible

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    PYRAX Virtual Machine                     │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────────┐      ┌──────────────────┐            │
│  │       EVM        │      │    WASM Runtime   │            │
│  │   (Solidity)     │      │     (Rust)        │            │
│  └────────┬─────────┘      └────────┬─────────┘            │
│           │                         │                       │
│           └─────────┬───────────────┘                       │
│                     │                                        │
│           ┌─────────▼─────────┐                             │
│           │   State Storage   │                             │
│           └───────────────────┘                             │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### WASM Runtime

- **Wasmer-based** — High-performance WASM execution
- **Sandboxed** — Isolated execution environment
- **Metered** — Gas metering for resource limits
- **Interoperable** — Can call EVM contracts

## Comparison: Solidity vs Rust

| Aspect | Solidity (EVM) | Rust (WASM) |
|--------|----------------|-------------|
| Learning curve | Lower for web devs | Steeper, but powerful |
| Execution speed | Good | Excellent |
| Memory safety | Manual | Compile-time guarantees |
| Tooling | Mature | Growing rapidly |
| Debugging | Limited | Full Rust toolchain |
| Binary size | Larger | Smaller (optimized) |
| Ecosystem | Large | Growing |

## Supported Features

### Core Features
- ✅ Contract deployment
- ✅ Function calls
- ✅ State storage
- ✅ Events/logging
- ✅ Cross-contract calls
- ✅ Native token transfers

### Advanced Features
- ✅ Custom types and structs
- ✅ Complex data structures
- ✅ Cryptographic operations
- ✅ EVM interoperability
- ✅ Upgradeable contracts

## Contract Lifecycle

```
┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐
│  Write  │───►│ Compile │───►│ Deploy  │───►│ Execute │
│  (Rust) │    │ (WASM)  │    │ (Chain) │    │ (Calls) │
└─────────┘    └─────────┘    └─────────┘    └─────────┘
```

1. **Write** — Develop contract in Rust
2. **Compile** — Build to optimized WASM
3. **Deploy** — Upload WASM bytecode to chain
4. **Execute** — Call contract functions

## Quick Example

```rust
use pyrax_sdk::prelude::*;

#[pyrax_contract]
pub struct Counter {
    value: u64,
}

#[pyrax_contract]
impl Counter {
    #[init]
    pub fn new() -> Self {
        Self { value: 0 }
    }

    pub fn increment(&mut self) {
        self.value += 1;
    }

    pub fn get(&self) -> u64 {
        self.value
    }
}
```

## Gas Costs

WASM operations have comparable gas costs to EVM:

| Operation | EVM Gas | WASM Gas |
|-----------|---------|----------|
| Storage write | 20,000 | 18,000 |
| Storage read | 2,100 | 1,800 |
| Computation | Variable | ~80% of EVM |
| Contract call | 2,600+ | 2,400+ |

## When to Use WASM

### Choose WASM When:
- You're experienced with Rust
- Performance is critical
- You need complex data structures
- Porting existing Rust code
- Building AI-integrated contracts

### Choose Solidity When:
- You need maximum ecosystem compatibility
- Using existing Solidity libraries
- Rapid prototyping
- Simpler contract logic

---

:::info Getting Started
Ready to build? Continue to:
- [Rust Setup](./rust-setup) — Set up your development environment
- [Contract Development](./rust-contract-development) — Write your first Rust contract
:::
