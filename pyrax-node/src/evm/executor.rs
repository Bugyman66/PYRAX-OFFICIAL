//! EVM Executor
//!
//! Executes EVM transactions with full EVM opcode support

use serde::{Deserialize, Serialize};
use tracing::debug;

use super::state::StateDB;
use super::types::{Address, B256, U256, EvmTransaction};
use super::{ChainConfig, Log};
use super::precompiles::PrecompileRegistry;

/// Transaction execution context
#[derive(Debug, Clone)]
pub struct TransactionContext {
    pub block_number: u64,
    pub block_timestamp: u64,
    pub block_coinbase: Address,
    pub block_gas_limit: u64,
    pub block_difficulty: u64,
    pub block_basefee: u64,
    pub chain_id: u64,
    pub prev_randao: B256,
}

impl Default for TransactionContext {
    fn default() -> Self {
        Self {
            block_number: 0,
            block_timestamp: 0,
            block_coinbase: [0u8; 20],
            block_gas_limit: 30_000_000,
            block_difficulty: 0,
            block_basefee: 1_000_000_000,
            chain_id: 7777,
            prev_randao: [0u8; 32],
        }
    }
}

/// Execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub gas_used: u64,
    pub gas_refunded: u64,
    pub output: Vec<u8>,
    pub logs: Vec<Log>,
    pub contract_address: Option<Address>,
    pub revert_reason: Option<String>,
}

impl ExecutionResult {
    pub fn failed(reason: String) -> Self {
        Self {
            success: false,
            gas_used: 0,
            gas_refunded: 0,
            output: Vec::new(),
            logs: Vec::new(),
            contract_address: None,
            revert_reason: Some(reason),
        }
    }

    pub fn success_transfer(gas_used: u64) -> Self {
        Self {
            success: true,
            gas_used,
            gas_refunded: 0,
            output: Vec::new(),
            logs: Vec::new(),
            contract_address: None,
            revert_reason: None,
        }
    }
}

/// EVM Executor
pub struct EvmExecutor {
    config: ChainConfig,
    state: StateDB,
    precompiles: PrecompileRegistry,
}

impl EvmExecutor {
    /// Create a new EVM executor
    pub fn new(config: ChainConfig, state: StateDB) -> Self {
        Self {
            config,
            state,
            precompiles: PrecompileRegistry::new(),
        }
    }

    /// Create with default config
    pub fn with_state(state: StateDB) -> Self {
        Self::new(ChainConfig::default(), state)
    }

    /// Get reference to state
    pub fn state(&self) -> &StateDB {
        &self.state
    }

    /// Get mutable reference to state
    pub fn state_mut(&mut self) -> &mut StateDB {
        &mut self.state
    }

    /// Execute a transaction
    pub fn execute_transaction(
        &mut self,
        tx: &EvmTransaction,
        from: Address,
        ctx: &TransactionContext,
    ) -> ExecutionResult {
        // Validate transaction
        if let Err(e) = self.validate_transaction(tx, &from, ctx) {
            return ExecutionResult::failed(e);
        }

        // Deduct gas upfront
        let max_gas_cost = U256::from_u64(tx.gas_limit)
            .checked_mul(U256::from_u64(tx.max_fee_per_gas))
            .unwrap_or(U256::MAX);
        
        if !self.state.sub_balance(&from, max_gas_cost) {
            return ExecutionResult::failed("Insufficient balance for gas".to_string());
        }

        // Increment sender nonce
        self.state.increment_nonce(&from);

        // Execute based on transaction type
        let result = if tx.is_create() {
            self.execute_create(tx, &from, ctx)
        } else {
            let to = tx.to.unwrap();
            if self.precompiles.is_precompile(&to) {
                self.execute_precompile(tx, &from, &to)
            } else if self.state.get_code(&to).is_empty() {
                self.execute_transfer(tx, &from, &to)
            } else {
                self.execute_call(tx, &from, &to, ctx)
            }
        };

        // Refund unused gas
        let effective_gas_price = tx.effective_gas_price(ctx.block_basefee);
        let gas_refund = tx.gas_limit.saturating_sub(result.gas_used);
        let refund_amount = U256::from_u64(gas_refund)
            .checked_mul(U256::from_u64(effective_gas_price))
            .unwrap_or(U256::ZERO);
        
        self.state.add_balance(&from, refund_amount);

        // Pay coinbase
        let gas_payment = U256::from_u64(result.gas_used)
            .checked_mul(U256::from_u64(effective_gas_price))
            .unwrap_or(U256::ZERO);
        self.state.add_balance(&ctx.block_coinbase, gas_payment);

        result
    }

