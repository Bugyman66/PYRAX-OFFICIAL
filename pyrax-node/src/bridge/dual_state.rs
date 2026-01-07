//! Dual State Bridge Core
//!
//! Handles conversion between UTXO and Account models

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use blake3::Hasher;

use crate::types::{Address, H256, Transaction, OutPoint, TxOutput};
use super::{
    UTXO_LOCK_CONFIRMATIONS, EVM_BURN_CONFIRMATIONS, 
    BRIDGE_FEE_BASIS_POINTS, MIN_BRIDGE_AMOUNT, MAX_BRIDGE_AMOUNT,
};

/// Bridge state for tracking conversions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BridgeState {
    /// Bridge is active and processing
    Active,
    /// Bridge is paused (maintenance)
    Paused,
    /// Bridge is stopped (emergency)
    Stopped,
}

/// Conversion direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConversionDirection {
    /// UTXO to EVM Account
    UtxoToAccount,
    /// EVM Account to UTXO
    AccountToUtxo,
}

/// Conversion status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConversionStatus {
    /// Pending confirmation on source chain
    Pending,
    /// Confirmed on source, awaiting mint/unlock
    Confirmed,
    /// Minting/unlocking in progress
    Processing,
    /// Completed successfully
    Completed,
    /// Failed (will be refunded)
    Failed,
    /// Refunded to source
    Refunded,
}

/// Locked UTXO record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedUtxo {
    /// Original UTXO outpoint
    pub outpoint: OutPoint,
    /// Lock transaction hash
    pub lock_tx_hash: H256,
    /// Amount locked (in base units)
    pub amount: u64,
    /// Original owner address (UTXO)
    pub utxo_owner: Address,
    /// Target EVM address
    pub evm_recipient: [u8; 20],
    /// Block height when locked
    pub lock_height: u64,
    /// Current confirmations
    pub confirmations: u64,
    /// Lock timestamp
    pub locked_at: u64,
    /// Whether unlocked (for reverse conversion)
    pub unlocked: bool,
}

/// Minted balance record (EVM side)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MintedBalance {
    /// Conversion ID
    pub conversion_id: H256,
    /// EVM recipient address
    pub recipient: [u8; 20],
    /// Amount minted
    pub amount: u64,
    /// Mint transaction hash on EVM
    pub mint_tx_hash: [u8; 32],
    /// Block number when minted
    pub mint_block: u64,
    /// Mint timestamp
    pub minted_at: u64,
    /// Whether burned (for reverse conversion)
    pub burned: bool,
}

/// Conversion request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionRequest {
    /// Unique conversion ID
    pub id: H256,
    /// Direction of conversion
    pub direction: ConversionDirection,
    /// Current status
    pub status: ConversionStatus,
    /// Amount being converted (before fees)
    pub amount: u64,
    /// Bridge fee deducted
    pub fee: u64,
    /// Net amount after fee
    pub net_amount: u64,
    /// Source address
    pub source_address: Address,
    /// Destination address (20 bytes for both)
    pub destination_address: [u8; 20],
    /// Source transaction hash
    pub source_tx_hash: H256,
    /// Destination transaction hash (once completed)
    pub destination_tx_hash: Option<H256>,
    /// Block height at request
    pub request_height: u64,
    /// Confirmations received
    pub confirmations: u64,
    /// Required confirmations
    pub required_confirmations: u64,
    /// Request timestamp
    pub created_at: u64,
    /// Completion timestamp
    pub completed_at: Option<u64>,
    /// Error message if failed
    pub error: Option<String>,
}

impl ConversionRequest {
    /// Create UTXO → Account conversion request
    pub fn utxo_to_account(
        source_tx_hash: H256,
        amount: u64,
        source_address: Address,
        evm_recipient: [u8; 20],
        request_height: u64,
    ) -> Result<Self, BridgeError> {
        Self::new(
            ConversionDirection::UtxoToAccount,
            amount,
            source_address,
            evm_recipient,
            source_tx_hash,
            request_height,
            UTXO_LOCK_CONFIRMATIONS,
        )
    }

