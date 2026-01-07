//! Cross-Model Transactions
//!
//! Atomic transactions that span both UTXO and Account models

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use blake3::Hasher;

use crate::types::{Address, H256, OutPoint};
use super::unified_address::UnifiedAddress;

/// Cross-model transaction type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrossModelTxType {
    /// Lock UTXO, mint EVM tokens
    LockAndMint,
    /// Burn EVM tokens, unlock UTXO
    BurnAndUnlock,
    /// Atomic swap between UTXO and EVM
    AtomicSwap,
    /// Split: UTXO to multiple EVM addresses
    SplitToEvm,
    /// Merge: multiple EVM balances to single UTXO
    MergeToUtxo,
}

/// Cross-model transaction status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrossModelTxStatus {
    /// Transaction created, not yet submitted
    Created,
    /// Submitted to source chain
    Submitted,
    /// Confirmed on source chain
    SourceConfirmed,
    /// Processing on destination chain
    Processing,
    /// Confirmed on destination chain
    DestConfirmed,
    /// Fully complete
    Complete,
    /// Failed, needs refund
    Failed,
    /// Refunded
    Refunded,
    /// Expired (timeout)
    Expired,
}

/// UTXO input for cross-model tx
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtxoInput {
    /// Outpoint being spent
    pub outpoint: OutPoint,
    /// Amount
    pub amount: u64,
    /// Owner address
    pub owner: Address,
    /// Signature (serialized)
    pub signature: Vec<u8>,
}

/// UTXO output for cross-model tx
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtxoOutput {
    /// Recipient address
    pub recipient: Address,
    /// Amount
    pub amount: u64,
    /// Script/lock type
    pub lock_type: LockType,
}

/// Lock type for UTXO outputs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LockType {
    /// Standard P2PKH
    Standard,
    /// Bridge lock (for conversion)
    BridgeLock,
    /// Timelock
    TimeLock { unlock_time: u64 },
    /// Hashlock (for atomic swaps)
    HashLock,
    /// Multi-sig
    MultiSig { required: u8, total: u8 },
}

/// EVM transfer for cross-model tx
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmTransfer {
    /// From address
    pub from: [u8; 20],
    /// To address
    pub to: [u8; 20],
    /// Amount
    pub amount: u64,
    /// Data (for contract calls)
    pub data: Vec<u8>,
    /// Gas limit
    pub gas_limit: u64,
}

/// Cross-model transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossModelTransaction {
    /// Transaction ID
    pub id: H256,
    /// Transaction type
    pub tx_type: CrossModelTxType,
    /// Current status
    pub status: CrossModelTxStatus,
    /// UTXO inputs (for UTXO → EVM)
    pub utxo_inputs: Vec<UtxoInput>,
    /// UTXO outputs (for EVM → UTXO)
    pub utxo_outputs: Vec<UtxoOutput>,
    /// EVM transfers
    pub evm_transfers: Vec<EvmTransfer>,
    /// Total UTXO input amount
    pub utxo_input_amount: u64,
    /// Total UTXO output amount
    pub utxo_output_amount: u64,
    /// Total EVM transfer amount
    pub evm_transfer_amount: u64,
    /// Fee paid
    pub fee: u64,
    /// Source chain tx hash
    pub source_tx_hash: Option<H256>,
    /// Destination chain tx hash
    pub dest_tx_hash: Option<H256>,
    /// Source chain confirmations
    pub source_confirmations: u64,
    /// Destination chain confirmations
    pub dest_confirmations: u64,
    /// Required source confirmations
    pub required_source_confirmations: u64,
    /// Required destination confirmations
    pub required_dest_confirmations: u64,
    /// Creation timestamp
    pub created_at: u64,
    /// Expiry timestamp
    pub expires_at: u64,
    /// Completion timestamp
    pub completed_at: Option<u64>,
    /// Error message if failed
    pub error: Option<String>,
    /// Initiator unified address
    pub initiator: UnifiedAddress,
}

