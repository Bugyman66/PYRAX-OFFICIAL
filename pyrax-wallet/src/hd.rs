use bip39::Mnemonic as Bip39Mnemonic;
use hmac::{Hmac, Mac};
use sha2::Sha512;
use k256::ecdsa::SigningKey;
use k256::elliptic_curve::sec1::ToEncodedPoint;
use rand::RngCore;

use crate::{Result, WalletError, PrivateKey, Address};

type HmacSha512 = Hmac<Sha512>;

pub struct Mnemonic {
    inner: Bip39Mnemonic,
    phrase: String,
}

impl Mnemonic {
    pub fn generate(word_count: usize) -> Result<Self> {
        // Calculate entropy bytes needed for word count
        // 12 words = 128 bits = 16 bytes
        // 15 words = 160 bits = 20 bytes
        // 18 words = 192 bits = 24 bytes
        // 21 words = 224 bits = 28 bytes
        // 24 words = 256 bits = 32 bytes
        let entropy_bytes = match word_count {
            12 => 16,
            15 => 20,
            18 => 24,
            21 => 28,
            24 => 32,
            _ => return Err(WalletError::InvalidMnemonic(
                "Word count must be 12, 15, 18, 21, or 24".into()
            )),
        };
        
        let mut entropy = vec![0u8; entropy_bytes];
        rand::thread_rng().fill_bytes(&mut entropy);
        
        let mnemonic = Bip39Mnemonic::from_entropy(&entropy)
            .map_err(|e| WalletError::InvalidMnemonic(e.to_string()))?;
        
        let phrase = mnemonic.to_string();
        Ok(Self { inner: mnemonic, phrase })
    }

    pub fn from_phrase(phrase: &str) -> Result<Self> {
        let mnemonic = Bip39Mnemonic::parse_normalized(phrase)
            .map_err(|e| WalletError::InvalidMnemonic(e.to_string()))?;
        let phrase_str = mnemonic.to_string();
        Ok(Self { inner: mnemonic, phrase: phrase_str })
    }

    pub fn phrase(&self) -> &str {
        &self.phrase
    }

    pub fn to_seed(&self, passphrase: &str) -> [u8; 64] {
        let salt = format!("mnemonic{}", passphrase);
        let mut seed = [0u8; 64];
        pbkdf2::pbkdf2_hmac::<Sha512>(
            self.phrase.as_bytes(),
            salt.as_bytes(),
            2048,
            &mut seed,
        );
        seed
    }

    pub fn to_hd_wallet(&self, passphrase: &str) -> HDWallet {
        let seed = self.to_seed(passphrase);
        HDWallet::from_seed(&seed)
    }
}

impl std::fmt::Debug for Mnemonic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Mnemonic([REDACTED])")
    }
}

pub struct HDWallet {
    master_key: [u8; 32],
    chain_code: [u8; 32],
}

impl HDWallet {
    pub fn from_seed(seed: &[u8]) -> Self {
        let mut mac = HmacSha512::new_from_slice(b"Bitcoin seed").unwrap();
        mac.update(seed);
        let result = mac.finalize().into_bytes();
        
        let mut master_key = [0u8; 32];
        let mut chain_code = [0u8; 32];
        master_key.copy_from_slice(&result[0..32]);
        chain_code.copy_from_slice(&result[32..64]);
        
        Self { master_key, chain_code }
    }

    pub fn derive_path(&self, path: &DerivationPath) -> Result<PrivateKey> {
        let mut key = self.master_key;
        let mut chain = self.chain_code;
        
        for &index in &path.indices {
            let (new_key, new_chain) = derive_child(&key, &chain, index)?;
            key = new_key;
            chain = new_chain;
        }
        
        PrivateKey::from_bytes(&key)
    }

    pub fn derive_account(&self, account: u32) -> Result<PrivateKey> {
        // Standard PYRAX path: m/44'/PYRAX_COIN_TYPE'/account'/0/0
        // Using coin type 60 (Ethereum) for compatibility
        let path = DerivationPath::pyrax(account, 0, 0);
        self.derive_path(&path)
    }

    pub fn derive_address(&self, account: u32, index: u32) -> Result<Address> {
        let path = DerivationPath::pyrax(account, 0, index);
        let key = self.derive_path(&path)?;
        Ok(key.address())
    }
}

