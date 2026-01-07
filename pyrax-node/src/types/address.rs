use serde::{Deserialize, Serialize};
use std::fmt;
use tiny_keccak::{Hasher, Keccak};

use super::H160;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Address(pub [u8; 20]);

impl Address {
    pub const ZERO: Address = Address([0u8; 20]);

    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        if slice.len() != 20 {
            return None;
        }
        let mut arr = [0u8; 20];
        arr.copy_from_slice(slice);
        Some(Address(arr))
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

    pub fn to_h160(&self) -> H160 {
        H160::from_slice(&self.0)
    }

    pub fn from_h160(h: H160) -> Self {
        Address(h.0)
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
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Address({})", self.to_checksum_string())
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_checksum_string())
    }
}

impl Serialize for Address {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_checksum_string())
    }
}

impl<'de> Deserialize<'de> for Address {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let s = s.strip_prefix("0x").unwrap_or(&s);
        let bytes = hex::decode(s).map_err(serde::de::Error::custom)?;
        Address::from_slice(&bytes).ok_or_else(|| {
            serde::de::Error::custom("Invalid address length")
        })
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
        let addr = Address::from_slice(&hex::decode("5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed").unwrap()).unwrap();
        assert_eq!(addr.to_checksum_string(), "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed");
    }
}
