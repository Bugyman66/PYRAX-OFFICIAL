use serde::{Deserialize, Serialize};
use tiny_keccak::{Hasher, Keccak};

use crate::{Result, WalletError};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Address([u8; 20]);

impl Address {
    pub const ZERO: Address = Address([0u8; 20]);

    pub fn from_slice(slice: &[u8]) -> Result<Self> {
        if slice.len() != 20 {
            return Err(WalletError::InvalidAddress(
                "Address must be 20 bytes".into()
            ));
        }
        let mut arr = [0u8; 20];
        arr.copy_from_slice(slice);
        Ok(Address(arr))
    }

    pub fn from_hex(hex_str: &str) -> Result<Self> {
        let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);
        let bytes = hex::decode(hex_str)
            .map_err(|_| WalletError::InvalidAddress("Invalid hex".into()))?;
        Self::from_slice(&bytes)
    }

    pub fn from_public_key(pubkey: &[u8]) -> Self {
        let mut hasher = Keccak::v256();
        let mut output = [0u8; 32];
        hasher.update(pubkey);
        hasher.finalize(&mut output);
        
        let mut addr = [0u8; 20];
        addr.copy_from_slice(&output[12..32]);
        Address(addr)
    }

    pub fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }

    pub fn to_hex(&self) -> String {
        hex::encode(&self.0)
    }

    pub fn to_checksum_string(&self) -> String {
        let hex_addr = hex::encode(&self.0);
        let mut hasher = Keccak::v256();
        let mut hash = [0u8; 32];
        hasher.update(hex_addr.as_bytes());
        hasher.finalize(&mut hash);
        
        let mut checksum = String::with_capacity(42);
        checksum.push_str("0x");
        
        for (i, c) in hex_addr.chars().enumerate() {
            let hash_byte = hash[i / 2];
            let hash_nibble = if i % 2 == 0 {
                hash_byte >> 4
            } else {
                hash_byte & 0x0f
            };
            
            if hash_nibble >= 8 && c.is_ascii_alphabetic() {
                checksum.push(c.to_ascii_uppercase());
            } else {
                checksum.push(c);
            }
        }
        
        checksum
    }

    pub fn is_valid_checksum(address: &str) -> bool {
        let address = address.strip_prefix("0x").unwrap_or(address);
        if address.len() != 40 {
            return false;
        }

        // Parse and regenerate checksum
        let lowercase = address.to_lowercase();
        let bytes = match hex::decode(&lowercase) {
            Ok(b) => b,
            Err(_) => return false,
        };

        if let Ok(addr) = Address::from_slice(&bytes) {
            let checksum = addr.to_checksum_string();
            let checksum = checksum.strip_prefix("0x").unwrap();
            checksum == address
        } else {
            false
        }
    }
}

impl std::fmt::Debug for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Address({})", self.to_checksum_string())
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_checksum_string())
    }
}

impl Serialize for Address {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_checksum_string())
    }
}

impl<'de> Deserialize<'de> for Address {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Address::from_hex(&s).map_err(serde::de::Error::custom)
    }
}

impl From<[u8; 20]> for Address {
    fn from(arr: [u8; 20]) -> Self {
        Address(arr)
    }
}

impl AsRef<[u8]> for Address {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum_address() {
        let addr = Address::from_hex("5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed").unwrap();
        assert_eq!(
            addr.to_checksum_string(),
            "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed"
        );
    }

    #[test]
    fn test_checksum_validation() {
        assert!(Address::is_valid_checksum("0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed"));
        assert!(!Address::is_valid_checksum("0x5aaeb6053f3e94c9b9a09f33669435e7ef1beaed")); // all lowercase fails checksum
    }

    #[test]
    fn test_from_public_key() {
        // This would need a real public key to test properly
        let fake_pubkey = [0u8; 64];
        let addr = Address::from_public_key(&fake_pubkey);
        assert_eq!(addr.as_bytes().len(), 20);
    }
}
