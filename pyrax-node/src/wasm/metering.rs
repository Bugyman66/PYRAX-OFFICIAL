//! Gas Metering for WASM Contracts
//!
//! Tracks and limits gas consumption during execution

use serde::{Deserialize, Serialize};

/// Gas configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasConfig {
    /// Base cost per WASM instruction
    pub instruction_base: u64,
    /// Memory grow cost per page
    pub memory_grow_per_page: u64,
    /// Memory copy cost per byte
    pub memory_copy_per_byte: u64,
    /// Call indirect cost
    pub call_indirect: u64,
    /// Table access cost
    pub table_access: u64,
    /// Global access cost
    pub global_access: u64,
    /// Load/store base cost
    pub load_store_base: u64,
    /// Control flow cost
    pub control_flow: u64,
    /// Comparison cost
    pub comparison: u64,
    /// Arithmetic cost
    pub arithmetic: u64,
    /// Bitwise operation cost
    pub bitwise: u64,
    /// Conversion cost
    pub conversion: u64,
}

impl Default for GasConfig {
    fn default() -> Self {
        Self {
            instruction_base: 1,
            memory_grow_per_page: 1000,
            memory_copy_per_byte: 1,
            call_indirect: 10,
            table_access: 5,
            global_access: 3,
            load_store_base: 3,
            control_flow: 2,
            comparison: 1,
            arithmetic: 1,
            bitwise: 1,
            conversion: 1,
        }
    }
}

/// Gas costs for specific operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GasCost {
    /// No cost (free operations)
    Free,
    /// Fixed cost
    Fixed(u64),
    /// Linear cost (base + per_unit * units)
    Linear { base: u64, per_unit: u64 },
    /// Quadratic cost (for memory-intensive ops)
    Quadratic { base: u64, per_unit: u64, per_unit_squared: u64 },
}

impl GasCost {
    /// Calculate total cost
    pub fn calculate(&self, units: u64) -> u64 {
        match self {
            GasCost::Free => 0,
            GasCost::Fixed(cost) => *cost,
            GasCost::Linear { base, per_unit } => base + per_unit * units,
            GasCost::Quadratic { base, per_unit, per_unit_squared } => {
                base + per_unit * units + per_unit_squared * units * units
            }
        }
    }
}

/// Gas meter for tracking consumption
#[derive(Debug, Clone)]
pub struct GasMeter {
    /// Gas limit
    limit: u64,
    /// Gas used
    used: u64,
    /// Gas refund (for storage cleanup)
    refund: u64,
    /// Configuration
    config: GasConfig,
    /// Is metering enabled
    enabled: bool,
}

impl GasMeter {
    /// Create new gas meter with limit
    pub fn new(limit: u64) -> Self {
        Self {
            limit,
            used: 0,
            refund: 0,
            config: GasConfig::default(),
            enabled: true,
        }
    }

    /// Create with custom config
    pub fn with_config(limit: u64, config: GasConfig) -> Self {
        Self {
            limit,
            used: 0,
            refund: 0,
            config,
            enabled: true,
        }
    }

    /// Create unlimited meter (for testing)
    pub fn unlimited() -> Self {
        Self {
            limit: u64::MAX,
            used: 0,
            refund: 0,
            config: GasConfig::default(),
            enabled: false,
        }
    }

    /// Consume gas, returns false if out of gas
    pub fn consume(&mut self, amount: u64) -> bool {
        if !self.enabled {
            return true;
        }

        if self.used + amount > self.limit {
            self.used = self.limit;
            false
        } else {
            self.used += amount;
            true
        }
    }

    /// Consume gas with cost calculation
    pub fn consume_cost(&mut self, cost: GasCost, units: u64) -> bool {
        self.consume(cost.calculate(units))
    }

    /// Add refund
    pub fn refund(&mut self, amount: u64) {
        self.refund += amount;
    }

    /// Get gas used
    pub fn gas_used(&self) -> u64 {
        self.used
    }

    /// Get gas remaining
    pub fn gas_remaining(&self) -> u64 {
        self.limit.saturating_sub(self.used)
    }

    /// Get gas limit
    pub fn gas_limit(&self) -> u64 {
        self.limit
    }

    /// Get total refund
    pub fn total_refund(&self) -> u64 {
        // Refund capped at 50% of gas used
        let max_refund = self.used / 2;
        self.refund.min(max_refund)
    }

    /// Get effective gas used (after refund)
    pub fn effective_gas_used(&self) -> u64 {
        self.used.saturating_sub(self.total_refund())
    }

    /// Check if out of gas
    pub fn is_out_of_gas(&self) -> bool {
        self.used >= self.limit
    }

    /// Reset meter
    pub fn reset(&mut self) {
        self.used = 0;
        self.refund = 0;
    }

    /// Set new limit
    pub fn set_limit(&mut self, limit: u64) {
        self.limit = limit;
    }

    /// Get config
    pub fn config(&self) -> &GasConfig {
        &self.config
    }
}

/// Gas schedule for different operations
#[derive(Debug, Clone)]
pub struct GasSchedule {
    /// WASM instruction costs
    pub wasm: WasmGasCosts,
    /// Host function costs
    pub host: HostGasCosts,
    /// EVM interop costs
    pub interop: InteropGasCosts,
}

