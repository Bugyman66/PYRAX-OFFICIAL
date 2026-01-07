//! Unified Address Format
//!
//! Single address format that works with both UTXO and Account models

use std::fmt;
use serde::{Deserialize, Serialize};
use blake3::Hasher;

use crate::types::Address;

/// Address type indicator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AddressType {
    /// UTXO-only address (legacy)
    Utxo,
    /// EVM-only address (legacy)
    Evm,
    /// Unified address (works with both)
    Unified,
}

/// Network prefix for bech32 encoding
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkPrefix {
    /// Mainnet
    Mainnet,
    /// Testnet
    Testnet,
    /// Devnet
    Devnet,
}

impl NetworkPrefix {
    pub fn as_str(&self) -> &'static str {
        match self {
            NetworkPrefix::Mainnet => "pyrax",
            NetworkPrefix::Testnet => "tpyrax",
            NetworkPrefix::Devnet => "dpyrax",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pyrax" => Some(NetworkPrefix::Mainnet),
            "tpyrax" => Some(NetworkPrefix::Testnet),
            "dpyrax" => Some(NetworkPrefix::Devnet),
            _ => None,
        }
    }
}

/// Unified address that works with both UTXO and EVM
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnifiedAddress {
    /// Address type
    pub address_type: AddressType,
    /// Network
    pub network: u8, // 0 = mainnet, 1 = testnet, 2 = devnet
    /// Core 20-byte address (compatible with both models)
    pub bytes: [u8; 20],
    /// Optional checksum for verification
    pub checksum: [u8; 4],
}

impl UnifiedAddress {
    /// Create unified address from raw bytes
    pub fn from_bytes(bytes: [u8; 20], network: NetworkPrefix) -> Self {
        let checksum = Self::compute_checksum(&bytes, AddressType::Unified);
        Self {
            address_type: AddressType::Unified,
            network: match network {
                NetworkPrefix::Mainnet => 0,
                NetworkPrefix::Testnet => 1,
                NetworkPrefix::Devnet => 2,
            },
            bytes,
            checksum,
        }
    }

    /// Create from UTXO address
    pub fn from_utxo_address(addr: &Address, network: NetworkPrefix) -> Self {
        let checksum = Self::compute_checksum(&addr.0, AddressType::Unified);
        Self {
            address_type: AddressType::Unified,
            network: match network {
                NetworkPrefix::Mainnet => 0,
                NetworkPrefix::Testnet => 1,
                NetworkPrefix::Devnet => 2,
            },
            bytes: addr.0,
            checksum,
        }
    }

    /// Create from EVM address (20-byte hex)
    pub fn from_evm_address(evm_addr: [u8; 20], network: NetworkPrefix) -> Self {
        let checksum = Self::compute_checksum(&evm_addr, AddressType::Unified);
        Self {
            address_type: AddressType::Unified,
            network: match network {
                NetworkPrefix::Mainnet => 0,
                NetworkPrefix::Testnet => 1,
                NetworkPrefix::Devnet => 2,
            },
            bytes: evm_addr,
            checksum,
        }
    }

    /// Create from public key (derives address)
    pub fn from_public_key(pubkey: &[u8], network: NetworkPrefix) -> Self {
        let mut hasher = Hasher::new();
        hasher.update(pubkey);
        let hash = hasher.finalize();
        
        // Take last 20 bytes as address (similar to Ethereum)
        let mut bytes = [0u8; 20];
        bytes.copy_from_slice(&hash.as_bytes()[12..32]);
        
        Self::from_bytes(bytes, network)
    }

    /// Compute checksum for address
    fn compute_checksum(bytes: &[u8; 20], addr_type: AddressType) -> [u8; 4] {
        let mut hasher = Hasher::new();
        hasher.update(&[addr_type as u8]);
        hasher.update(bytes);
        let hash = hasher.finalize();
        
        let mut checksum = [0u8; 4];
        checksum.copy_from_slice(&hash.as_bytes()[0..4]);
        checksum
    }

    /// Verify checksum
    pub fn verify_checksum(&self) -> bool {
        let computed = Self::compute_checksum(&self.bytes, self.address_type);
        self.checksum == computed
    }

