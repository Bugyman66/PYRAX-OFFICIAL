//! TriStream DAG - Stream definitions
//!
//! PYRAX uses three parallel consensus streams:
//! - Stream A: BLAKE3 PoW (ASIC-friendly, 10s blocks, UTXO transactions)
//! - Stream B: KAWPOW PoW (GPU mining, 60s blocks, AI compute jobs)
//! - Stream C: ZK-STARK + PoS (Finality checkpoints)
//!
//! All streams are unified via GHOSTDAG ordering into a single state view.

use serde::{Deserialize, Serialize};

/// The three consensus streams in PYRAX
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Stream {
    /// Stream A: BLAKE3 PoW, 10-second blocks, UTXO transactions
    /// ASIC-friendly for high-speed value transfer
    A = 0,
    
    /// Stream B: KAWPOW PoW, 60-second blocks, GPU mining + AI jobs
    /// GPU-resistant algorithm, dual-use for AI compute
    B = 1,
    
    /// Stream C: ZK-STARK + PoS, finality checkpoints
    /// Provides irreversible finality via ZK proofs
    C = 2,
}

impl Stream {
    /// Block time in seconds for this stream
    pub const fn block_time_secs(&self) -> u64 {
        match self {
            Stream::A => 10,  // Fast UTXO transactions
            Stream::B => 60,  // GPU mining blocks
            Stream::C => 0,   // Event-driven (checkpoints)
        }
    }
    
    /// Block reward in base units (1 PYRAX = 10^8 base units)
    pub const fn block_reward(&self) -> u64 {
        match self {
            Stream::A => 50 * 100_000_000,   // 50 PYRAX
            Stream::B => 100 * 100_000_000,  // 100 PYRAX
            Stream::C => 10 * 100_000_000,   // 10 PYRAX per checkpoint
        }
    }
    
    /// Gas fee share percentage (out of 100)
    pub const fn gas_share_percent(&self) -> u8 {
        match self {
            Stream::A => 20,  // 20% to ASIC miners
            Stream::B => 40,  // 40% to GPU miners
            Stream::C => 30,  // 30% to stakers
        }
    }
    
    /// PoW algorithm name
    pub const fn algorithm(&self) -> &'static str {
        match self {
            Stream::A => "BLAKE3",
            Stream::B => "KAWPOW",
            Stream::C => "ZK-STARK",
        }
    }
    
    /// Whether this stream uses Proof of Work
    pub const fn is_pow(&self) -> bool {
        matches!(self, Stream::A | Stream::B)
    }
    
    /// Default RPC port offset for this stream
    pub const fn default_port_offset(&self) -> u16 {
        match self {
            Stream::A => 0,   // 8545 for mainnet
            Stream::B => 1,   // 8546 for mainnet
            Stream::C => 2,   // 8547 for mainnet
        }
    }
    
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Stream::A),
            1 => Some(Stream::B),
            2 => Some(Stream::C),
            _ => None,
        }
    }
}

impl std::fmt::Display for Stream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Stream::A => write!(f, "Stream A (BLAKE3)"),
            Stream::B => write!(f, "Stream B (KAWPOW)"),
            Stream::C => write!(f, "Stream C (ZK-STARK)"),
        }
    }
}

/// Protocol treasury receives 10% of all gas fees
pub const TREASURY_GAS_SHARE_PERCENT: u8 = 10;

/// Token decimals (1 PYRAX = 10^8 base units)
pub const TOKEN_DECIMALS: u8 = 8;

/// Maximum token supply: 100 billion PYRAX
pub const MAX_SUPPLY: u64 = 100_000_000_000 * 100_000_000; // In base units

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gas_shares_sum_to_90() {
        // 90% goes to streams, 10% to treasury
        let total = Stream::A.gas_share_percent() 
            + Stream::B.gas_share_percent() 
            + Stream::C.gas_share_percent();
        assert_eq!(total, 90);
        assert_eq!(total + TREASURY_GAS_SHARE_PERCENT, 100);
    }
    
    #[test]
    fn test_stream_roundtrip() {
        for stream in [Stream::A, Stream::B, Stream::C] {
            let byte = stream as u8;
            let recovered = Stream::from_u8(byte).unwrap();
            assert_eq!(stream, recovered);
        }
    }
}
