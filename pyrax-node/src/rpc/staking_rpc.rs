//! PYRAX Stream C Staking RPC
//!
//! Production JSON-RPC methods for the staking service:
//! - Validator registration and management
//! - Stake deposits and withdrawals
//! - Checkpoint submission and attestation
//! - Staking statistics and queries

use std::sync::Arc;
use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::types::{H256, Address};
use crate::services::staking::{
    StakingService, StakingConfig, Validator, Checkpoint, StakingStats,
};
use crate::zkrollup::{Proof, ProofType};

/// Staking RPC trait
#[rpc(server)]
pub trait StakingRpc {
    /// Get staking statistics
    #[method(name = "pyrax_getStakingStats")]
    async fn get_staking_stats(&self) -> RpcResult<Value>;

    /// Get validator info
    #[method(name = "pyrax_getValidator")]
    async fn get_validator(&self, address: String) -> RpcResult<Value>;

    /// Get all validators
    #[method(name = "pyrax_getValidators")]
    async fn get_validators(&self) -> RpcResult<Value>;

    /// Get active validator set
    #[method(name = "pyrax_getActiveValidators")]
    async fn get_active_validators(&self) -> RpcResult<Value>;

    /// Register as a validator
    #[method(name = "pyrax_registerValidator")]
    async fn register_validator(&self, address: String, stake: u64) -> RpcResult<Value>;

    /// Deposit additional stake
    #[method(name = "pyrax_depositStake")]
    async fn deposit_stake(&self, address: String, amount: u64) -> RpcResult<Value>;

    /// Start unbonding stake
    #[method(name = "pyrax_startUnbonding")]
    async fn start_unbonding(&self, address: String, amount: u64) -> RpcResult<Value>;

    /// Complete unbonding and withdraw
    #[method(name = "pyrax_completeUnbonding")]
    async fn complete_unbonding(&self, address: String) -> RpcResult<Value>;

    /// Get latest checkpoint
    #[method(name = "pyrax_getLatestCheckpoint")]
    async fn get_latest_checkpoint(&self) -> RpcResult<Value>;

    /// Get checkpoint by ID
    #[method(name = "pyrax_getCheckpoint")]
    async fn get_checkpoint(&self, id: u64) -> RpcResult<Value>;

    /// Submit a checkpoint with ZK proof
    #[method(name = "pyrax_submitCheckpoint")]
    async fn submit_checkpoint(
        &self,
        validator: String,
        stream_a_height: u64,
        stream_b_height: u64,
        state_root: String,
        utxo_commitment: String,
        proof_data: String,
    ) -> RpcResult<Value>;

    /// Attest to a checkpoint
    #[method(name = "pyrax_attestCheckpoint")]
    async fn attest_checkpoint(&self, validator: String, checkpoint_id: u64) -> RpcResult<Value>;

    /// Get Stream C info
    #[method(name = "pyrax_getStreamCInfo")]
    async fn get_stream_c_info(&self) -> RpcResult<Value>;
}

/// Staking RPC implementation
pub struct StakingRpcImpl {
    staking_service: Arc<StakingService>,
}

impl StakingRpcImpl {
    pub fn new(staking_service: Arc<StakingService>) -> Self {
        Self { staking_service }
    }

    fn parse_address(s: &str) -> Result<Address, String> {
        let s = s.strip_prefix("0x").unwrap_or(s);
        if s.len() != 40 {
            return Err("Invalid address length".to_string());
        }
        let bytes = hex::decode(s).map_err(|e| e.to_string())?;
        Ok(Address::from_slice(&bytes))
    }

    fn parse_h256(s: &str) -> Result<H256, String> {
        let s = s.strip_prefix("0x").unwrap_or(s);
        if s.len() != 64 {
            return Err("Invalid hash length".to_string());
        }
        let bytes = hex::decode(s).map_err(|e| e.to_string())?;
        Ok(H256::from_slice(&bytes))
    }

    fn validator_to_json(v: &Validator) -> Value {
        json!({
            "address": format!("0x{}", hex::encode(&v.address.0)),
            "stake": v.stake,
            "stakePyrax": v.stake as f64 / 100_000_000.0,
            "effectiveStake": v.effective_stake(),
            "commissionBps": v.commission_bps,
            "totalRewards": v.total_rewards,
            "totalSlashed": v.total_slashed,
            "checkpointsValidated": v.checkpoints_validated,
            "invalidProofs": v.invalid_proofs,
            "registeredAt": v.registered_at,
            "lastActiveAt": v.last_active_at,
            "isActive": v.is_active,
            "unbondingAt": v.unbonding_at,
            "unbondingAmount": v.unbonding_amount,
        })
    }