    /// Get as UTXO Address type
    pub fn to_utxo_address(&self) -> Address {
        Address(self.bytes)
    }

    /// Get as EVM address (20-byte array)
    pub fn to_evm_address(&self) -> [u8; 20] {
        self.bytes
    }

    /// Get network prefix
    pub fn network_prefix(&self) -> NetworkPrefix {
        match self.network {
            0 => NetworkPrefix::Mainnet,
            1 => NetworkPrefix::Testnet,
            _ => NetworkPrefix::Devnet,
        }
    }

    /// Encode as bech32-like string (custom encoding)
    pub fn to_bech32(&self) -> Result<String, AddressError> {
        let prefix = self.network_prefix().as_str();
        
        // Build data: [type_byte, 20 address bytes, 4 checksum bytes]
        let mut data = Vec::with_capacity(25);
        data.push(self.address_type as u8);
        data.extend_from_slice(&self.bytes);
        data.extend_from_slice(&self.checksum);
        
        // Use base32 encoding with bech32 charset
        let encoded_data = base32_encode(&data);
        Ok(format!("{}1{}", prefix, encoded_data))
    }

    /// Decode from bech32-like string
    pub fn from_bech32(s: &str) -> Result<Self, AddressError> {
        // Find the separator '1'
        let sep_pos = s.rfind('1')
            .ok_or_else(|| AddressError::DecodingError("Missing separator".to_string()))?;
        
        let hrp = &s[..sep_pos];
        let data_part = &s[sep_pos + 1..];
        
        let network = NetworkPrefix::from_str(hrp)
            .ok_or_else(|| AddressError::InvalidNetwork(hrp.to_string()))?;
        
        let bytes = base32_decode(data_part)
            .map_err(|e| AddressError::DecodingError(e))?;
        
        if bytes.len() != 25 {
            return Err(AddressError::InvalidLength(bytes.len()));
        }
        
        let address_type = match bytes[0] {
            0 => AddressType::Utxo,
            1 => AddressType::Evm,
            2 => AddressType::Unified,
            _ => return Err(AddressError::InvalidType(bytes[0])),
        };
        
        let mut addr_bytes = [0u8; 20];
        addr_bytes.copy_from_slice(&bytes[1..21]);
        
        let mut checksum = [0u8; 4];
        checksum.copy_from_slice(&bytes[21..25]);
        
        let addr = Self {
            address_type,
            network: match network {
                NetworkPrefix::Mainnet => 0,
                NetworkPrefix::Testnet => 1,
                NetworkPrefix::Devnet => 2,
            },
            bytes: addr_bytes,
            checksum,
        };
        
        if !addr.verify_checksum() {
            return Err(AddressError::InvalidChecksum);
        }
        
        Ok(addr)
    }

    /// Format as hex string (EVM-compatible)
    pub fn to_hex(&self) -> String {
        format!("0x{}", hex::encode(&self.bytes))
    }

    /// Parse from hex string
    pub fn from_hex(s: &str, network: NetworkPrefix) -> Result<Self, AddressError> {
        let s = s.strip_prefix("0x").unwrap_or(s);
        
        if s.len() != 40 {
            return Err(AddressError::InvalidLength(s.len() / 2));
        }
        
        let bytes = hex::decode(s)
            .map_err(|e| AddressError::DecodingError(e.to_string()))?;
        
        let mut addr_bytes = [0u8; 20];
        addr_bytes.copy_from_slice(&bytes);
        
        Ok(Self::from_bytes(addr_bytes, network))
    }

    /// Check if addresses are equal (ignoring network)
    pub fn same_address(&self, other: &UnifiedAddress) -> bool {
        self.bytes == other.bytes
    }
}

impl fmt::Display for UnifiedAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.to_bech32() {
            Ok(s) => write!(f, "{}", s),
            Err(_) => write!(f, "0x{}", hex::encode(&self.bytes)),
        }
    }
}

/// Address errors
#[derive(Debug, thiserror::Error)]
pub enum AddressError {
    #[error("Encoding error: {0}")]
    EncodingError(String),

