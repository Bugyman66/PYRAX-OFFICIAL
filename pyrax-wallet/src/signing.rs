use k256::ecdsa::{Signature, RecoveryId, VerifyingKey, signature::hazmat::PrehashSigner};
use tiny_keccak::{Hasher, Keccak};

use crate::{Result, WalletError, PrivateKey, PublicKey, Address};

#[derive(Debug, Clone)]
pub struct SignedMessage {
    pub message_hash: [u8; 32],
    pub r: [u8; 32],
    pub s: [u8; 32],
    pub v: u8,
}

impl SignedMessage {
    pub fn to_bytes(&self) -> [u8; 65] {
        let mut bytes = [0u8; 65];
        bytes[0..32].copy_from_slice(&self.r);
        bytes[32..64].copy_from_slice(&self.s);
        bytes[64] = self.v;
        bytes
    }

    pub fn from_bytes(bytes: &[u8; 65]) -> Self {
        let mut r = [0u8; 32];
        let mut s = [0u8; 32];
        r.copy_from_slice(&bytes[0..32]);
        s.copy_from_slice(&bytes[32..64]);
        
        Self {
            message_hash: [0u8; 32], // Not stored in bytes
            r,
            s,
            v: bytes[64],
        }
    }

    pub fn recover_public_key(&self) -> Result<PublicKey> {
        let mut sig_bytes = [0u8; 64];
        sig_bytes[0..32].copy_from_slice(&self.r);
        sig_bytes[32..64].copy_from_slice(&self.s);
        
        let signature = Signature::from_bytes((&sig_bytes).into())
            .map_err(|e| WalletError::SigningError(e.to_string()))?;
        
        let recovery_id = RecoveryId::from_byte(self.v)
            .ok_or_else(|| WalletError::SigningError("Invalid recovery ID".into()))?;
        
        let verifying_key = VerifyingKey::recover_from_prehash(
            &self.message_hash,
            &signature,
            recovery_id,
        ).map_err(|e| WalletError::SigningError(e.to_string()))?;
        
        Ok(PublicKey::from_verifying_key(&verifying_key))
    }

    pub fn recover_address(&self) -> Result<Address> {
        let pubkey = self.recover_public_key()?;
        Ok(pubkey.address())
    }
}

pub fn sign_message(key: &PrivateKey, message: &[u8]) -> Result<SignedMessage> {
    // Ethereum-style message signing
    let prefix = format!("\x19Ethereum Signed Message:\n{}", message.len());
    
    let mut hasher = Keccak::v256();
    let mut hash = [0u8; 32];
    hasher.update(prefix.as_bytes());
    hasher.update(message);
    hasher.finalize(&mut hash);
    
    sign_hash(key, &hash)
}

pub fn sign_hash(key: &PrivateKey, hash: &[u8; 32]) -> Result<SignedMessage> {
    let (signature, recovery_id): (Signature, RecoveryId) = key.signing_key()
        .sign_prehash_recoverable(hash)
        .map_err(|e| WalletError::SigningError(e.to_string()))?;
    
    let sig_bytes = signature.to_bytes();
    let mut r = [0u8; 32];
    let mut s = [0u8; 32];
    r.copy_from_slice(&sig_bytes[0..32]);
    s.copy_from_slice(&sig_bytes[32..64]);
    
    Ok(SignedMessage {
        message_hash: *hash,
        r,
        s,
        v: recovery_id.to_byte(),
    })
}

pub fn sign_transaction(
    key: &PrivateKey,
    tx_hash: &[u8; 32],
) -> Result<SignedMessage> {
    sign_hash(key, tx_hash)
}

pub fn verify_signature(
    message: &[u8],
    signature: &SignedMessage,
    expected_address: &Address,
) -> Result<bool> {
    let recovered = signature.recover_address()?;
    Ok(&recovered == expected_address)
}

pub fn hash_message(message: &[u8]) -> [u8; 32] {
    let prefix = format!("\x19Ethereum Signed Message:\n{}", message.len());
    
    let mut hasher = Keccak::v256();
    let mut hash = [0u8; 32];
    hasher.update(prefix.as_bytes());
    hasher.update(message);
    hasher.finalize(&mut hash);
    
    hash
}

pub fn keccak256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak::v256();
    let mut hash = [0u8; 32];
    hasher.update(data);
    hasher.finalize(&mut hash);
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_and_verify() {
        let key = PrivateKey::random();
        let message = b"Hello, PYRAX!";
        
        let signature = sign_message(&key, message).unwrap();
        let recovered = signature.recover_address().unwrap();
        
        assert_eq!(recovered, key.address());
    }

    #[test]
    fn test_sign_hash() {
        let key = PrivateKey::random();
        let hash = keccak256(b"test data");
        
        let signature = sign_hash(&key, &hash).unwrap();
        let recovered = signature.recover_address().unwrap();
        
        assert_eq!(recovered, key.address());
    }

    #[test]
    fn test_signature_bytes_roundtrip() {
        let key = PrivateKey::random();
        let message = b"Test message";
        
        let signature = sign_message(&key, message).unwrap();
        let bytes = signature.to_bytes();
        let restored = SignedMessage::from_bytes(&bytes);
        
        assert_eq!(signature.r, restored.r);
        assert_eq!(signature.s, restored.s);
        assert_eq!(signature.v, restored.v);
    }
}
