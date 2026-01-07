pub mod keys;
pub mod signing;
pub mod addresses;
pub mod hd;
pub mod keystore;
pub mod error;

pub use keys::{PrivateKey, PublicKey};
pub use signing::{sign_transaction, sign_message, verify_signature};
pub use addresses::Address;
pub use hd::{Mnemonic, HDWallet, DerivationPath};
pub use keystore::KeyStore;
pub use error::WalletError;

pub type Result<T> = std::result::Result<T, WalletError>;