    #[error("Decoding error: {0}")]
    DecodingError(String),

    #[error("Invalid bech32 variant")]
    InvalidVariant,

    #[error("Invalid network: {0}")]
    InvalidNetwork(String),

    #[error("Invalid length: expected 20 bytes, got {0}")]
    InvalidLength(usize),

    #[error("Invalid address type: {0}")]
    InvalidType(u8),

    #[error("Invalid checksum")]
    InvalidChecksum,

    #[error("Invalid hex: {0}")]
    InvalidHex(String),
}

/// Address converter utilities
pub struct AddressConverter;

impl AddressConverter {
    /// Convert UTXO address to unified
    pub fn utxo_to_unified(addr: &Address, network: NetworkPrefix) -> UnifiedAddress {
        UnifiedAddress::from_utxo_address(addr, network)
    }

    /// Convert EVM address to unified
    pub fn evm_to_unified(evm_addr: [u8; 20], network: NetworkPrefix) -> UnifiedAddress {
        UnifiedAddress::from_evm_address(evm_addr, network)
    }

    /// Convert unified to UTXO
    pub fn unified_to_utxo(unified: &UnifiedAddress) -> Address {
        unified.to_utxo_address()
    }

    /// Convert unified to EVM
    pub fn unified_to_evm(unified: &UnifiedAddress) -> [u8; 20] {
        unified.to_evm_address()
    }

    /// Parse any address format
    pub fn parse(s: &str, default_network: NetworkPrefix) -> Result<UnifiedAddress, AddressError> {
        // Try bech32 first
        if let Ok(addr) = UnifiedAddress::from_bech32(s) {
            return Ok(addr);
        }
        
        // Try hex format
        if s.starts_with("0x") || s.len() == 40 {
            return UnifiedAddress::from_hex(s, default_network);
        }
        
        Err(AddressError::DecodingError("Unrecognized address format".to_string()))
    }

    /// Check if string is valid address
    pub fn is_valid(s: &str) -> bool {
        Self::parse(s, NetworkPrefix::Mainnet).is_ok()
    }
}

/// Bech32 charset for encoding
const BECH32_CHARSET: &[u8; 32] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";

/// Encode bytes to bech32 base32 string
fn base32_encode(data: &[u8]) -> String {
    let mut result = String::new();
    let mut acc = 0u32;
    let mut bits = 0u32;
    
    for &byte in data {
        acc = (acc << 8) | (byte as u32);
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            let idx = ((acc >> bits) & 0x1f) as usize;
            result.push(BECH32_CHARSET[idx] as char);
        }
    }
    
    if bits > 0 {
        let idx = ((acc << (5 - bits)) & 0x1f) as usize;
        result.push(BECH32_CHARSET[idx] as char);
    }
    
    result
}