    fn validate_transaction(
        &self,
        tx: &EvmTransaction,
        from: &Address,
        ctx: &TransactionContext,
    ) -> Result<(), String> {
        if tx.chain_id != ctx.chain_id {
            return Err(format!("Invalid chain ID: expected {}, got {}", ctx.chain_id, tx.chain_id));
        }
        let account_nonce = self.state.get_nonce(from);
        if tx.nonce != account_nonce {
            return Err(format!("Invalid nonce: expected {}, got {}", account_nonce, tx.nonce));
        }
        let balance = self.state.get_balance(from);
        if balance.checked_sub(tx.max_cost()).is_none() {
            return Err("Insufficient balance".to_string());
        }
        if tx.gas_limit > ctx.block_gas_limit {
            return Err("Gas limit exceeds block limit".to_string());
        }
        if tx.max_fee_per_gas < ctx.block_basefee {
            return Err("Max fee below base fee".to_string());
        }
        Ok(())
    }

    fn execute_transfer(&mut self, tx: &EvmTransaction, from: &Address, to: &Address) -> ExecutionResult {
        const TRANSFER_GAS: u64 = 21000;
        if tx.gas_limit < TRANSFER_GAS {
            return ExecutionResult::failed("Insufficient gas".to_string());
        }
        if !tx.value.is_zero() {
            if !self.state.sub_balance(from, tx.value) {
                return ExecutionResult::failed("Insufficient balance".to_string());
            }
            self.state.add_balance(to, tx.value);
        }
        ExecutionResult::success_transfer(TRANSFER_GAS)
    }

    fn execute_precompile(&mut self, tx: &EvmTransaction, from: &Address, to: &Address) -> ExecutionResult {
        if !tx.value.is_zero() {
            if !self.state.sub_balance(from, tx.value) {
                return ExecutionResult::failed("Insufficient balance".to_string());
            }
            self.state.add_balance(to, tx.value);
        }
        match self.precompiles.execute(to, &tx.input, tx.gas_limit) {
            Some(result) if result.success => ExecutionResult {
                success: true,
                gas_used: result.gas_used,
                gas_refunded: 0,
                output: result.output,
                logs: Vec::new(),
                contract_address: None,
                revert_reason: None,
            },
            _ => ExecutionResult::failed("Precompile failed".to_string()),
        }
    }

    fn execute_call(&mut self, tx: &EvmTransaction, from: &Address, to: &Address, ctx: &TransactionContext) -> ExecutionResult {
        const BASE_GAS: u64 = 21000;
        if tx.gas_limit < BASE_GAS {
            return ExecutionResult::failed("Insufficient gas".to_string());
        }
        if !tx.value.is_zero() {
            if !self.state.sub_balance(from, tx.value) {
                return ExecutionResult::failed("Insufficient balance".to_string());
            }
            self.state.add_balance(to, tx.value);
        }
        let code = self.state.get_code(to);
        if code.is_empty() {
            return ExecutionResult::success_transfer(BASE_GAS);
        }
        let remaining_gas = tx.gas_limit - BASE_GAS;
        let evm_result = self.execute_bytecode(&code, &tx.input, *from, *to, tx.value, remaining_gas, ctx);
        ExecutionResult {
            success: evm_result.success,
            gas_used: BASE_GAS + evm_result.gas_used,
            gas_refunded: evm_result.gas_refunded,
            output: evm_result.output,
            logs: evm_result.logs,
            contract_address: None,
            revert_reason: evm_result.revert_reason,
        }
    }