    fn checkpoint_to_json(c: &Checkpoint) -> Value {
        json!({
            "id": c.id,
            "streamAHeight": c.stream_a_height,
            "streamBHeight": c.stream_b_height,
            "stateRoot": format!("0x{}", hex::encode(&c.state_root.0)),
            "utxoCommitment": format!("0x{}", hex::encode(&c.utxo_commitment.0)),
            "validator": format!("0x{}", hex::encode(&c.validator.0)),
            "timestamp": c.timestamp,
            "finalized": c.finalized,
            "attestationCount": c.attestations.len(),
            "hasProof": c.proof.is_some(),
        })
    }
}

#[async_trait::async_trait]
impl StakingRpcServer for StakingRpcImpl {
    async fn get_staking_stats(&self) -> RpcResult<Value> {
        let stats = self.staking_service.get_stats().await;
        Ok(json!({
            "totalValidators": stats.total_validators,
            "activeValidators": stats.active_validators,
            "totalStaked": stats.total_staked,
            "totalStakedPyrax": stats.total_staked as f64 / 100_000_000.0,
            "totalRewards": stats.total_rewards,
            "totalSlashed": stats.total_slashed,
            "totalCheckpoints": stats.total_checkpoints,
            "finalizedCheckpoints": stats.finalized_checkpoints,
            "minStake": stats.min_stake,
            "minStakePyrax": stats.min_stake as f64 / 100_000_000.0,
            "checkpointInterval": stats.checkpoint_interval,
        }))
    }

    async fn get_validator(&self, address: String) -> RpcResult<Value> {
        let addr = Self::parse_address(&address)
            .map_err(|e| jsonrpsee::types::ErrorObject::owned(-32602, e, None::<()>))?;

        match self.staking_service.get_validator(&addr).await {
            Some(v) => Ok(Self::validator_to_json(&v)),
            None => Err(jsonrpsee::types::ErrorObject::owned(
                -32001,
                "Validator not found",
                None::<()>,
            )),
        }
    }

    async fn get_validators(&self) -> RpcResult<Value> {
        let validators = self.staking_service.get_all_validators().await;
        let json_validators: Vec<Value> = validators.iter()
            .map(Self::validator_to_json)
            .collect();
        Ok(json!({
            "count": json_validators.len(),
            "validators": json_validators,
        }))
    }

    async fn get_active_validators(&self) -> RpcResult<Value> {
        let active = self.staking_service.get_active_validators().await;
        let addresses: Vec<String> = active.iter()
            .map(|a| format!("0x{}", hex::encode(&a.0)))
            .collect();
        Ok(json!({
            "count": addresses.len(),
            "validators": addresses,
        }))
    }

    async fn register_validator(&self, address: String, stake: u64) -> RpcResult<Value> {
        let addr = Self::parse_address(&address)
            .map_err(|e| jsonrpsee::types::ErrorObject::owned(-32602, e, None::<()>))?;

        self.staking_service.register_validator(addr, stake).await
            .map_err(|e| jsonrpsee::types::ErrorObject::owned(-32000, e, None::<()>))?;

        Ok(json!({
            "success": true,
            "address": address,
            "stake": stake,
            "stakePyrax": stake as f64 / 100_000_000.0,
        }))
    }

    async fn deposit_stake(&self, address: String, amount: u64) -> RpcResult<Value> {
        let addr = Self::parse_address(&address)
            .map_err(|e| jsonrpsee::types::ErrorObject::owned(-32602, e, None::<()>))?;

        let new_stake = self.staking_service.deposit_stake(addr, amount).await
            .map_err(|e| jsonrpsee::types::ErrorObject::owned(-32000, e, None::<()>))?;

        Ok(json!({
            "success": true,
            "address": address,
            "deposited": amount,
            "newStake": new_stake,
            "newStakePyrax": new_stake as f64 / 100_000_000.0,
        }))
    }

    async fn start_unbonding(&self, address: String, amount: u64) -> RpcResult<Value> {
        let addr = Self::parse_address(&address)
            .map_err(|e| jsonrpsee::types::ErrorObject::owned(-32602, e, None::<()>))?;

        self.staking_service.start_unbonding(addr, amount).await
            .map_err(|e| jsonrpsee::types::ErrorObject::owned(-32000, e, None::<()>))?;

        Ok(json!({
            "success": true,
            "address": address,
            "unbondingAmount": amount,
            "unbondingPeriodDays": 7,
        }))
    }

    async fn complete_unbonding(&self, address: String) -> RpcResult<Value> {
        let addr = Self::parse_address(&address)
            .map_err(|e| jsonrpsee::types::ErrorObject::owned(-32602, e, None::<()>))?;

        let amount = self.staking_service.complete_unbonding(addr).await
            .map_err(|e| jsonrpsee::types::ErrorObject::owned(-32000, e, None::<()>))?;

        Ok(json!({
            "success": true,
            "address": address,
            "withdrawn": amount,
            "withdrawnPyrax": amount as f64 / 100_000_000.0,
        }))
    }