    /// Create Account → UTXO conversion request
    pub fn account_to_utxo(
        burn_tx_hash: H256,
        amount: u64,
        evm_address: [u8; 20],
        utxo_recipient: Address,
        request_height: u64,
    ) -> Result<Self, BridgeError> {
        Self::new(
            ConversionDirection::AccountToUtxo,
            amount,
            Address(evm_address),
            utxo_recipient.0,
            burn_tx_hash,
            request_height,
            EVM_BURN_CONFIRMATIONS,
        )
    }

    fn new(
        direction: ConversionDirection,
        amount: u64,
        source_address: Address,
        destination_address: [u8; 20],
        source_tx_hash: H256,
        request_height: u64,
        required_confirmations: u64,
    ) -> Result<Self, BridgeError> {
        // Validate amount
        if amount < MIN_BRIDGE_AMOUNT {
            return Err(BridgeError::AmountTooSmall {
                min: MIN_BRIDGE_AMOUNT,
                provided: amount,
            });
        }
        if amount > MAX_BRIDGE_AMOUNT {
            return Err(BridgeError::AmountTooLarge {
                max: MAX_BRIDGE_AMOUNT,
                provided: amount,
            });
        }

        // Calculate fee
        let fee = (amount as u128 * BRIDGE_FEE_BASIS_POINTS as u128 / 10000) as u64;
        let net_amount = amount.saturating_sub(fee);

        // Generate conversion ID
        let id = Self::generate_id(&source_tx_hash, &source_address, amount);

        Ok(Self {
            id,
            direction,
            status: ConversionStatus::Pending,
            amount,
            fee,
            net_amount,
            source_address,
            destination_address,
            source_tx_hash,
            destination_tx_hash: None,
            request_height,
            confirmations: 0,
            required_confirmations,
            created_at: current_timestamp(),
            completed_at: None,
            error: None,
        })
    }

    fn generate_id(tx_hash: &H256, address: &Address, amount: u64) -> H256 {
        let mut hasher = Hasher::new();
        hasher.update(&tx_hash.0);
        hasher.update(&address.0);
        hasher.update(&amount.to_be_bytes());
        hasher.update(&current_timestamp().to_be_bytes());
        H256::from_slice(hasher.finalize().as_bytes())
    }

    /// Update confirmation count
    pub fn update_confirmations(&mut self, current_height: u64) {
        if current_height >= self.request_height {
            self.confirmations = current_height - self.request_height + 1;
            
            if self.confirmations >= self.required_confirmations 
                && self.status == ConversionStatus::Pending 
            {
                self.status = ConversionStatus::Confirmed;
            }
        }
    }

    /// Mark as processing
    pub fn mark_processing(&mut self) {
        if self.status == ConversionStatus::Confirmed {
            self.status = ConversionStatus::Processing;
        }
    }

    /// Mark as completed
    pub fn mark_completed(&mut self, dest_tx_hash: H256) {
        self.status = ConversionStatus::Completed;
        self.destination_tx_hash = Some(dest_tx_hash);
        self.completed_at = Some(current_timestamp());
    }

    /// Mark as failed
    pub fn mark_failed(&mut self, error: String) {
        self.status = ConversionStatus::Failed;
        self.error = Some(error);
    }

    /// Mark as refunded
    pub fn mark_refunded(&mut self) {
        self.status = ConversionStatus::Refunded;
        self.completed_at = Some(current_timestamp());
    }

    /// Check if ready to process
    pub fn is_ready(&self) -> bool {
        self.status == ConversionStatus::Confirmed
    }
}

/// Bridge errors
#[derive(Debug, thiserror::Error)]
pub enum BridgeError {
    #[error("Bridge is paused")]
    BridgePaused,

    #[error("Bridge is stopped")]
    BridgeStopped,

    #[error("Amount too small: minimum {min}, provided {provided}")]
    AmountTooSmall { min: u64, provided: u64 },

    #[error("Amount too large: maximum {max}, provided {provided}")]
    AmountTooLarge { max: u64, provided: u64 },

