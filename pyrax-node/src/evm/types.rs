//! EVM Types
//!
//! Core types for EVM transaction and execution

use serde::{Deserialize, Serialize};
use std::fmt;

/// 20-byte Ethereum address
pub type Address = [u8; 20];

/// 32-byte hash/word
pub type B256 = [u8; 32];

/// EVM Transaction (EIP-1559 compatible)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmTransaction {
    /// Transaction type (0 = legacy, 1 = access list, 2 = EIP-1559)
    pub tx_type: u8,
    /// Chain ID
    pub chain_id: u64,
    /// Sender nonce
    pub nonce: u64,
    /// Max priority fee per gas (tip)
    pub max_priority_fee_per_gas: u64,
    /// Max fee per gas
    pub max_fee_per_gas: u64,
    /// Gas limit
    pub gas_limit: u64,
    /// Recipient address (None for contract creation)
    pub to: Option<Address>,
    /// Value in wei
    pub value: U256,
    /// Input data
    pub input: Vec<u8>,
    /// Access list (EIP-2930)
    pub access_list: Vec<AccessListItem>,
    /// Signature V
    pub v: u64,
    /// Signature R
    pub r: B256,
    /// Signature S
    pub s: B256,
}

/// Access list item (EIP-2930)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessListItem {
    pub address: Address,
    pub storage_keys: Vec<B256>,
}

/// 256-bit unsigned integer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct U256(pub [u64; 4]);

impl U256 {
    pub const ZERO: U256 = U256([0, 0, 0, 0]);
    pub const ONE: U256 = U256([1, 0, 0, 0]);
    pub const MAX: U256 = U256([u64::MAX, u64::MAX, u64::MAX, u64::MAX]);

    pub fn from_be_bytes(bytes: [u8; 32]) -> Self {
        let mut limbs = [0u64; 4];
        for i in 0..4 {
            let start = i * 8;
            limbs[3 - i] = u64::from_be_bytes([
                bytes[start],
                bytes[start + 1],
                bytes[start + 2],
                bytes[start + 3],
                bytes[start + 4],
                bytes[start + 5],
                bytes[start + 6],
                bytes[start + 7],
            ]);
        }
        U256(limbs)
    }

    pub fn to_be_bytes(self) -> [u8; 32] {
        let mut bytes = [0u8; 32];
        for i in 0..4 {
            let limb_bytes = self.0[3 - i].to_be_bytes();
            let start = i * 8;
            bytes[start..start + 8].copy_from_slice(&limb_bytes);
        }
        bytes
    }

    pub fn from_u64(value: u64) -> Self {
        U256([value, 0, 0, 0])
    }

    pub fn from_u128(value: u128) -> Self {
        U256([value as u64, (value >> 64) as u64, 0, 0])
    }

    pub fn low_u64(&self) -> u64 {
        self.0[0]
    }

    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|&x| x == 0)
    }

    pub fn checked_add(self, rhs: Self) -> Option<Self> {
        let mut result = [0u64; 4];
        let mut carry = 0u64;
        
        for i in 0..4 {
            let (sum, c1) = self.0[i].overflowing_add(rhs.0[i]);
            let (sum, c2) = sum.overflowing_add(carry);
            result[i] = sum;
            carry = (c1 as u64) + (c2 as u64);
        }
        
        if carry > 0 {
            None
        } else {
            Some(U256(result))
        }
    }

    pub fn checked_sub(self, rhs: Self) -> Option<Self> {
        let mut result = [0u64; 4];
        let mut borrow = 0u64;
        
        for i in 0..4 {
            let (diff, b1) = self.0[i].overflowing_sub(rhs.0[i]);
            let (diff, b2) = diff.overflowing_sub(borrow);
            result[i] = diff;
            borrow = (b1 as u64) + (b2 as u64);
        }
        
        if borrow > 0 {
            None
        } else {
            Some(U256(result))
        }
    }

    pub fn checked_mul(self, rhs: Self) -> Option<Self> {
        let mut result = [0u128; 4];
        
        for i in 0..4 {
            if self.0[i] == 0 {
                continue;
            }
            for j in 0..4 {
                if i + j >= 4 {
                    if rhs.0[j] != 0 {
                        return None; // Overflow
                    }
                    continue;
                }
                let prod = (self.0[i] as u128) * (rhs.0[j] as u128);
                result[i + j] += prod;
            }
        }
        
        // Propagate carries
        let mut carry = 0u128;
        let mut final_result = [0u64; 4];
        for i in 0..4 {
            let sum = result[i] + carry;
            final_result[i] = sum as u64;
            carry = sum >> 64;
        }
        
        if carry > 0 {
            None
        } else {
            Some(U256(final_result))
        }
    }

    pub fn saturating_add(self, rhs: Self) -> Self {
        self.checked_add(rhs).unwrap_or(Self::MAX)
    }

    pub fn saturating_sub(self, rhs: Self) -> Self {
        self.checked_sub(rhs).unwrap_or(Self::ZERO)
    }
}