/// Decode bech32 base32 string to bytes
fn base32_decode(s: &str) -> Result<Vec<u8>, String> {
    let mut result = Vec::new();
    let mut acc = 0u32;
    let mut bits = 0u32;
    
    for c in s.chars() {
        let idx = BECH32_CHARSET.iter().position(|&x| x as char == c)
            .ok_or_else(|| format!("Invalid character: {}", c))?;
        
        acc = (acc << 5) | (idx as u32);
        bits += 5;
        
        while bits >= 8 {
            bits -= 8;
            result.push(((acc >> bits) & 0xff) as u8);
        }
    }
    
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_from_bytes() {
        let bytes = [1u8; 20];
        let addr = UnifiedAddress::from_bytes(bytes, NetworkPrefix::Mainnet);
        
        assert_eq!(addr.address_type, AddressType::Unified);
        assert_eq!(addr.bytes, bytes);
        assert!(addr.verify_checksum());
    }

    #[test]
    fn test_bech32_roundtrip() {
        let bytes = [42u8; 20];
        let addr = UnifiedAddress::from_bytes(bytes, NetworkPrefix::Mainnet);
        
        let encoded = addr.to_bech32().unwrap();
        assert!(encoded.starts_with("pyrax1"));
        
        let decoded = UnifiedAddress::from_bech32(&encoded).unwrap();
        assert_eq!(addr.bytes, decoded.bytes);
        assert!(decoded.verify_checksum());
    }

    #[test]
    fn test_hex_roundtrip() {
        let bytes = [0xab; 20];
        let addr = UnifiedAddress::from_bytes(bytes, NetworkPrefix::Testnet);
        
        let hex = addr.to_hex();
        assert!(hex.starts_with("0x"));
        
        let parsed = UnifiedAddress::from_hex(&hex, NetworkPrefix::Testnet).unwrap();
        assert_eq!(addr.bytes, parsed.bytes);
    }

    #[test]
    fn test_utxo_conversion() {
        let utxo_addr = Address([5u8; 20]);
        let unified = UnifiedAddress::from_utxo_address(&utxo_addr, NetworkPrefix::Devnet);
        
        let back = unified.to_utxo_address();
        assert_eq!(utxo_addr, back);
    }

    #[test]
    fn test_evm_conversion() {
        let evm_addr = [0xde; 20];
        let unified = UnifiedAddress::from_evm_address(evm_addr, NetworkPrefix::Mainnet);
        
        let back = unified.to_evm_address();
        assert_eq!(evm_addr, back);
    }

    #[test]
    fn test_from_public_key() {
        let pubkey = [7u8; 33]; // Compressed pubkey
        let addr = UnifiedAddress::from_public_key(&pubkey, NetworkPrefix::Mainnet);
        
        assert_eq!(addr.address_type, AddressType::Unified);
        assert!(addr.verify_checksum());
    }

    #[test]
    fn test_network_prefixes() {
        let bytes = [1u8; 20];
        
        let mainnet = UnifiedAddress::from_bytes(bytes, NetworkPrefix::Mainnet);
        assert!(mainnet.to_bech32().unwrap().starts_with("pyrax1"));
        
        let testnet = UnifiedAddress::from_bytes(bytes, NetworkPrefix::Testnet);
        assert!(testnet.to_bech32().unwrap().starts_with("tpyrax1"));
        
        let devnet = UnifiedAddress::from_bytes(bytes, NetworkPrefix::Devnet);
        assert!(devnet.to_bech32().unwrap().starts_with("dpyrax1"));
    }

    #[test]
    fn test_address_converter() {
        let utxo = Address([10u8; 20]);
        let unified = AddressConverter::utxo_to_unified(&utxo, NetworkPrefix::Mainnet);
        let back = AddressConverter::unified_to_utxo(&unified);
        assert_eq!(utxo, back);
    }

    #[test]
    fn test_parse_any_format() {
        // Hex format
        let hex = "0xabababababababababababababababababababab";
        let addr = AddressConverter::parse(hex, NetworkPrefix::Mainnet).unwrap();
        assert_eq!(addr.bytes, [0xab; 20]);
        
        // Bech32 format (create then parse)
        let original = UnifiedAddress::from_bytes([0xcd; 20], NetworkPrefix::Testnet);
        let bech32 = original.to_bech32().unwrap();
        let parsed = AddressConverter::parse(&bech32, NetworkPrefix::Testnet).unwrap();
        assert_eq!(original.bytes, parsed.bytes);
    }

    #[test]
    fn test_invalid_checksum() {
        let bytes = [1u8; 20];
        let mut addr = UnifiedAddress::from_bytes(bytes, NetworkPrefix::Mainnet);
        
        // Corrupt checksum
        addr.checksum[0] ^= 0xff;
        assert!(!addr.verify_checksum());
    }

    #[test]
    fn test_same_address() {
        let bytes = [42u8; 20];
        let mainnet = UnifiedAddress::from_bytes(bytes, NetworkPrefix::Mainnet);
        let testnet = UnifiedAddress::from_bytes(bytes, NetworkPrefix::Testnet);
        
        assert!(mainnet.same_address(&testnet));
        
        let different = UnifiedAddress::from_bytes([43u8; 20], NetworkPrefix::Mainnet);
        assert!(!mainnet.same_address(&different));
    }
}