impl CrossModelTransaction {
    /// Create a Lock-and-Mint transaction (UTXO → EVM)
    pub fn lock_and_mint(
        utxo_inputs: Vec<UtxoInput>,
        evm_recipient: [u8; 20],
        initiator: UnifiedAddress,
    ) -> Self {
        let utxo_input_amount: u64 = utxo_inputs.iter().map(|i| i.amount).sum();
        let fee = Self::calculate_fee(utxo_input_amount);
        let net_amount = utxo_input_amount.saturating_sub(fee);

        let evm_transfers = vec![EvmTransfer {
            from: [0u8; 20], // Bridge address
            to: evm_recipient,
            amount: net_amount,
            data: Vec::new(),
            gas_limit: 21000,
        }];

        let id = Self::generate_id(&utxo_inputs, &evm_transfers);

        Self {
            id,
            tx_type: CrossModelTxType::LockAndMint,
            status: CrossModelTxStatus::Created,
            utxo_inputs,
            utxo_outputs: Vec::new(),
            evm_transfers,
            utxo_input_amount,
            utxo_output_amount: 0,
            evm_transfer_amount: net_amount,
            fee,
            source_tx_hash: None,
            dest_tx_hash: None,
            source_confirmations: 0,
            dest_confirmations: 0,
            required_source_confirmations: 6,
            required_dest_confirmations: 12,
            created_at: current_timestamp(),
            expires_at: current_timestamp() + 86400, // 24 hours
            completed_at: None,
            error: None,
            initiator,
        }
    }

    /// Create a Burn-and-Unlock transaction (EVM → UTXO)
    pub fn burn_and_unlock(
        evm_sender: [u8; 20],
        amount: u64,
        utxo_recipient: Address,
        initiator: UnifiedAddress,
    ) -> Self {
        let fee = Self::calculate_fee(amount);
        let net_amount = amount.saturating_sub(fee);

        let evm_transfers = vec![EvmTransfer {
            from: evm_sender,
            to: [0u8; 20], // Burn address
            amount,
            data: Vec::new(),
            gas_limit: 50000,
        }];

        let utxo_outputs = vec![UtxoOutput {
            recipient: utxo_recipient,
            amount: net_amount,
            lock_type: LockType::Standard,
        }];

        let id = Self::generate_id_from_evm(&evm_transfers, &utxo_outputs);

        Self {
            id,
            tx_type: CrossModelTxType::BurnAndUnlock,
            status: CrossModelTxStatus::Created,
            utxo_inputs: Vec::new(),
            utxo_outputs,
            evm_transfers,
            utxo_input_amount: 0,
            utxo_output_amount: net_amount,
            evm_transfer_amount: amount,
            fee,
            source_tx_hash: None,
            dest_tx_hash: None,
            source_confirmations: 0,
            dest_confirmations: 0,
            required_source_confirmations: 12,
            required_dest_confirmations: 6,
            created_at: current_timestamp(),
            expires_at: current_timestamp() + 86400,
            completed_at: None,
            error: None,
            initiator,
        }
    }

    /// Calculate fee (0.1%)
    fn calculate_fee(amount: u64) -> u64 {
        (amount as u128 * 10 / 10000) as u64
    }

    /// Generate transaction ID from UTXO inputs
    fn generate_id(utxo_inputs: &[UtxoInput], evm_transfers: &[EvmTransfer]) -> H256 {
        let mut hasher = Hasher::new();
        for input in utxo_inputs {
            hasher.update(&input.outpoint.txid.0);
            hasher.update(&input.outpoint.vout.to_be_bytes());
        }
        for transfer in evm_transfers {
            hasher.update(&transfer.to);
            hasher.update(&transfer.amount.to_be_bytes());
        }
        hasher.update(&current_timestamp().to_be_bytes());
        H256::from_slice(hasher.finalize().as_bytes())
    }

    /// Generate transaction ID from EVM transfers
    fn generate_id_from_evm(evm_transfers: &[EvmTransfer], utxo_outputs: &[UtxoOutput]) -> H256 {
        let mut hasher = Hasher::new();
        for transfer in evm_transfers {
            hasher.update(&transfer.from);
            hasher.update(&transfer.amount.to_be_bytes());
        }
        for output in utxo_outputs {
            hasher.update(&output.recipient.0);
            hasher.update(&output.amount.to_be_bytes());
        }
        hasher.update(&current_timestamp().to_be_bytes());
        H256::from_slice(hasher.finalize().as_bytes())
    }