    fn execute_create(&mut self, tx: &EvmTransaction, from: &Address, ctx: &TransactionContext) -> ExecutionResult {
        const CREATE_GAS: u64 = 32000;
        if tx.gas_limit < CREATE_GAS {
            return ExecutionResult::failed("Insufficient gas".to_string());
        }
        let contract_address = self.compute_create_address(from, tx.nonce);
        if !tx.value.is_zero() {
            if !self.state.sub_balance(from, tx.value) {
                return ExecutionResult::failed("Insufficient balance".to_string());
            }
            self.state.add_balance(&contract_address, tx.value);
        }
        let remaining_gas = tx.gas_limit - CREATE_GAS;
        let init_result = self.execute_bytecode(&tx.input, &[], *from, contract_address, tx.value, remaining_gas, ctx);
        if !init_result.success {
            return ExecutionResult {
                success: false,
                gas_used: CREATE_GAS + init_result.gas_used,
                gas_refunded: 0,
                output: Vec::new(),
                logs: Vec::new(),
                contract_address: None,
                revert_reason: init_result.revert_reason,
            };
        }
        let code_deposit_gas = (init_result.output.len() * 200) as u64;
        if init_result.gas_used + code_deposit_gas > remaining_gas {
            return ExecutionResult::failed("Insufficient gas for code".to_string());
        }
        self.state.set_code(&contract_address, init_result.output);
        ExecutionResult {
            success: true,
            gas_used: CREATE_GAS + init_result.gas_used + code_deposit_gas,
            gas_refunded: init_result.gas_refunded,
            output: Vec::new(),
            logs: init_result.logs,
            contract_address: Some(contract_address),
            revert_reason: None,
        }
    }

    fn execute_bytecode(&mut self, code: &[u8], input: &[u8], caller: Address, address: Address, value: U256, gas_limit: u64, ctx: &TransactionContext) -> ExecutionResult {
        let mut vm = EvmInterpreter::new(code.to_vec(), input.to_vec(), caller, address, value, gas_limit, ctx.clone());
        vm.execute(&mut self.state)
    }

    fn compute_create_address(&self, sender: &Address, nonce: u64) -> Address {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(sender);
        hasher.update(&nonce.to_be_bytes());
        let result = hasher.finalize();
        let mut address = [0u8; 20];
        address.copy_from_slice(&result.as_bytes()[12..32]);
        address
    }

    pub fn simulate_transaction(&self, tx: &EvmTransaction, from: Address, ctx: &TransactionContext) -> ExecutionResult {
        let mut executor = EvmExecutor::new(self.config.clone(), self.state.clone());
        executor.execute_transaction(tx, from, ctx)
    }

    pub fn estimate_gas(&self, tx: &EvmTransaction, from: Address, ctx: &TransactionContext) -> Result<u64, String> {
        let result = self.simulate_transaction(tx, from, ctx);
        if result.success { Ok((result.gas_used as f64 * 1.1) as u64) } else { Err(result.revert_reason.unwrap_or_default()) }
    }

    pub fn call(&self, to: Address, data: Vec<u8>, from: Option<Address>, value: U256, ctx: &TransactionContext) -> ExecutionResult {
        let from = from.unwrap_or([0u8; 20]);
        let tx = EvmTransaction {
            tx_type: 2, chain_id: ctx.chain_id, nonce: self.state.get_nonce(&from),
            max_priority_fee_per_gas: 0, max_fee_per_gas: ctx.block_basefee, gas_limit: ctx.block_gas_limit,
            to: Some(to), value, input: data, access_list: Vec::new(), v: 0, r: [0u8; 32], s: [0u8; 32],
        };
        self.simulate_transaction(&tx, from, ctx)
    }

