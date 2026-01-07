use thiserror::Error;

#[derive(Error, Debug)]
pub enum WalletError {
    #[error("Invalid mnemonic: {0}")]
    InvalidMnemonic(String),

    #[error("Invalid private key: {0}")]
    InvalidPrivateKey(String),

    #[error("Invalid public key: {0}")]
    InvalidPublicKey(String),

    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Invalid derivation path: {0}")]
    InvalidDerivationPath(String),

    #[error("Signing error: {0}")]
    SigningError(String),

    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("Decryption error: {0}")]
    DecryptionError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Key not found: {0}")]
    KeyNotFound(String),

    #[error("Invalid password")]
    InvalidPassword,

    #[error("Keystore locked")]
    KeystoreLocked,
}