    /// Check if transaction is expired
    pub fn is_expired(&self) -> bool {
        current_timestamp() > self.expires_at
    }

    /// Check if source is confirmed
    pub fn is_source_confirmed(&self) -> bool {
        self.source_confirmations >= self.required_source_confirmations
    }

    /// Check if destination is confirmed
    pub fn is_dest_confirmed(&self) -> bool {
        self.dest_confirmations >= self.required_dest_confirmations
    }

    /// Update source confirmations
    pub fn update_source_confirmations(&mut self, confirmations: u64) {
        self.source_confirmations = confirmations;
        if self.is_source_confirmed() && self.status == CrossModelTxStatus::Submitted {
            self.status = CrossModelTxStatus::SourceConfirmed;
        }
    }

    /// Update destination confirmations
    pub fn update_dest_confirmations(&mut self, confirmations: u64) {
        self.dest_confirmations = confirmations;
        if self.is_dest_confirmed() && self.status == CrossModelTxStatus::Processing {
            self.status = CrossModelTxStatus::DestConfirmed;
        }
    }

    /// Mark as submitted
    pub fn submit(&mut self, source_tx_hash: H256) {
        self.source_tx_hash = Some(source_tx_hash);
        self.status = CrossModelTxStatus::Submitted;
    }

    /// Mark as processing
    pub fn start_processing(&mut self) {
        if self.status == CrossModelTxStatus::SourceConfirmed {
            self.status = CrossModelTxStatus::Processing;
        }
    }

    /// Mark destination tx
    pub fn set_dest_tx(&mut self, dest_tx_hash: H256) {
        self.dest_tx_hash = Some(dest_tx_hash);
    }

    /// Complete the transaction
    pub fn complete(&mut self) {
        self.status = CrossModelTxStatus::Complete;
        self.completed_at = Some(current_timestamp());
    }

    /// Mark as failed
    pub fn fail(&mut self, error: String) {
        self.status = CrossModelTxStatus::Failed;
        self.error = Some(error);
    }

    /// Mark as refunded
    pub fn refund(&mut self) {
        self.status = CrossModelTxStatus::Refunded;
        self.completed_at = Some(current_timestamp());
    }

    /// Verify transaction integrity
    pub fn verify(&self) -> Result<(), String> {
        match self.tx_type {
            CrossModelTxType::LockAndMint => {
                if self.utxo_inputs.is_empty() {
                    return Err("Lock-and-mint requires UTXO inputs".to_string());
                }
                if self.evm_transfers.is_empty() {
                    return Err("Lock-and-mint requires EVM transfer".to_string());
                }
                let expected = self.utxo_input_amount.saturating_sub(self.fee);
                if self.evm_transfer_amount != expected {
                    return Err(format!(
                        "Amount mismatch: expected {}, got {}",
                        expected, self.evm_transfer_amount
                    ));
                }
            }
            CrossModelTxType::BurnAndUnlock => {
                if self.evm_transfers.is_empty() {
                    return Err("Burn-and-unlock requires EVM transfer".to_string());
                }
                if self.utxo_outputs.is_empty() {
                    return Err("Burn-and-unlock requires UTXO output".to_string());
                }
                let expected = self.evm_transfer_amount.saturating_sub(self.fee);
                if self.utxo_output_amount != expected {
                    return Err(format!(
                        "Amount mismatch: expected {}, got {}",
                        expected, self.utxo_output_amount
                    ));
                }
            }
            _ => {}
        }
        Ok(())
    }
}

/// Atomic swap state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwapState {
    /// Swap initiated, waiting for counterparty
    Initiated,
    /// Counterparty accepted
    Accepted,
    /// Hashlock revealed
    Revealed,
    /// Swap completed
    Completed,
    /// Swap refunded (timeout)
    Refunded,
    /// Swap cancelled
    Cancelled,
}

