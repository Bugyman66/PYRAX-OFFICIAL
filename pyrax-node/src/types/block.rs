use serde::{Deserialize, Serialize};
use tiny_keccak::{Hasher, Keccak};

use super::{Address, Hash, H256, BlockNumber, Wei, SignedTransaction};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub version: u32,
    pub parent_hash: H256,
    pub merkle_root: H256,
    pub state_root: H256,
    pub timestamp: u64,
    pub difficulty: u64,
    pub nonce: u64,
    pub height: BlockNumber,
    pub extra_nonce: u64,
    pub beneficiary: Address,
}

impl BlockHeader {
    pub fn hash(&self) -> H256 {
        let encoded = self.encode_for_hash();
        let mut hasher = Keccak::v256();
        let mut output = [0u8; 32];
        hasher.update(&encoded);
        hasher.finalize(&mut output);
        H256::from_slice(&output)
    }

    pub fn encode_for_hash(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(200);
        buf.extend_from_slice(&self.version.to_le_bytes());
        buf.extend_from_slice(self.parent_hash.as_bytes());
        buf.extend_from_slice(self.merkle_root.as_bytes());
        buf.extend_from_slice(self.state_root.as_bytes());
        buf.extend_from_slice(&self.timestamp.to_le_bytes());
        buf.extend_from_slice(&self.difficulty.to_le_bytes());
        buf.extend_from_slice(&self.nonce.to_le_bytes());
        buf.extend_from_slice(&self.height.to_le_bytes());
        buf.extend_from_slice(&self.extra_nonce.to_le_bytes());
        buf.extend_from_slice(self.beneficiary.as_bytes());
        buf
    }

    pub fn genesis(chain_id: u32) -> Self {
        Self {
            version: 1,
            parent_hash: H256::zero(),
            merkle_root: H256::zero(),
            state_root: H256::zero(),
            timestamp: 1704067200, // 2024-01-01 00:00:00 UTC
            difficulty: 1_000_000,
            nonce: 0,
            height: 0,
            extra_nonce: 0,
            beneficiary: Address::ZERO,
        }
    }

    pub fn meets_difficulty(&self, hash: &H256) -> bool {
        let target = Self::difficulty_to_target(self.difficulty);
        hash.as_bytes() <= target.as_bytes()
    }

    pub fn difficulty_to_target(difficulty: u64) -> H256 {
        if difficulty == 0 {
            return H256::from_slice(&[0xff; 32]);
        }
        
        let max_target = ethereum_types::U256::from_str_radix(
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            16
        ).unwrap();
        
        let target = max_target / ethereum_types::U256::from(difficulty);
        let mut bytes = [0u8; 32];
        target.to_big_endian(&mut bytes);
        H256::from_slice(&bytes)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockBody {
    pub transactions: Vec<SignedTransaction>,
}

impl BlockBody {
    pub fn new(transactions: Vec<SignedTransaction>) -> Self {
        Self { transactions }
    }

    pub fn empty() -> Self {
        Self { transactions: vec![] }
    }

    pub fn merkle_root(&self) -> H256 {
        if self.transactions.is_empty() {
            return H256::zero();
        }

        let mut hashes: Vec<H256> = self.transactions
            .iter()
            .map(|tx| tx.hash())
            .collect();

        while hashes.len() > 1 {
            if hashes.len() % 2 == 1 {
                hashes.push(*hashes.last().unwrap());
            }

            hashes = hashes
                .chunks(2)
                .map(|pair| {
                    let mut hasher = Keccak::v256();
                    let mut output = [0u8; 32];
                    hasher.update(pair[0].as_bytes());
                    hasher.update(pair[1].as_bytes());
                    hasher.finalize(&mut output);
                    H256::from_slice(&output)
                })
                .collect();
        }

        hashes[0]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub body: BlockBody,
}

impl Block {
    pub fn new(header: BlockHeader, body: BlockBody) -> Self {
        Self { header, body }
    }

    pub fn hash(&self) -> H256 {
        self.header.hash()
    }

    pub fn height(&self) -> BlockNumber {
        self.header.height
    }

    pub fn genesis(chain_id: u32) -> Self {
        Self {
            header: BlockHeader::genesis(chain_id),
            body: BlockBody::empty(),
        }
    }

    pub fn total_fees(&self) -> Wei {
        self.body.transactions
            .iter()
            .map(|tx| tx.fee())
            .fold(Wei::zero(), |acc, fee| acc + fee)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_block() {
        let genesis = Block::genesis(1);
        assert_eq!(genesis.height(), 0);
        assert_eq!(genesis.header.parent_hash, H256::zero());
    }

    #[test]
    fn test_merkle_root_empty() {
        let body = BlockBody::empty();
        assert_eq!(body.merkle_root(), H256::zero());
    }
}
