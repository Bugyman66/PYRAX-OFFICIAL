use serde::{Deserialize, Serialize};
use tiny_keccak::{Hasher, Keccak};
use k256::ecdsa::{SigningKey, Signature, RecoveryId, VerifyingKey};
use k256::ecdsa::signature::Signer;

use super::{Address, H256, Nonce, Gas, Wei, ChainId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum TxType {
    Transfer = 0,
    ContractDeploy = 1,
    ContractCall = 2,
    AiJobSubmit = 3,
    AiJobClaim = 4,
    AiJobComplete = 5,
    ModelRegister = 6,
    Stake = 7,
    Unstake = 8,
}

impl TxType {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(TxType::Transfer),
            1 => Some(TxType::ContractDeploy),
            2 => Some(TxType::ContractCall),
            3 => Some(TxType::AiJobSubmit),
            4 => Some(TxType::AiJobClaim),
            5 => Some(TxType::AiJobComplete),
            6 => Some(TxType::ModelRegister),
            7 => Some(TxType::Stake),
            8 => Some(TxType::Unstake),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub version: u16,
    pub tx_type: TxType,
    pub chain_id: ChainId,
    pub nonce: Nonce,
    pub gas_price: Gas,
    pub gas_limit: Gas,
    pub to: Option<Address>,
    pub value: Wei,
    pub data: Vec<u8>,
}

impl Transaction {
    pub fn new_transfer(
        chain_id: ChainId,
        nonce: Nonce,
        gas_price: Gas,
        gas_limit: Gas,
        to: Address,
        value: Wei,
    ) -> Self {
        Self {
            version: 1,
            tx_type: TxType::Transfer,
            chain_id,
            nonce,
            gas_price,
            gas_limit,
            to: Some(to),
            value,
            data: vec![],
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(256);
        buf.extend_from_slice(&self.version.to_le_bytes());
        buf.push(self.tx_type as u8);
        buf.extend_from_slice(&self.chain_id.0.to_le_bytes());
        buf.extend_from_slice(&self.nonce.to_le_bytes());
        buf.extend_from_slice(&self.gas_price.to_le_bytes());
        buf.extend_from_slice(&self.gas_limit.to_le_bytes());
        
        if let Some(to) = &self.to {
            buf.push(1);
            buf.extend_from_slice(to.as_bytes());
        } else {
            buf.push(0);
        }
        
        let mut value_bytes = [0u8; 32];
        self.value.to_little_endian(&mut value_bytes);
        buf.extend_from_slice(&value_bytes);
        
        buf.extend_from_slice(&(self.data.len() as u32).to_le_bytes());
        buf.extend_from_slice(&self.data);
        
        buf
    }

    pub fn signing_hash(&self) -> H256 {
        let encoded = self.encode();
        let mut hasher = Keccak::v256();
        let mut output = [0u8; 32];
        hasher.update(&encoded);
        hasher.finalize(&mut output);
        H256::from_slice(&output)
    }

    pub fn sign(self, key: &SigningKey) -> SignedTransaction {
        let hash = self.signing_hash();
        let (signature, recovery_id) = key.sign_recoverable(hash.as_bytes()).unwrap();
        
        let sig_bytes = signature.to_bytes();
        let mut r = [0u8; 32];
        let mut s = [0u8; 32];
        r.copy_from_slice(&sig_bytes[0..32]);
        s.copy_from_slice(&sig_bytes[32..64]);
        
        SignedTransaction {
            tx: self,
            r: H256::from_slice(&r),
            s: H256::from_slice(&s),
            v: recovery_id.to_byte(),
        }
    }

    pub fn intrinsic_gas(&self) -> Gas {
        let base_gas: Gas = match self.tx_type {
            TxType::Transfer => 21_000,
            TxType::ContractDeploy => 32_000,
            TxType::ContractCall => 21_000,
            TxType::AiJobSubmit => 50_000,
            TxType::AiJobClaim => 30_000,
            TxType::AiJobComplete => 40_000,
            TxType::ModelRegister => 75_000,
            TxType::Stake => 25_000,
            TxType::Unstake => 25_000,
        };
        
        let data_gas: Gas = self.data.iter().map(|&byte| {
            if byte == 0 { 4 } else { 16 }
        }).sum();
        
        base_gas + data_gas
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedTransaction {
    pub tx: Transaction,
    pub r: H256,
    pub s: H256,
    pub v: u8,
}

impl SignedTransaction {
    pub fn hash(&self) -> H256 {
        let mut buf = self.tx.encode();
        buf.extend_from_slice(self.r.as_bytes());
        buf.extend_from_slice(self.s.as_bytes());
        buf.push(self.v);
        
        let mut hasher = Keccak::v256();
        let mut output = [0u8; 32];
        hasher.update(&buf);
        hasher.finalize(&mut output);
        H256::from_slice(&output)
    }

    pub fn recover_sender(&self) -> Option<Address> {
        let hash = self.tx.signing_hash();
        
        let mut sig_bytes = [0u8; 64];
        sig_bytes[0..32].copy_from_slice(self.r.as_bytes());
        sig_bytes[32..64].copy_from_slice(self.s.as_bytes());
        
        let signature = Signature::from_bytes((&sig_bytes).into()).ok()?;
        let recovery_id = RecoveryId::from_byte(self.v)?;
        
        let verifying_key = VerifyingKey::recover_from_prehash(
            hash.as_bytes(),
            &signature,
            recovery_id,
        ).ok()?;
        
        let pubkey_bytes = verifying_key.to_encoded_point(false);
        let pubkey_uncompressed = pubkey_bytes.as_bytes();
        
        Some(Address::from_public_key(&pubkey_uncompressed[1..]))
    }

    pub fn verify(&self) -> bool {
        self.recover_sender().is_some()
    }

    pub fn fee(&self) -> Wei {
        Wei::from(self.tx.gas_price) * Wei::from(self.tx.gas_limit)
    }

    pub fn sender(&self) -> Option<Address> {
        self.recover_sender()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::OsRng;

    #[test]
    fn test_sign_and_verify() {
        let key = SigningKey::random(&mut OsRng);
        let tx = Transaction::new_transfer(
            ChainId::DEVNET,
            0,
            1_000_000_000,
            21_000,
            Address::ZERO,
            Wei::from(1_000_000_000_000_000_000u64),
        );
        
        let signed = tx.sign(&key);
        assert!(signed.verify());
        
        let sender = signed.recover_sender().unwrap();
        let verifying_key = key.verifying_key();
        let expected = Address::from_public_key(
            &verifying_key.to_encoded_point(false).as_bytes()[1..]
        );
        assert_eq!(sender, expected);
    }
}