impl Default for GasSchedule {
    fn default() -> Self {
        Self {
            wasm: WasmGasCosts::default(),
            host: HostGasCosts::default(),
            interop: InteropGasCosts::default(),
        }
    }
}

/// WASM instruction gas costs
#[derive(Debug, Clone)]
pub struct WasmGasCosts {
    pub regular: u64,
    pub div: u64,
    pub mul: u64,
    pub mem_load: u64,
    pub mem_store: u64,
    pub mem_grow: u64,
    pub i64_ops: u64,
    pub call: u64,
    pub call_indirect: u64,
    pub local_get: u64,
    pub local_set: u64,
    pub global_get: u64,
    pub global_set: u64,
    pub br: u64,
    pub br_if: u64,
    pub br_table_base: u64,
    pub br_table_per_entry: u64,
}

impl Default for WasmGasCosts {
    fn default() -> Self {
        Self {
            regular: 1,
            div: 16,
            mul: 4,
            mem_load: 3,
            mem_store: 3,
            mem_grow: 8000,
            i64_ops: 2,
            call: 5,
            call_indirect: 10,
            local_get: 1,
            local_set: 1,
            global_get: 2,
            global_set: 3,
            br: 2,
            br_if: 3,
            br_table_base: 5,
            br_table_per_entry: 1,
        }
    }
}

/// Host function gas costs
#[derive(Debug, Clone)]
pub struct HostGasCosts {
    pub storage_read_base: u64,
    pub storage_read_per_byte: u64,
    pub storage_write_base: u64,
    pub storage_write_per_byte: u64,
    pub storage_delete: u64,
    pub storage_delete_refund: u64,
    pub log_base: u64,
    pub log_per_topic: u64,
    pub log_per_byte: u64,
    pub blake3_base: u64,
    pub blake3_per_word: u64,
    pub keccak256_base: u64,
    pub keccak256_per_word: u64,
    pub ecrecover: u64,
    pub get_caller: u64,
    pub get_self: u64,
    pub get_value: u64,
    pub get_block_height: u64,
    pub get_block_timestamp: u64,
    pub balance: u64,
    pub transfer: u64,
}

impl Default for HostGasCosts {
    fn default() -> Self {
        Self {
            storage_read_base: 200,
            storage_read_per_byte: 3,
            storage_write_base: 5000,
            storage_write_per_byte: 10,
            storage_delete: 5000,
            storage_delete_refund: 15000,
            log_base: 375,
            log_per_topic: 375,
            log_per_byte: 8,
            blake3_base: 30,
            blake3_per_word: 6,
            keccak256_base: 30,
            keccak256_per_word: 6,
            ecrecover: 3000,
            get_caller: 20,
            get_self: 20,
            get_value: 20,
            get_block_height: 20,
            get_block_timestamp: 20,
            balance: 700,
            transfer: 9000,
        }
    }
}

/// EVM ↔ WASM interop costs
#[derive(Debug, Clone)]
pub struct InteropGasCosts {
    pub call_evm_base: u64,
    pub call_evm_per_byte: u64,
    pub call_wasm_base: u64,
    pub call_wasm_per_byte: u64,
    pub abi_encode_per_byte: u64,
    pub abi_decode_per_byte: u64,
}

impl Default for InteropGasCosts {
    fn default() -> Self {
        Self {
            call_evm_base: 1000,
            call_evm_per_byte: 5,
            call_wasm_base: 500,
            call_wasm_per_byte: 3,
            abi_encode_per_byte: 2,
            abi_decode_per_byte: 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gas_cost_calculation() {
        assert_eq!(GasCost::Free.calculate(100), 0);
        assert_eq!(GasCost::Fixed(50).calculate(100), 50);
        assert_eq!(GasCost::Linear { base: 10, per_unit: 2 }.calculate(5), 20);
        assert_eq!(
            GasCost::Quadratic { base: 10, per_unit: 2, per_unit_squared: 1 }.calculate(3),
            10 + 6 + 9
        );
    }

    #[test]
    fn test_gas_meter() {
        let mut meter = GasMeter::new(1000);
        
        assert!(meter.consume(500));
        assert_eq!(meter.gas_used(), 500);
        assert_eq!(meter.gas_remaining(), 500);
        
        assert!(meter.consume(400));
        assert_eq!(meter.gas_used(), 900);
        
        assert!(!meter.consume(200)); // Out of gas
        assert!(meter.is_out_of_gas());
    }

    #[test]
    fn test_gas_refund() {
        let mut meter = GasMeter::new(1000);
        
        meter.consume(800);
        meter.refund(500);
        
        // Refund capped at 50% of used (400)
        assert_eq!(meter.total_refund(), 400);
        assert_eq!(meter.effective_gas_used(), 400);
    }

    #[test]
    fn test_unlimited_meter() {
        let mut meter = GasMeter::unlimited();
        
        assert!(meter.consume(u64::MAX / 2));
        assert!(!meter.is_out_of_gas());
    }

    #[test]
    fn test_gas_schedule() {
        let schedule = GasSchedule::default();
        
        assert!(schedule.wasm.div > schedule.wasm.regular);
        assert!(schedule.host.storage_write_base > schedule.host.storage_read_base);
    }
}