/// Atomic swap between UTXO and EVM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomicSwap {
    /// Swap ID
    pub id: H256,
    /// Current state
    pub state: SwapState,
    /// Initiator (party A)
    pub initiator: UnifiedAddress,
    /// Counterparty (party B)
    pub counterparty: UnifiedAddress,
    /// Initiator gives (UTXO or EVM)
    pub initiator_gives_utxo: bool,
    /// Initiator amount
    pub initiator_amount: u64,
    /// Counterparty amount
    pub counterparty_amount: u64,
    /// Hash of secret
    pub hash_lock: [u8; 32],
    /// Secret (revealed on completion)
    pub secret: Option<[u8; 32]>,
    /// Timelock for initiator (seconds)
    pub initiator_timelock: u64,
    /// Timelock for counterparty (shorter)
    pub counterparty_timelock: u64,
    /// Initiator's lock transaction
    pub initiator_lock_tx: Option<H256>,
    /// Counterparty's lock transaction
    pub counterparty_lock_tx: Option<H256>,
    /// Initiator's claim transaction
    pub initiator_claim_tx: Option<H256>,
    /// Counterparty's claim transaction
    pub counterparty_claim_tx: Option<H256>,
    /// Creation timestamp
    pub created_at: u64,
    /// Expiry timestamp
    pub expires_at: u64,
}

impl AtomicSwap {
    /// Create new atomic swap
    pub fn new(
        initiator: UnifiedAddress,
        counterparty: UnifiedAddress,
        initiator_gives_utxo: bool,
        initiator_amount: u64,
        counterparty_amount: u64,
        secret: [u8; 32],
        timelock_hours: u64,
    ) -> Self {
        let hash_lock = Self::hash_secret(&secret);
        let now = current_timestamp();
        
        Self {
            id: Self::generate_id(&initiator, &counterparty, &hash_lock),
            state: SwapState::Initiated,
            initiator,
            counterparty,
            initiator_gives_utxo,
            initiator_amount,
            counterparty_amount,
            hash_lock,
            secret: Some(secret),
            initiator_timelock: now + timelock_hours * 3600,
            counterparty_timelock: now + (timelock_hours / 2) * 3600,
            initiator_lock_tx: None,
            counterparty_lock_tx: None,
            initiator_claim_tx: None,
            counterparty_claim_tx: None,
            created_at: now,
            expires_at: now + timelock_hours * 3600,
        }
    }

    /// Hash the secret
    fn hash_secret(secret: &[u8; 32]) -> [u8; 32] {
        let mut hasher = Hasher::new();
        hasher.update(secret);
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(result.as_bytes());
        hash
    }

    /// Generate swap ID
    fn generate_id(initiator: &UnifiedAddress, counterparty: &UnifiedAddress, hash_lock: &[u8; 32]) -> H256 {
        let mut hasher = Hasher::new();
        hasher.update(&initiator.bytes);
        hasher.update(&counterparty.bytes);
        hasher.update(hash_lock);
        hasher.update(&current_timestamp().to_be_bytes());
        H256::from_slice(hasher.finalize().as_bytes())
    }

    /// Verify secret matches hash
    pub fn verify_secret(&self, secret: &[u8; 32]) -> bool {
        Self::hash_secret(secret) == self.hash_lock
    }

    /// Accept swap (counterparty locks funds)
    pub fn accept(&mut self, lock_tx: H256) {
        if self.state == SwapState::Initiated {
            self.counterparty_lock_tx = Some(lock_tx);
            self.state = SwapState::Accepted;
        }
    }

    /// Reveal secret (initiator claims counterparty's funds)
    pub fn reveal(&mut self, secret: [u8; 32], claim_tx: H256) -> bool {
        if self.state != SwapState::Accepted {
            return false;
        }
        if !self.verify_secret(&secret) {
            return false;
        }
        self.secret = Some(secret);
        self.initiator_claim_tx = Some(claim_tx);
        self.state = SwapState::Revealed;
        true
    }

    /// Complete swap (counterparty claims using revealed secret)
    pub fn complete(&mut self, claim_tx: H256) {
        if self.state == SwapState::Revealed {
            self.counterparty_claim_tx = Some(claim_tx);
            self.state = SwapState::Completed;
        }
    }

    /// Refund (timelock expired)
    pub fn refund(&mut self) {
        if self.can_refund() {
            self.state = SwapState::Refunded;
        }
    }