    #[error("Conversion not found: {0:?}")]
    ConversionNotFound(H256),

    #[error("Invalid conversion status: expected {expected:?}, got {actual:?}")]
    InvalidStatus {
        expected: ConversionStatus,
        actual: ConversionStatus,
    },

    #[error("UTXO not found: {0:?}")]
    UtxoNotFound(OutPoint),

    #[error("UTXO already locked: {0:?}")]
    UtxoAlreadyLocked(OutPoint),

    #[error("Insufficient balance: required {required}, available {available}")]
    InsufficientBalance { required: u64, available: u64 },

    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Transaction verification failed: {0}")]
    VerificationFailed(String),

    #[error("Not enough confirmations: required {required}, got {current}")]
    InsufficientConfirmations { required: u64, current: u64 },

    #[error("Duplicate conversion request")]
    DuplicateRequest,

    #[error("Storage error: {0}")]
    StorageError(String),
}

/// Dual State Bridge - Main bridge implementation
pub struct DualStateBridge {
    /// Current bridge state
    state: Arc<RwLock<BridgeState>>,
    /// Pending conversions by ID
    conversions: Arc<RwLock<HashMap<H256, ConversionRequest>>>,
    /// Locked UTXOs by outpoint
    locked_utxos: Arc<RwLock<HashMap<OutPoint, LockedUtxo>>>,
    /// Minted balances by conversion ID
    minted_balances: Arc<RwLock<HashMap<H256, MintedBalance>>>,
    /// Conversion queue (IDs ready to process)
    processing_queue: Arc<RwLock<Vec<H256>>>,
    /// Total locked in bridge (UTXO side)
    total_locked: Arc<RwLock<u64>>,
    /// Total minted (EVM side)
    total_minted: Arc<RwLock<u64>>,
    /// Total fees collected
    total_fees: Arc<RwLock<u64>>,
    /// Current UTXO chain height
    utxo_height: Arc<RwLock<u64>>,
    /// Current EVM chain height
    evm_height: Arc<RwLock<u64>>,
}