    async fn get_latest_checkpoint(&self) -> RpcResult<Value> {
        match self.staking_service.get_latest_checkpoint().await {
            Some(c) => Ok(Self::checkpoint_to_json(&c)),
            None => Ok(json!({ "checkpoint": null })),
        }
    }

    async fn get_checkpoint(&self, id: u64) -> RpcResult<Value> {
        match self.staking_service.get_checkpoint(id).await {
            Some(c) => Ok(Self::checkpoint_to_json(&c)),
            None => Err(jsonrpsee::types::ErrorObject::owned(
                -32001,
                "Checkpoint not found",
                None::<()>,
            )),
        }
    }

    async fn submit_checkpoint(
        &self,
        validator: String,
        stream_a_height: u64,
        stream_b_height: u64,
        state_root: String,
        utxo_commitment: String,
        proof_data: String,
    ) -> RpcResult<Value> {
        let addr = Self::parse_address(&validator)
            .map_err(|e| jsonrpsee::types::ErrorObject::owned(-32602, e, None::<()>))?;
        let state = Self::parse_h256(&state_root)
            .map_err(|e| jsonrpsee::types::ErrorObject::owned(-32602, e, None::<()>))?;
        let utxo = Self::parse_h256(&utxo_commitment)
            .map_err(|e| jsonrpsee::types::ErrorObject::owned(-32602, e, None::<()>))?;

        let proof_bytes = hex::decode(proof_data.strip_prefix("0x").unwrap_or(&proof_data))
            .map_err(|e| jsonrpsee::types::ErrorObject::owned(-32602, e.to_string(), None::<()>))?;

        // Construct proof
        let proof = Proof {
            proof_type: ProofType::Stark,
            data: proof_bytes,
            public_inputs: vec![state, utxo],
            batch_hash: H256::zero(),
            pre_state_root: H256::zero(),
            post_state_root: state,
            prover: addr,
            generated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            generation_time_ms: 0,
            proof_hash: H256::zero(),
        };

        let checkpoint_id = self.staking_service.submit_checkpoint(
            addr,
            stream_a_height,
            stream_b_height,
            state,
            utxo,
            proof,
        ).await.map_err(|e| jsonrpsee::types::ErrorObject::owned(-32000, e, None::<()>))?;

        Ok(json!({
            "success": true,
            "checkpointId": checkpoint_id,
            "validator": validator,
            "streamAHeight": stream_a_height,
            "streamBHeight": stream_b_height,
        }))
    }

    async fn attest_checkpoint(&self, validator: String, checkpoint_id: u64) -> RpcResult<Value> {
        let addr = Self::parse_address(&validator)
            .map_err(|e| jsonrpsee::types::ErrorObject::owned(-32602, e, None::<()>))?;

        self.staking_service.attest_checkpoint(addr, checkpoint_id).await
            .map_err(|e| jsonrpsee::types::ErrorObject::owned(-32000, e, None::<()>))?;

        Ok(json!({
            "success": true,
            "validator": validator,
            "checkpointId": checkpoint_id,
        }))
    }

    async fn get_stream_c_info(&self) -> RpcResult<Value> {
        let stats = self.staking_service.get_stats().await;
        let latest_checkpoint = self.staking_service.get_latest_checkpoint().await;

        Ok(json!({
            "stream": "C",
            "algorithm": "ZK-STARK",
            "consensusType": "Proof of Stake",
            "rewardPerCheckpoint": 10.0,
            "minStakePyrax": stats.min_stake as f64 / 100_000_000.0,
            "checkpointInterval": stats.checkpoint_interval,
            "activeValidators": stats.active_validators,
            "totalStakedPyrax": stats.total_staked as f64 / 100_000_000.0,
            "totalCheckpoints": stats.total_checkpoints,
            "finalizedCheckpoints": stats.finalized_checkpoints,
            "latestCheckpoint": latest_checkpoint.map(|c| Self::checkpoint_to_json(&c)),
            "unbondingPeriodDays": 7,
            "slashPercent": 10,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_address() {
        let addr = StakingRpcImpl::parse_address("0x1111111111111111111111111111111111111111");
        assert!(addr.is_ok());
        assert_eq!(addr.unwrap().0, [0x11; 20]);
    }

    #[test]
    fn test_parse_h256() {
        let hash = StakingRpcImpl::parse_h256(
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        );
        assert!(hash.is_ok());
        assert!(hash.unwrap().is_zero());
    }
}