    /// Check if refund is possible
    pub fn can_refund(&self) -> bool {
        let now = current_timestamp();
        match self.state {
            SwapState::Initiated => now > self.initiator_timelock,
            SwapState::Accepted => now > self.counterparty_timelock,
            _ => false,
        }
    }

    /// Check if swap is expired
    pub fn is_expired(&self) -> bool {
        current_timestamp() > self.expires_at
    }
}

/// Cross-model transaction manager
pub struct CrossModelTxManager {
    /// Pending transactions
    transactions: Arc<RwLock<HashMap<H256, CrossModelTransaction>>>,
    /// Active swaps
    swaps: Arc<RwLock<HashMap<H256, AtomicSwap>>>,
    /// Transactions by initiator
    by_initiator: Arc<RwLock<HashMap<[u8; 20], Vec<H256>>>>,
}

impl CrossModelTxManager {
    pub fn new() -> Self {
        Self {
            transactions: Arc::new(RwLock::new(HashMap::new())),
            swaps: Arc::new(RwLock::new(HashMap::new())),
            by_initiator: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Submit a cross-model transaction
    pub fn submit(&self, tx: CrossModelTransaction) -> Result<H256, String> {
        tx.verify()?;
        
        let id = tx.id;
        let initiator_bytes = tx.initiator.bytes;
        
        self.transactions.write().insert(id, tx);
        self.by_initiator.write()
            .entry(initiator_bytes)
            .or_insert_with(Vec::new)
            .push(id);
        
        Ok(id)
    }

    /// Get transaction by ID
    pub fn get(&self, id: &H256) -> Option<CrossModelTransaction> {
        self.transactions.read().get(id).cloned()
    }

    /// Get transactions for address
    pub fn get_for_address(&self, address: &[u8; 20]) -> Vec<CrossModelTransaction> {
        let tx_ids = self.by_initiator.read()
            .get(address)
            .cloned()
            .unwrap_or_default();
        
        let txs = self.transactions.read();
        tx_ids.iter()
            .filter_map(|id| txs.get(id).cloned())
            .collect()
    }

    /// Update transaction status
    pub fn update_status(&self, id: &H256, f: impl FnOnce(&mut CrossModelTransaction)) {
        if let Some(tx) = self.transactions.write().get_mut(id) {
            f(tx);
        }
    }

    /// Create atomic swap
    pub fn create_swap(&self, swap: AtomicSwap) -> H256 {
        let id = swap.id;
        self.swaps.write().insert(id, swap);
        id
    }

    /// Get swap by ID
    pub fn get_swap(&self, id: &H256) -> Option<AtomicSwap> {
        self.swaps.read().get(id).cloned()
    }

    /// Update swap
    pub fn update_swap(&self, id: &H256, f: impl FnOnce(&mut AtomicSwap)) {
        if let Some(swap) = self.swaps.write().get_mut(id) {
            f(swap);
        }
    }

    /// Process expired transactions
    pub fn process_expired(&self) -> Vec<H256> {
        let mut expired = Vec::new();
        let mut txs = self.transactions.write();
        
        for (id, tx) in txs.iter_mut() {
            if tx.is_expired() && tx.status != CrossModelTxStatus::Expired {
                tx.status = CrossModelTxStatus::Expired;
                expired.push(*id);
            }
        }
        
        expired
    }

    /// Get statistics
    pub fn stats(&self) -> CrossModelStats {
        let txs = self.transactions.read();
        let swaps = self.swaps.read();
        
        CrossModelStats {
            total_transactions: txs.len() as u64,
            pending_transactions: txs.values()
                .filter(|t| matches!(t.status, 
                    CrossModelTxStatus::Created | 
                    CrossModelTxStatus::Submitted |
                    CrossModelTxStatus::SourceConfirmed |
                    CrossModelTxStatus::Processing
                ))
                .count() as u64,
            completed_transactions: txs.values()
                .filter(|t| t.status == CrossModelTxStatus::Complete)
                .count() as u64,
            total_swaps: swaps.len() as u64,
            active_swaps: swaps.values()
                .filter(|s| matches!(s.state, SwapState::Initiated | SwapState::Accepted))
                .count() as u64,
        }
    }
}

impl Default for CrossModelTxManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Cross-model statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossModelStats {
    pub total_transactions: u64,
    pub pending_transactions: u64,
    pub completed_transactions: u64,
    pub total_swaps: u64,
    pub active_swaps: u64,
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::unified_address::NetworkPrefix;

    #[test]
    fn test_lock_and_mint() {
        let initiator = UnifiedAddress::from_bytes([1u8; 20], NetworkPrefix::Mainnet);
        
        let inputs = vec![UtxoInput {
            outpoint: OutPoint { txid: H256([1u8; 32]), vout: 0 },
            amount: 1_000_000_000,
            owner: Address([1u8; 20]),
            signature: vec![],
        }];
        
        let tx = CrossModelTransaction::lock_and_mint(
            inputs,
            [2u8; 20],
            initiator,
        );
        
        assert_eq!(tx.tx_type, CrossModelTxType::LockAndMint);
        assert_eq!(tx.utxo_input_amount, 1_000_000_000);
        assert_eq!(tx.fee, 1_000_000); // 0.1%
        assert_eq!(tx.evm_transfer_amount, 999_000_000);
        assert!(tx.verify().is_ok());
    }

    #[test]
    fn test_burn_and_unlock() {
        let initiator = UnifiedAddress::from_bytes([1u8; 20], NetworkPrefix::Mainnet);
        
        let tx = CrossModelTransaction::burn_and_unlock(
            [1u8; 20],
            500_000_000,
            Address([2u8; 20]),
            initiator,
        );
        
        assert_eq!(tx.tx_type, CrossModelTxType::BurnAndUnlock);
        assert_eq!(tx.evm_transfer_amount, 500_000_000);
        assert_eq!(tx.fee, 500_000); // 0.1%
        assert_eq!(tx.utxo_output_amount, 499_500_000);
        assert!(tx.verify().is_ok());
    }

    #[test]
    fn test_atomic_swap() {
        let initiator = UnifiedAddress::from_bytes([1u8; 20], NetworkPrefix::Mainnet);
        let counterparty = UnifiedAddress::from_bytes([2u8; 20], NetworkPrefix::Mainnet);
        
        let secret = [42u8; 32];
        let swap = AtomicSwap::new(
            initiator,
            counterparty,
            true, // initiator gives UTXO
            1_000_000_000,
            500_000_000,
            secret,
            24, // 24 hours
        );
        
        assert_eq!(swap.state, SwapState::Initiated);
        assert!(swap.verify_secret(&secret));
        assert!(!swap.verify_secret(&[0u8; 32]));
    }

    #[test]
    fn test_swap_flow() {
        let initiator = UnifiedAddress::from_bytes([1u8; 20], NetworkPrefix::Mainnet);
        let counterparty = UnifiedAddress::from_bytes([2u8; 20], NetworkPrefix::Mainnet);
        
        let secret = [42u8; 32];
        let mut swap = AtomicSwap::new(
            initiator,
            counterparty,
            true,
            1_000_000_000,
            500_000_000,
            secret,
            24,
        );
        
        // Accept
        swap.accept(H256([1u8; 32]));
        assert_eq!(swap.state, SwapState::Accepted);
        
        // Reveal
        assert!(swap.reveal(secret, H256([2u8; 32])));
        assert_eq!(swap.state, SwapState::Revealed);
        
        // Complete
        swap.complete(H256([3u8; 32]));
        assert_eq!(swap.state, SwapState::Completed);
    }

    #[test]
    fn test_cross_model_manager() {
        let manager = CrossModelTxManager::new();
        let initiator = UnifiedAddress::from_bytes([1u8; 20], NetworkPrefix::Mainnet);
        
        let tx = CrossModelTransaction::burn_and_unlock(
            [1u8; 20],
            100_000_000,
            Address([2u8; 20]),
            initiator.clone(),
        );
        
        let id = manager.submit(tx).unwrap();
        
        let retrieved = manager.get(&id).unwrap();
        assert_eq!(retrieved.id, id);
        
        let for_addr = manager.get_for_address(&initiator.bytes);
        assert_eq!(for_addr.len(), 1);
    }
}