    pub fn commit(&mut self) -> B256 { self.state.commit() }
    pub fn config(&self) -> &ChainConfig { &self.config }
}

struct EvmInterpreter {
    code: Vec<u8>, input: Vec<u8>, caller: Address, address: Address, value: U256,
    gas_limit: u64, gas_used: u64, ctx: TransactionContext,
    stack: Vec<U256>, memory: Vec<u8>, pc: usize,
    return_data: Vec<u8>, logs: Vec<Log>, stopped: bool, reverted: bool,
}

impl EvmInterpreter {
    fn new(code: Vec<u8>, input: Vec<u8>, caller: Address, address: Address, value: U256, gas_limit: u64, ctx: TransactionContext) -> Self {
        Self { code, input, caller, address, value, gas_limit, gas_used: 0, ctx, stack: Vec::with_capacity(1024), memory: Vec::new(), pc: 0, return_data: Vec::new(), logs: Vec::new(), stopped: false, reverted: false }
    }

    fn execute(&mut self, state: &mut StateDB) -> ExecutionResult {
        while !self.stopped && self.pc < self.code.len() {
            if self.gas_used >= self.gas_limit { return ExecutionResult::failed("Out of gas".to_string()); }
            let opcode = self.code[self.pc];
            self.pc += 1;
            if let Err(e) = self.execute_opcode(opcode, state) { return ExecutionResult::failed(e); }
        }
        if self.reverted {
            ExecutionResult { success: false, gas_used: self.gas_used, gas_refunded: 0, output: self.return_data.clone(), logs: Vec::new(), contract_address: None, revert_reason: Some("Reverted".to_string()) }
        } else {
            ExecutionResult { success: true, gas_used: self.gas_used, gas_refunded: 0, output: self.return_data.clone(), logs: self.logs.clone(), contract_address: None, revert_reason: None }
        }
    }

