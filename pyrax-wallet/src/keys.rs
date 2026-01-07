use k256::ecdsa::{SigningKey, VerifyingKey};
use k256::elliptic_curve::rand_core::OsRng;
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::{Result, WalletError, Address};

#[derive(Clone, ZeroizeOnDrop)]
pub struct PrivateKey {
    #[zeroize(skip)]
    inner: SigningKey,
    bytes: [u8; 32],
}

impl PrivateKey {
    pub fn random() -> Self {
        let inner = SigningKey::random(&mut OsRng);
        let bytes: [u8; 32] = inner.to_bytes().into();
        Self { inner, bytes }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 32 {
            return Err(WalletError::InvalidPrivateKey(
                "Private key must be 32 bytes".into()
            ));
        }
        
        let inner = SigningKey::from_bytes(bytes.into())
            .map_err(|e| WalletError::InvalidPrivateKey(e.to_string()))?;
        
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(bytes);
        
        Ok(Self {
            inner,
            bytes: key_bytes,
        })
    }

    pub fn from_hex(hex_str: &str) -> Result<Self> {
        let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);
        let bytes = hex::decode(hex_str)
            .map_err(|_| WalletError::InvalidPrivateKey("Invalid hex".into()))?;
        Self::from_bytes(&bytes)
    }

    pub fn to_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }

    pub fn to_hex(&self) -> String {
        hex::encode(&self.bytes)
    }

    pub fn public_key(&self) -> PublicKey {
        PublicKey::from_signing_key(&self.inner)
    }

    pub fn address(&self) -> Address {
        self.public_key().address()
    }

    pub(crate) fn signing_key(&self) -> &SigningKey {
        &self.inner
    }
}

impl std::fmt::Debug for PrivateKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PrivateKey([REDACTED])")
    }
}

#[derive(Clone, Debug)]
pub struct PublicKey {
    inner: VerifyingKey,
    compressed: [u8; 33],
    uncompressed: [u8; 65],
}

impl PublicKey {
    pub fn from_signing_key(key: &SigningKey) -> Self {
        let verifying_key = key.verifying_key();
        Self::from_verifying_key(verifying_key)
    }

    pub fn from_verifying_key(key: &VerifyingKey) -> Self {
        let point = key.to_encoded_point(false);
        let uncompressed_bytes = point.as_bytes();
        
        let point_compressed = key.to_encoded_point(true);
        let compressed_bytes = point_compressed.as_bytes();
        
        let mut compressed = [0u8; 33];
        let mut uncompressed = [0u8; 65];
        
        compressed.copy_from_slice(compressed_bytes);
        uncompressed.copy_from_slice(uncompressed_bytes);
        
        Self {
            inner: *key,
            compressed,
            uncompressed,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let key = match bytes.len() {
            33 => {
                VerifyingKey::from_sec1_bytes(bytes)
                    .map_err(|e| WalletError::InvalidPublicKey(e.to_string()))?
            }
            65 => {
                VerifyingKey::from_sec1_bytes(bytes)
                    .map_err(|e| WalletError::InvalidPublicKey(e.to_string()))?
            }
            64 => {
                // Uncompressed without prefix
                let mut full = [0u8; 65];
                full[0] = 0x04;
                full[1..].copy_from_slice(bytes);
                VerifyingKey::from_sec1_bytes(&full)
                    .map_err(|e| WalletError::InvalidPublicKey(e.to_string()))?
            }
            _ => {
                return Err(WalletError::InvalidPublicKey(
                    "Public key must be 33 (compressed) or 65 (uncompressed) bytes".into()
                ));
            }
        };
        
        Ok(Self::from_verifying_key(&key))
    }

    pub fn to_bytes_compressed(&self) -> &[u8; 33] {
        &self.compressed
    }

    pub fn to_bytes_uncompressed(&self) -> &[u8; 65] {
        &self.uncompressed
    }

    pub fn to_hex_compressed(&self) -> String {
        hex::encode(&self.compressed)
    }

    pub fn to_hex_uncompressed(&self) -> String {
        hex::encode(&self.uncompressed)
    }

    pub fn address(&self) -> Address {
        Address::from_public_key(&self.uncompressed[1..])
    }

    pub(crate) fn verifying_key(&self) -> &VerifyingKey {
        &self.inner
    }
}

impl Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_hex_compressed())
    }
}

impl<'de> Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let s = s.strip_prefix("0x").unwrap_or(&s);
        let bytes = hex::decode(s).map_err(serde::de::Error::custom)?;
        PublicKey::from_bytes(&bytes).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let key = PrivateKey::random();
        let pubkey = key.public_key();
        let address = key.address();
        
        assert_eq!(pubkey.to_bytes_compressed().len(), 33);
        assert_eq!(pubkey.to_bytes_uncompressed().len(), 65);
        assert_eq!(address.as_bytes().len(), 20);
    }

    #[test]
    fn test_key_roundtrip() {
        let key = PrivateKey::random();
        let bytes = key.to_bytes();
        let restored = PrivateKey::from_bytes(bytes).unwrap();
        
        assert_eq!(key.address(), restored.address());
    }

    #[test]
    fn test_known_key() {
        // Test vector
        let key = PrivateKey::from_hex(
            "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
        ).unwrap();
        
        let pubkey = key.public_key();
        assert!(pubkey.to_hex_compressed().len() == 66); // 33 bytes * 2
    }
}