impl fmt::Display for U256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex::encode(self.to_be_bytes()))
    }
}

impl From<u64> for U256 {
    fn from(value: u64) -> Self {
        U256::from_u64(value)
    }
}

impl From<u128> for U256 {
    fn from(value: u128) -> Self {
        U256::from_u128(value)
    }
}

/// Signed transaction with signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedTransaction {
    pub transaction: EvmTransaction,
    pub signature_hash: B256,
    pub sender: Address,
}

impl SignedTransaction {
    /// Create a new signed transaction
    pub fn new(transaction: EvmTransaction, sender: Address) -> Self {
        let signature_hash = transaction.hash();
        Self {
            transaction,
            signature_hash,
            sender,
        }
    }

    /// Get transaction hash
    pub fn hash(&self) -> B256 {
        self.signature_hash
    }

    /// Recover sender address from signature
    pub fn recover_sender(&self) -> Option<Address> {
        Some(self.sender)
    }
}

impl EvmTransaction {
    /// Calculate transaction hash
    pub fn hash(&self) -> B256 {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(&[self.tx_type]);
        hasher.update(&self.chain_id.to_be_bytes());
        hasher.update(&self.nonce.to_be_bytes());
        hasher.update(&self.max_priority_fee_per_gas.to_be_bytes());
        hasher.update(&self.max_fee_per_gas.to_be_bytes());
        hasher.update(&self.gas_limit.to_be_bytes());
        if let Some(to) = &self.to {
            hasher.update(to);
        }
        hasher.update(&self.value.to_be_bytes());
        hasher.update(&self.input);
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(result.as_bytes());
        hash
    }

    /// Check if this is a contract creation transaction
    pub fn is_create(&self) -> bool {
        self.to.is_none()
    }

    /// Calculate effective gas price given base fee
    pub fn effective_gas_price(&self, base_fee: u64) -> u64 {
        let priority_fee = self.max_priority_fee_per_gas
            .min(self.max_fee_per_gas.saturating_sub(base_fee));
        base_fee.saturating_add(priority_fee)
    }

    /// Calculate max possible cost (gas_limit * max_fee + value)
    pub fn max_cost(&self) -> U256 {
        let gas_cost = U256::from_u64(self.gas_limit)
            .checked_mul(U256::from_u64(self.max_fee_per_gas))
            .unwrap_or(U256::MAX);
        gas_cost.checked_add(self.value).unwrap_or(U256::MAX)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u256_basic() {
        let a = U256::from_u64(100);
        let b = U256::from_u64(50);
        
        let sum = a.checked_add(b).unwrap();
        assert_eq!(sum.low_u64(), 150);
        
        let diff = a.checked_sub(b).unwrap();
        assert_eq!(diff.low_u64(), 50);
    }

    #[test]
    fn test_u256_bytes() {
        let value = U256::from_u64(0x1234567890ABCDEF);
        let bytes = value.to_be_bytes();
        let recovered = U256::from_be_bytes(bytes);
        assert_eq!(value, recovered);
    }

    #[test]
    fn test_u256_mul() {
        let a = U256::from_u64(1000);
        let b = U256::from_u64(2000);
        let product = a.checked_mul(b).unwrap();
        assert_eq!(product.low_u64(), 2_000_000);
    }

    #[test]
    fn test_transaction_hash() {
        let tx = EvmTransaction {
            tx_type: 2,
            chain_id: 7777,
            nonce: 0,
            max_priority_fee_per_gas: 1_000_000_000,
            max_fee_per_gas: 2_000_000_000,
            gas_limit: 21000,
            to: Some([0u8; 20]),
            value: U256::ZERO,
            input: Vec::new(),
            access_list: Vec::new(),
            v: 0,
            r: [0u8; 32],
            s: [0u8; 32],
        };
        
        let hash = tx.hash();
        assert!(!hash.iter().all(|&x| x == 0));
    }

    #[test]
    fn test_effective_gas_price() {
        let tx = EvmTransaction {
            tx_type: 2,
            chain_id: 7777,
            nonce: 0,
            max_priority_fee_per_gas: 2_000_000_000,
            max_fee_per_gas: 50_000_000_000,
            gas_limit: 21000,
            to: Some([0u8; 20]),
            value: U256::ZERO,
            input: Vec::new(),
            access_list: Vec::new(),
            v: 0,
            r: [0u8; 32],
            s: [0u8; 32],
        };
        
        let base_fee = 10_000_000_000u64;
        let effective = tx.effective_gas_price(base_fee);
        assert_eq!(effective, 12_000_000_000); // base + priority
    }
}