impl DualStateBridge {
    /// Create new bridge
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(BridgeState::Active)),
            conversions: Arc::new(RwLock::new(HashMap::new())),
            locked_utxos: Arc::new(RwLock::new(HashMap::new())),
            minted_balances: Arc::new(RwLock::new(HashMap::new())),
            processing_queue: Arc::new(RwLock::new(Vec::new())),
            total_locked: Arc::new(RwLock::new(0)),
            total_minted: Arc::new(RwLock::new(0)),
            total_fees: Arc::new(RwLock::new(0)),
            utxo_height: Arc::new(RwLock::new(0)),
            evm_height: Arc::new(RwLock::new(0)),
        }
    }

    /// Check if bridge is active
    pub fn is_active(&self) -> bool {
        *self.state.read() == BridgeState::Active
    }

    /// Pause the bridge
    pub fn pause(&self) {
        *self.state.write() = BridgeState::Paused;
    }

    /// Resume the bridge
    pub fn resume(&self) {
        *self.state.write() = BridgeState::Active;
    }

    /// Emergency stop
    pub fn emergency_stop(&self) {
        *self.state.write() = BridgeState::Stopped;
    }

    /// Update chain heights
    pub fn update_heights(&self, utxo_height: u64, evm_height: u64) {
        *self.utxo_height.write() = utxo_height;
        *self.evm_height.write() = evm_height;
    }

    /// Initiate UTXO → Account conversion
    pub fn lock_utxo(
        &self,
        outpoint: OutPoint,
        lock_tx_hash: H256,
        amount: u64,
        utxo_owner: Address,
        evm_recipient: [u8; 20],
    ) -> Result<H256, BridgeError> {
        // Check bridge state
        match *self.state.read() {
            BridgeState::Paused => return Err(BridgeError::BridgePaused),
            BridgeState::Stopped => return Err(BridgeError::BridgeStopped),
            BridgeState::Active => {}
        }

        // Check if already locked
        if self.locked_utxos.read().contains_key(&outpoint) {
            return Err(BridgeError::UtxoAlreadyLocked(outpoint));
        }

        let current_height = *self.utxo_height.read();

        // Create conversion request
        let request = ConversionRequest::utxo_to_account(
            lock_tx_hash,
            amount,
            utxo_owner,
            evm_recipient,
            current_height,
        )?;

        let conversion_id = request.id;

        // Record locked UTXO
        let locked = LockedUtxo {
            outpoint: outpoint.clone(),
            lock_tx_hash,
            amount,
            utxo_owner,
            evm_recipient,
            lock_height: current_height,
            confirmations: 0,
            locked_at: current_timestamp(),
            unlocked: false,
        };

        // Store records
        self.locked_utxos.write().insert(outpoint, locked);
        self.conversions.write().insert(conversion_id, request);
        *self.total_locked.write() += amount;

        Ok(conversion_id)
    }

    /// Initiate Account → UTXO conversion (burn EVM tokens)
    pub fn burn_for_utxo(
        &self,
        burn_tx_hash: H256,
        amount: u64,
        evm_burner: [u8; 20],
        utxo_recipient: Address,
    ) -> Result<H256, BridgeError> {
        // Check bridge state
        match *self.state.read() {
            BridgeState::Paused => return Err(BridgeError::BridgePaused),
            BridgeState::Stopped => return Err(BridgeError::BridgeStopped),
            BridgeState::Active => {}
        }

        let current_height = *self.evm_height.read();

        // Create conversion request
        let request = ConversionRequest::account_to_utxo(
            burn_tx_hash,
            amount,
            evm_burner,
            utxo_recipient,
            current_height,
        )?;

        let conversion_id = request.id;

        // Store request
        self.conversions.write().insert(conversion_id, request);

        Ok(conversion_id)
    }

    /// Process confirmations and update conversion statuses
    pub fn process_confirmations(&self) {
        let utxo_height = *self.utxo_height.read();
        let evm_height = *self.evm_height.read();
        
        let mut conversions = self.conversions.write();
        let mut queue = self.processing_queue.write();

        for (id, conv) in conversions.iter_mut() {
            if conv.status == ConversionStatus::Pending {
                let height = match conv.direction {
                    ConversionDirection::UtxoToAccount => utxo_height,
                    ConversionDirection::AccountToUtxo => evm_height,
                };
                
                conv.update_confirmations(height);
                
                if conv.is_ready() && !queue.contains(id) {
                    queue.push(*id);
                }
            }
        }
    }

    /// Get next conversion ready to process
    pub fn get_next_ready(&self) -> Option<ConversionRequest> {
        let mut queue = self.processing_queue.write();
        let conversions = self.conversions.read();
        
        while let Some(id) = queue.first().cloned() {
            if let Some(conv) = conversions.get(&id) {
                if conv.is_ready() {
                    queue.remove(0);
                    return Some(conv.clone());
                }
            }
            queue.remove(0);
        }
        
        None
    }

    /// Complete UTXO → Account conversion (mint EVM tokens)
    pub fn complete_mint(
        &self,
        conversion_id: H256,
        mint_tx_hash: [u8; 32],
        mint_block: u64,
    ) -> Result<(), BridgeError> {
        let mut conversions = self.conversions.write();
        let conv = conversions.get_mut(&conversion_id)
            .ok_or(BridgeError::ConversionNotFound(conversion_id))?;

        if conv.direction != ConversionDirection::UtxoToAccount {
            return Err(BridgeError::InvalidStatus {
                expected: ConversionStatus::Confirmed,
                actual: conv.status,
            });
        }

        conv.mark_processing();

        // Record minted balance
        let minted = MintedBalance {
            conversion_id,
            recipient: conv.destination_address,
            amount: conv.net_amount,
            mint_tx_hash,
            mint_block,
            minted_at: current_timestamp(),
            burned: false,
        };

        self.minted_balances.write().insert(conversion_id, minted);
        *self.total_minted.write() += conv.net_amount;
        *self.total_fees.write() += conv.fee;

        conv.mark_completed(H256(mint_tx_hash));

        Ok(())
    }

    /// Complete Account → UTXO conversion (unlock UTXO)
    pub fn complete_unlock(
        &self,
        conversion_id: H256,
        unlock_tx_hash: H256,
    ) -> Result<(), BridgeError> {
        let mut conversions = self.conversions.write();
        let conv = conversions.get_mut(&conversion_id)
            .ok_or(BridgeError::ConversionNotFound(conversion_id))?;

        if conv.direction != ConversionDirection::AccountToUtxo {
            return Err(BridgeError::InvalidStatus {
                expected: ConversionStatus::Confirmed,
                actual: conv.status,
            });
        }

        conv.mark_processing();

        // Update totals
        *self.total_minted.write() = self.total_minted.read()
            .saturating_sub(conv.amount);
        *self.total_locked.write() = self.total_locked.read()
            .saturating_sub(conv.net_amount);
        *self.total_fees.write() += conv.fee;

        conv.mark_completed(unlock_tx_hash);

        Ok(())
    }

    /// Get conversion by ID
    pub fn get_conversion(&self, id: &H256) -> Option<ConversionRequest> {
        self.conversions.read().get(id).cloned()
    }

    /// Get all pending conversions
    pub fn get_pending_conversions(&self) -> Vec<ConversionRequest> {
        self.conversions.read()
            .values()
            .filter(|c| c.status == ConversionStatus::Pending)
            .cloned()
            .collect()
    }

    /// Get conversions for an address
    pub fn get_conversions_for_address(&self, address: &Address) -> Vec<ConversionRequest> {
        self.conversions.read()
            .values()
            .filter(|c| &c.source_address == address || c.destination_address == address.0)
            .cloned()
            .collect()
    }

    /// Get bridge statistics
    pub fn stats(&self) -> BridgeStats {
        let conversions = self.conversions.read();
        
        BridgeStats {
            state: *self.state.read(),
            total_locked: *self.total_locked.read(),
            total_minted: *self.total_minted.read(),
            total_fees: *self.total_fees.read(),
            pending_conversions: conversions.values()
                .filter(|c| c.status == ConversionStatus::Pending)
                .count() as u64,
            completed_conversions: conversions.values()
                .filter(|c| c.status == ConversionStatus::Completed)
                .count() as u64,
            utxo_height: *self.utxo_height.read(),
            evm_height: *self.evm_height.read(),
        }
    }

    /// Verify bridge invariant (locked == minted)
    pub fn verify_invariant(&self) -> bool {
        let locked = *self.total_locked.read();
        let minted = *self.total_minted.read();
        let fees = *self.total_fees.read();
        
        // Minted should equal locked minus fees
        minted + fees <= locked
    }
}

