use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use serde::{Deserialize, Serialize};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, KeyInit};
use argon2::{Argon2, password_hash::SaltString};
use rand::RngCore;

use crate::{Result, WalletError, PrivateKey, Address, Mnemonic, HDWallet};

pub struct KeyStore {
    path: PathBuf,
    keys: HashMap<Address, EncryptedKey>,
    master_key: Option<[u8; 32]>,
    hd_wallet: Option<HDWallet>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EncryptedKey {
    address: String,
    crypto: CryptoParams,
    id: String,
    version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CryptoParams {
    cipher: String,
    ciphertext: String,
    cipherparams: CipherParams,
    kdf: String,
    kdfparams: KdfParams,
    mac: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CipherParams {
    iv: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct KdfParams {
    salt: String,
    n: u32,
    r: u32,
    p: u32,
    dklen: u32,
}

impl KeyStore {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        
        if !path.exists() {
            fs::create_dir_all(&path)?;
        }

        let mut store = Self {
            path,
            keys: HashMap::new(),
            master_key: None,
            hd_wallet: None,
        };

        store.load_keys()?;
        Ok(store)
    }

    fn load_keys(&mut self) -> Result<()> {
        if !self.path.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(&self.path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().map_or(false, |e| e == "json") {
                if let Ok(contents) = fs::read_to_string(&path) {
                    if let Ok(encrypted) = serde_json::from_str::<EncryptedKey>(&contents) {
                        if let Ok(addr) = Address::from_hex(&encrypted.address) {
                            self.keys.insert(addr, encrypted);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn create_key(&mut self, password: &str) -> Result<Address> {
        let key = PrivateKey::random();
        self.import_key(&key, password)
    }

    pub fn import_key(&mut self, key: &PrivateKey, password: &str) -> Result<Address> {
        let address = key.address();
        let encrypted = encrypt_key(key, password)?;
        
        // Save to file
        let filename = format!("UTC--{}--{}.json", 
            chrono_lite_timestamp(),
            address.to_hex()
        );
        let filepath = self.path.join(&filename);
        
        let json = serde_json::to_string_pretty(&encrypted)
            .map_err(|e| WalletError::SerializationError(e.to_string()))?;
        fs::write(&filepath, json)?;
        
        self.keys.insert(address, encrypted);
        Ok(address)
    }

    pub fn import_mnemonic(&mut self, mnemonic: &Mnemonic, password: &str, passphrase: &str) -> Result<Address> {
        let wallet = mnemonic.to_hd_wallet(passphrase);
        let key = wallet.derive_account(0)?;
        let address = self.import_key(&key, password)?;
        self.hd_wallet = Some(wallet);
        Ok(address)
    }

    pub fn get_key(&self, address: &Address, password: &str) -> Result<PrivateKey> {
        let encrypted = self.keys.get(address)
            .ok_or_else(|| WalletError::KeyNotFound(address.to_string()))?;
        
        decrypt_key(encrypted, password)
    }

    pub fn list_addresses(&self) -> Vec<Address> {
        self.keys.keys().cloned().collect()
    }

    pub fn contains(&self, address: &Address) -> bool {
        self.keys.contains_key(address)
    }

    pub fn remove(&mut self, address: &Address) -> Result<()> {
        if !self.keys.contains_key(address) {
            return Err(WalletError::KeyNotFound(address.to_string()));
        }

        // Find and remove the file
        for entry in fs::read_dir(&self.path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().map_or(false, |e| e == "json") {
                let filename = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                
                if filename.contains(&address.to_hex()) {
                    fs::remove_file(&path)?;
                    break;
                }
            }
        }

        self.keys.remove(address);
        Ok(())
    }

    pub fn unlock(&mut self, password: &str) -> Result<()> {
        // Derive master key from password
        let salt = SaltString::generate(&mut rand::thread_rng());
        let mut master_key = [0u8; 32];
        
        Argon2::default()
            .hash_password_into(
                password.as_bytes(),
                salt.as_str().as_bytes(),
                &mut master_key,
            )
            .map_err(|e| WalletError::EncryptionError(e.to_string()))?;
        
        self.master_key = Some(master_key);
        Ok(())
    }

    pub fn lock(&mut self) {
        if let Some(mut key) = self.master_key.take() {
            key.iter_mut().for_each(|b| *b = 0);
        }
    }

    pub fn is_locked(&self) -> bool {
        self.master_key.is_none()
    }

    pub fn derive_next_address(&mut self, password: &str) -> Result<Address> {
        let wallet = self.hd_wallet.as_ref()
            .ok_or(WalletError::KeystoreLocked)?;
        
        let next_index = self.keys.len() as u32;
        let key = wallet.derive_path(&crate::hd::DerivationPath::pyrax(0, 0, next_index))?;
        self.import_key(&key, password)
    }
}

fn encrypt_key(key: &PrivateKey, password: &str) -> Result<EncryptedKey> {
    let mut salt = [0u8; 32];
    let mut iv = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut salt);
    rand::thread_rng().fill_bytes(&mut iv);

    // Derive encryption key using Argon2
    let mut derived_key = [0u8; 32];
    Argon2::default()
        .hash_password_into(
            password.as_bytes(),
            &salt,
            &mut derived_key,
        )
        .map_err(|e| WalletError::EncryptionError(e.to_string()))?;

    // Encrypt private key
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&derived_key));
    let nonce = Nonce::from_slice(&iv);
    
    let ciphertext = cipher.encrypt(nonce, key.to_bytes().as_ref())
        .map_err(|e| WalletError::EncryptionError(e.to_string()))?;

    // Calculate MAC
    let mut mac_input = derived_key[16..32].to_vec();
    mac_input.extend_from_slice(&ciphertext);
    let mac = crate::signing::keccak256(&mac_input);

    Ok(EncryptedKey {
        address: key.address().to_hex(),
        crypto: CryptoParams {
            cipher: "aes-256-gcm".to_string(),
            ciphertext: hex::encode(&ciphertext),
            cipherparams: CipherParams {
                iv: hex::encode(&iv),
            },
            kdf: "argon2id".to_string(),
            kdfparams: KdfParams {
                salt: hex::encode(&salt),
                n: 65536,
                r: 8,
                p: 1,
                dklen: 32,
            },
            mac: hex::encode(&mac),
        },
        id: uuid_v4(),
        version: 3,
    })
}

fn decrypt_key(encrypted: &EncryptedKey, password: &str) -> Result<PrivateKey> {
    let salt = hex::decode(&encrypted.crypto.kdfparams.salt)
        .map_err(|_| WalletError::DecryptionError("Invalid salt".into()))?;
    let iv = hex::decode(&encrypted.crypto.cipherparams.iv)
        .map_err(|_| WalletError::DecryptionError("Invalid IV".into()))?;
    let ciphertext = hex::decode(&encrypted.crypto.ciphertext)
        .map_err(|_| WalletError::DecryptionError("Invalid ciphertext".into()))?;
    let expected_mac = hex::decode(&encrypted.crypto.mac)
        .map_err(|_| WalletError::DecryptionError("Invalid MAC".into()))?;

    // Derive key
    let mut derived_key = [0u8; 32];
    Argon2::default()
        .hash_password_into(
            password.as_bytes(),
            &salt,
            &mut derived_key,
        )
        .map_err(|e| WalletError::DecryptionError(e.to_string()))?;

    // Verify MAC
    let mut mac_input = derived_key[16..32].to_vec();
    mac_input.extend_from_slice(&ciphertext);
    let computed_mac = crate::signing::keccak256(&mac_input);
    
    if computed_mac[..] != expected_mac[..] {
        return Err(WalletError::InvalidPassword);
    }

    // Decrypt
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&derived_key));
    let nonce = Nonce::from_slice(&iv);
    
    let plaintext = cipher.decrypt(nonce, ciphertext.as_ref())
        .map_err(|_| WalletError::InvalidPassword)?;

    PrivateKey::from_bytes(&plaintext)
}

fn chrono_lite_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap();
    format!("{}", duration.as_secs())
}

fn uuid_v4() -> String {
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    
    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        u16::from_be_bytes([bytes[4], bytes[5]]),
        u16::from_be_bytes([bytes[6], bytes[7]]),
        u16::from_be_bytes([bytes[8], bytes[9]]),
        u64::from_be_bytes([0, 0, bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]])
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_create_and_retrieve_key() {
        let dir = tempdir().unwrap();
        let mut store = KeyStore::new(dir.path()).unwrap();
        
        let password = "test_password_123";
        let address = store.create_key(password).unwrap();
        
        let key = store.get_key(&address, password).unwrap();
        assert_eq!(key.address(), address);
    }

    #[test]
    fn test_wrong_password() {
        let dir = tempdir().unwrap();
        let mut store = KeyStore::new(dir.path()).unwrap();
        
        let address = store.create_key("correct_password").unwrap();
        let result = store.get_key(&address, "wrong_password");
        
        assert!(matches!(result, Err(WalletError::InvalidPassword)));
    }

    #[test]
    fn test_import_key() {
        let dir = tempdir().unwrap();
        let mut store = KeyStore::new(dir.path()).unwrap();
        
        let key = PrivateKey::random();
        let original_address = key.address();
        
        let address = store.import_key(&key, "password").unwrap();
        assert_eq!(address, original_address);
        
        let retrieved = store.get_key(&address, "password").unwrap();
        assert_eq!(retrieved.address(), original_address);
    }
}