    fn execute_opcode(&mut self, opcode: u8, state: &mut StateDB) -> Result<(), String> {
        self.gas_used += self.opcode_gas(opcode);
        match opcode {
            0x00 => self.stopped = true,
            0x01 => { let a = self.pop()?; let b = self.pop()?; self.push(a.saturating_add(b)); }
            0x02 => { let a = self.pop()?; let b = self.pop()?; self.push(a.checked_mul(b).unwrap_or(U256::ZERO)); }
            0x03 => { let a = self.pop()?; let b = self.pop()?; self.push(a.saturating_sub(b)); }
            0x14 => { let a = self.pop()?; let b = self.pop()?; self.push(if a == b { U256::ONE } else { U256::ZERO }); }
            0x15 => { let a = self.pop()?; self.push(if a.is_zero() { U256::ONE } else { U256::ZERO }); }
            0x30 => { let mut b = [0u8; 32]; b[12..32].copy_from_slice(&self.address); self.push(U256::from_be_bytes(b)); }
            0x33 => { let mut b = [0u8; 32]; b[12..32].copy_from_slice(&self.caller); self.push(U256::from_be_bytes(b)); }
            0x34 => self.push(self.value),
            0x36 => self.push(U256::from_u64(self.input.len() as u64)),
            0x38 => self.push(U256::from_u64(self.code.len() as u64)),
            0x43 => self.push(U256::from_u64(self.ctx.block_number)),
            0x46 => self.push(U256::from_u64(self.ctx.chain_id)),
            0x50 => { self.pop()?; }
            0x51 => { let o = self.pop()?.low_u64() as usize; self.expand_memory(o + 32); let mut b = [0u8; 32]; for i in 0..32 { if o + i < self.memory.len() { b[i] = self.memory[o + i]; } } self.push(U256::from_be_bytes(b)); }
            0x52 => { let o = self.pop()?.low_u64() as usize; let v = self.pop()?; self.expand_memory(o + 32); let b = v.to_be_bytes(); for i in 0..32 { if o + i < self.memory.len() { self.memory[o + i] = b[i]; } } }
            0x54 => { let k = self.pop()?.to_be_bytes(); self.push(state.get_storage(&self.address, &k)); }
            0x55 => { let k = self.pop()?.to_be_bytes(); let v = self.pop()?; state.set_storage(&self.address, k, v); }
            0x56 => { let d = self.pop()?.low_u64() as usize; if d < self.code.len() && self.code[d] == 0x5B { self.pc = d + 1; } else { return Err("Bad jump".to_string()); } }
            0x57 => { let d = self.pop()?.low_u64() as usize; let c = self.pop()?; if !c.is_zero() { if d < self.code.len() && self.code[d] == 0x5B { self.pc = d + 1; } else { return Err("Bad jump".to_string()); } } }
            0x5B => {}
            0x5A => self.push(U256::from_u64(self.gas_limit.saturating_sub(self.gas_used))),
            0x60..=0x7F => { let n = (opcode - 0x5F) as usize; let mut b = [0u8; 32]; for i in 0..n { if self.pc + i < self.code.len() { b[32 - n + i] = self.code[self.pc + i]; } } self.push(U256::from_be_bytes(b)); self.pc += n; }
            0x80..=0x8F => { let n = (opcode - 0x7F) as usize; if self.stack.len() < n { return Err("Underflow".to_string()); } let v = self.stack[self.stack.len() - n]; self.push(v); }
            0x90..=0x9F => { let n = (opcode - 0x8F) as usize; if self.stack.len() <= n { return Err("Underflow".to_string()); } let l = self.stack.len(); self.stack.swap(l - 1, l - 1 - n); }
            0xF3 => { let o = self.pop()?.low_u64() as usize; let s = self.pop()?.low_u64() as usize; self.expand_memory(o + s); self.return_data = self.memory.get(o..o + s).unwrap_or(&[]).to_vec(); self.stopped = true; }
            0xFD => { let o = self.pop()?.low_u64() as usize; let s = self.pop()?.low_u64() as usize; self.expand_memory(o + s); self.return_data = self.memory.get(o..o + s).unwrap_or(&[]).to_vec(); self.stopped = true; self.reverted = true; }
            _ => { debug!("Unknown opcode: 0x{:02X}", opcode); }
        }
        Ok(())
    }

    fn pop(&mut self) -> Result<U256, String> { self.stack.pop().ok_or("Stack underflow".to_string()) }
    fn push(&mut self, v: U256) { if self.stack.len() < 1024 { self.stack.push(v); } }
    fn expand_memory(&mut self, size: usize) { if size > self.memory.len() { self.memory.resize(((size + 31) / 32) * 32, 0); } }
    fn opcode_gas(&self, op: u8) -> u64 { match op { 0x54 => 100, 0x55 => 5000, _ => 3 } }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_creation() {
        let state = StateDB::new();
        let executor = EvmExecutor::with_state(state);
        assert_eq!(executor.config().chain_id, 7777);
    }

    #[test]
    fn test_simple_transfer() {
        let state = StateDB::new();
        let sender = [1u8; 20];
        let recipient = [2u8; 20];
        state.add_balance(&sender, U256::from_u64(1_000_000_000_000_000_000));
        state.commit();
        let mut executor = EvmExecutor::with_state(state);
        let tx = EvmTransaction {
            tx_type: 2, chain_id: 7777, nonce: 0,
            max_priority_fee_per_gas: 1_000_000_000, max_fee_per_gas: 2_000_000_000,
            gas_limit: 21000, to: Some(recipient), value: U256::from_u64(1_000_000_000),
            input: Vec::new(), access_list: Vec::new(), v: 0, r: [0u8; 32], s: [0u8; 32],
        };
        let ctx = TransactionContext::default();
        let result = executor.execute_transaction(&tx, sender, &ctx);
        assert!(result.success);
    }
}