impl Default for DualStateBridge {
    fn default() -> Self {
        Self::new()
    }
}

/// Bridge statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeStats {
    pub state: BridgeState,
    pub total_locked: u64,
    pub total_minted: u64,
    pub total_fees: u64,
    pub pending_conversions: u64,
    pub completed_conversions: u64,
    pub utxo_height: u64,
    pub evm_height: u64,
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

    #[test]
    fn test_bridge_creation() {
        let bridge = DualStateBridge::new();
        assert!(bridge.is_active());
        assert!(bridge.verify_invariant());
    }

    #[test]
    fn test_bridge_pause_resume() {
        let bridge = DualStateBridge::new();
        
        bridge.pause();
        assert!(!bridge.is_active());
        
        bridge.resume();
        assert!(bridge.is_active());
    }

    #[test]
    fn test_lock_utxo() {
        let bridge = DualStateBridge::new();
        bridge.update_heights(100, 50);
        
        let outpoint = OutPoint {
            txid: H256([1u8; 32]),
            vout: 0,
        };
        
        let result = bridge.lock_utxo(
            outpoint,
            H256([2u8; 32]),
            1_000_000_000, // 10 PYRAX
            Address([1u8; 20]),
            [2u8; 20],
        );
        
        assert!(result.is_ok());
        let conv_id = result.unwrap();
        
        let conv = bridge.get_conversion(&conv_id).unwrap();
        assert_eq!(conv.direction, ConversionDirection::UtxoToAccount);
        assert_eq!(conv.status, ConversionStatus::Pending);
        assert_eq!(conv.amount, 1_000_000_000);
        
        // Fee should be 0.1%
        assert_eq!(conv.fee, 1_000_000);
        assert_eq!(conv.net_amount, 999_000_000);
    }

    #[test]
    fn test_burn_for_utxo() {
        let bridge = DualStateBridge::new();
        bridge.update_heights(100, 50);
        
        let result = bridge.burn_for_utxo(
            H256([3u8; 32]),
            5_000_000_000, // 50 PYRAX
            [1u8; 20],
            Address([2u8; 20]),
        );
        
        assert!(result.is_ok());
        let conv_id = result.unwrap();
        
        let conv = bridge.get_conversion(&conv_id).unwrap();
        assert_eq!(conv.direction, ConversionDirection::AccountToUtxo);
        assert_eq!(conv.status, ConversionStatus::Pending);
    }

    #[test]
    fn test_amount_validation() {
        let bridge = DualStateBridge::new();
        bridge.update_heights(100, 50);
        
        // Too small
        let result = bridge.lock_utxo(
            OutPoint { txid: H256([1u8; 32]), vout: 0 },
            H256([2u8; 32]),
            100, // Too small
            Address([1u8; 20]),
            [2u8; 20],
        );
        assert!(matches!(result, Err(BridgeError::AmountTooSmall { .. })));
        
        // Too large
        let result = bridge.lock_utxo(
            OutPoint { txid: H256([1u8; 32]), vout: 1 },
            H256([2u8; 32]),
            MAX_BRIDGE_AMOUNT + 1,
            Address([1u8; 20]),
            [2u8; 20],
        );
        assert!(matches!(result, Err(BridgeError::AmountTooLarge { .. })));
    }

    #[test]
    fn test_confirmation_processing() {
        let bridge = DualStateBridge::new();
        bridge.update_heights(100, 50);
        
        let outpoint = OutPoint {
            txid: H256([1u8; 32]),
            vout: 0,
        };
        
        let conv_id = bridge.lock_utxo(
            outpoint,
            H256([2u8; 32]),
            1_000_000_000,
            Address([1u8; 20]),
            [2u8; 20],
        ).unwrap();
        
        // Initially pending
        let conv = bridge.get_conversion(&conv_id).unwrap();
        assert_eq!(conv.status, ConversionStatus::Pending);
        
        // Advance height by required confirmations
        bridge.update_heights(100 + UTXO_LOCK_CONFIRMATIONS, 50);
        bridge.process_confirmations();
        
        // Should be confirmed now
        let conv = bridge.get_conversion(&conv_id).unwrap();
        assert_eq!(conv.status, ConversionStatus::Confirmed);
        assert!(conv.is_ready());
    }

    #[test]
    fn test_complete_mint() {
        let bridge = DualStateBridge::new();
        bridge.update_heights(100, 50);
        
        let outpoint = OutPoint {
            txid: H256([1u8; 32]),
            vout: 0,
        };
        
        let conv_id = bridge.lock_utxo(
            outpoint,
            H256([2u8; 32]),
            1_000_000_000,
            Address([1u8; 20]),
            [2u8; 20],
        ).unwrap();
        
        // Advance to confirm
        bridge.update_heights(106, 50);
        bridge.process_confirmations();
        
        // Complete mint
        let result = bridge.complete_mint(conv_id, [3u8; 32], 51);
        assert!(result.is_ok());
        
        let conv = bridge.get_conversion(&conv_id).unwrap();
        assert_eq!(conv.status, ConversionStatus::Completed);
        
        // Check totals
        let stats = bridge.stats();
        assert_eq!(stats.total_minted, 999_000_000);
        assert_eq!(stats.total_fees, 1_000_000);
    }
}
