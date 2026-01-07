//! Wallet management module for PYRAX Desktop
//!
//! Integrates with pyrax-wallet library for key management and signing.

use std::path::PathBuf;
use pyrax_wallet::{KeyStore, Mnemonic, PrivateKey, Address, HDWallet};
use tracing::{info, warn};

/// Wallet manager wrapping the pyrax-wallet keystore
pub struct WalletManager {
    keystore: Option<KeyStore>,
    hd_wallet: Option<HDWallet>,
    data_dir: PathBuf,
    unlocked: bool,
}

impl WalletManager {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            keystore: None,
            hd_wallet: None,
            data_dir,
            unlocked: false,
        }
    }

    /// Initialize or load the keystore
    pub fn init(&mut self) -> Result<(), String> {
        let keystore_path = self.data_dir.join("keystore");
        
        let keystore = KeyStore::new(&keystore_path)
            .map_err(|e| format!("Failed to initialize keystore: {}", e))?;
        
        self.keystore = Some(keystore);
        info!("Keystore initialized at {:?}", keystore_path);
        Ok(())
    }

    /// Create a new wallet with a random mnemonic
    pub fn create(&mut self, password: &str) -> Result<String, String> {
        let mnemonic = Mnemonic::generate(24)
            .map_err(|e| format!("Failed to generate mnemonic: {}", e))?;
        
        let phrase = mnemonic.phrase().to_string();
        
        // Import the mnemonic
        self.import_mnemonic(&phrase, password, "")?;
        
        Ok(phrase)
    }

    /// Import wallet from mnemonic phrase
    pub fn import_mnemonic(&mut self, phrase: &str, password: &str, passphrase: &str) -> Result<Address, String> {
        let mnemonic = Mnemonic::from_phrase(phrase)
            .map_err(|e| format!("Invalid mnemonic: {}", e))?;
        
        let keystore = self.keystore.as_mut()
            .ok_or("Keystore not initialized")?;
        
        let address = keystore.import_mnemonic(&mnemonic, password, passphrase)
            .map_err(|e| format!("Failed to import mnemonic: {}", e))?;
        
        self.hd_wallet = Some(mnemonic.to_hd_wallet(passphrase));
        self.unlocked = true;
        
        info!("Wallet imported: {}", address);
        Ok(address)
    }

    /// Unlock the wallet with password
    pub fn unlock(&mut self, password: &str) -> Result<(), String> {
        let keystore = self.keystore.as_mut()
            .ok_or("Keystore not initialized")?;
        
        keystore.unlock(password)
            .map_err(|e| format!("Failed to unlock: {}", e))?;
        
        self.unlocked = true;
        info!("Wallet unlocked");
        Ok(())
    }

    /// Lock the wallet
    pub fn lock(&mut self) {
        if let Some(keystore) = self.keystore.as_mut() {
            keystore.lock();
        }
        self.unlocked = false;
        info!("Wallet locked");
    }

    /// Check if wallet is unlocked
    pub fn is_unlocked(&self) -> bool {
        self.unlocked
    }

    /// Get all addresses in the keystore
    pub fn get_addresses(&self) -> Result<Vec<Address>, String> {
        let keystore = self.keystore.as_ref()
            .ok_or("Keystore not initialized")?;
        
        Ok(keystore.list_addresses())
    }

    /// Create a new address
    pub fn create_address(&mut self, password: &str) -> Result<Address, String> {
        let keystore = self.keystore.as_mut()
            .ok_or("Keystore not initialized")?;
        
        if let Some(ref hd_wallet) = self.hd_wallet {
            keystore.derive_next_address(password)
                .map_err(|e| format!("Failed to derive address: {}", e))
        } else {
            keystore.create_key(password)
                .map_err(|e| format!("Failed to create address: {}", e))
        }
    }

    /// Get private key for an address (requires unlock)
    pub fn get_key(&self, address: &Address, password: &str) -> Result<PrivateKey, String> {
        if !self.unlocked {
            return Err("Wallet is locked".to_string());
        }
        
        let keystore = self.keystore.as_ref()
            .ok_or("Keystore not initialized")?;
        
        keystore.get_key(address, password)
            .map_err(|e| format!("Failed to get key: {}", e))
    }

    /// Check if keystore contains an address
    pub fn has_address(&self, address: &Address) -> bool {
        self.keystore.as_ref()
            .map(|ks| ks.contains(address))
            .unwrap_or(false)
    }
}