fn derive_child(
    parent_key: &[u8; 32],
    parent_chain: &[u8; 32],
    index: u32,
) -> Result<([u8; 32], [u8; 32])> {
    let mut mac = HmacSha512::new_from_slice(parent_chain).unwrap();
    
    if index >= 0x80000000 {
        // Hardened derivation
        mac.update(&[0x00]);
        mac.update(parent_key);
    } else {
        // Normal derivation - use public key
        let signing_key = SigningKey::from_bytes(parent_key.into())
            .map_err(|e| WalletError::InvalidPrivateKey(e.to_string()))?;
        let public_key = signing_key.verifying_key();
        let point = public_key.to_encoded_point(true);
        mac.update(point.as_bytes());
    }
    
    mac.update(&index.to_be_bytes());
    let result = mac.finalize().into_bytes();
    
    let mut child_key = [0u8; 32];
    let mut child_chain = [0u8; 32];
    child_key.copy_from_slice(&result[0..32]);
    child_chain.copy_from_slice(&result[32..64]);
    
    // Add parent key to child key (mod n)
    use k256::elliptic_curve::ops::Reduce;
    use k256::U256;
    
    let parent_uint = U256::from_be_slice(parent_key);
    let child_uint = U256::from_be_slice(&child_key);
    
    // Add and reduce modulo curve order
    let parent_scalar = <k256::Scalar as Reduce<U256>>::reduce(parent_uint);
    let child_scalar = <k256::Scalar as Reduce<U256>>::reduce(child_uint);
    
    let sum = parent_scalar + child_scalar;
    let sum_bytes: [u8; 32] = sum.to_bytes().into();
    
    Ok((sum_bytes, child_chain))
}

#[derive(Debug, Clone)]
pub struct DerivationPath {
    pub indices: Vec<u32>,
}

impl DerivationPath {
    pub fn new(indices: Vec<u32>) -> Self {
        Self { indices }
    }

    pub fn parse(path: &str) -> Result<Self> {
        let path = path.strip_prefix("m/").unwrap_or(path);
        
        let indices: Result<Vec<u32>> = path
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|s| {
                let (num_str, hardened) = if s.ends_with('\'') || s.ends_with('h') {
                    (&s[..s.len()-1], true)
                } else {
                    (s, false)
                };
                
                let num: u32 = num_str.parse()
                    .map_err(|_| WalletError::InvalidDerivationPath(
                        format!("Invalid index: {}", s)
                    ))?;
                
                if hardened {
                    Ok(num | 0x80000000)
                } else {
                    Ok(num)
                }
            })
            .collect();
        
        Ok(Self { indices: indices? })
    }

    pub fn pyrax(account: u32, change: u32, index: u32) -> Self {
        // m/44'/60'/account'/change/index
        // Using 60 for Ethereum compatibility
        Self {
            indices: vec![
                44 | 0x80000000,
                60 | 0x80000000,
                account | 0x80000000,
                change,
                index,
            ],
        }
    }

    pub fn to_string(&self) -> String {
        let parts: Vec<String> = self.indices.iter().map(|&i| {
            if i >= 0x80000000 {
                format!("{}'", i & 0x7FFFFFFF)
            } else {
                i.to_string()
            }
        }).collect();
        
        format!("m/{}", parts.join("/"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mnemonic_generation() {
        let mnemonic = Mnemonic::generate(12).unwrap();
        let words: Vec<&str> = mnemonic.phrase().split_whitespace().collect();
        assert_eq!(words.len(), 12);
    }

    #[test]
    fn test_mnemonic_roundtrip() {
        let mnemonic = Mnemonic::generate(24).unwrap();
        let phrase = mnemonic.phrase().to_string();
        let restored = Mnemonic::from_phrase(&phrase).unwrap();
        assert_eq!(mnemonic.phrase(), restored.phrase());
    }

    #[test]
    fn test_derivation_path_parse() {
        let path = DerivationPath::parse("m/44'/60'/0'/0/0").unwrap();
        assert_eq!(path.indices.len(), 5);
        assert_eq!(path.indices[0], 44 | 0x80000000);
        assert_eq!(path.indices[1], 60 | 0x80000000);
    }

    #[test]
    fn test_hd_derivation() {
        let mnemonic = Mnemonic::generate(12).unwrap();
        let wallet = mnemonic.to_hd_wallet("");
        
        let key1 = wallet.derive_account(0).unwrap();
        let key2 = wallet.derive_account(1).unwrap();
        
        assert_ne!(key1.address(), key2.address());
    }
}
