//! Compute Node Management
//!
//! Management of decentralized compute nodes in the network

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::types::{Address, H256};
use super::{MIN_COMPUTE_STAKE, MAX_CONCURRENT_JOBS};

/// Compute node status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeStatus {
    /// Node is registering
    Registering,
    /// Node is active and accepting jobs
    Active,
    /// Node is busy with max jobs
    Busy,
    /// Node is paused by operator
    Paused,
    /// Node is offline
    Offline,
    /// Node is slashed/penalized
    Slashed,
    /// Node is unbonding stake
    Unbonding,
}

/// Hardware capability type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeCapability {
    /// GPU compute (CUDA)
    GpuCuda,
    /// GPU compute (ROCm)
    GpuRocm,
    /// High memory
    HighMemory,
    /// Fast storage (NVMe)
    FastStorage,
    /// Trusted Execution Environment
    TEE,
    /// High bandwidth network
    HighBandwidth,
    /// LLM inference optimized
    LLMOptimized,
    /// Vision model optimized
    VisionOptimized,
}

/// GPU information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    /// GPU model name
    pub model: String,
    /// GPU vendor
    pub vendor: String,
    /// VRAM in MB
    pub vram_mb: u64,
    /// Compute capability (CUDA)
    pub compute_capability: Option<String>,
    /// Driver version
    pub driver_version: String,
    /// CUDA version (if applicable)
    pub cuda_version: Option<String>,
    /// GPU index
    pub index: u32,
}

/// Node hardware specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSpec {
    /// CPU model
    pub cpu_model: String,
    /// CPU cores
    pub cpu_cores: u32,
    /// RAM in MB
    pub ram_mb: u64,
    /// Storage in MB
    pub storage_mb: u64,
    /// Storage type (SSD, NVMe, HDD)
    pub storage_type: String,
    /// GPU information
    pub gpus: Vec<GpuInfo>,
    /// Total GPU memory
    pub total_gpu_memory_mb: u64,
    /// Network bandwidth (Mbps)
    pub bandwidth_mbps: u32,
    /// Geographic region
    pub region: String,
    /// Node capabilities
    pub capabilities: Vec<NodeCapability>,
}

impl NodeSpec {
    /// Calculate total compute power score
    pub fn compute_score(&self) -> u64 {
        let gpu_score: u64 = self.gpus.iter()
            .map(|g| g.vram_mb / 1024) // GB of VRAM
            .sum();
        
        let cpu_score = self.cpu_cores as u64 * 10;
        let mem_score = self.ram_mb / 1024;
        
        gpu_score * 100 + cpu_score + mem_score
    }

    /// Check if meets requirements
    pub fn meets_requirements(&self, req: &super::job::ResourceRequirements) -> bool {
        if self.total_gpu_memory_mb < req.gpu_memory_mb {
            return false;
        }
        if self.cpu_cores < req.cpu_cores {
            return false;
        }
        if self.ram_mb < req.ram_mb {
            return false;
        }
        if self.storage_mb < req.storage_mb {
            return false;
        }
        if let Some(ref gpu_type) = req.gpu_type {
            if !self.gpus.iter().any(|g| g.model.contains(gpu_type)) {
                return false;
            }
        }
        true
    }
}

/// Compute node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeNode {
    /// Node address (operator)
    pub address: Address,
    /// Node endpoint URL
    pub endpoint: String,
    /// Hardware specification
    pub spec: NodeSpec,
    /// Current status
    pub status: NodeStatus,
    /// Staked amount
    pub stake: u64,
    /// Reputation score (0-100)
    pub reputation: u32,
    /// Total jobs completed
    pub jobs_completed: u64,
    /// Total jobs failed
    pub jobs_failed: u64,
    /// Total earnings (PYRAX)
    pub total_earnings: u64,
    /// Current active jobs
    pub active_jobs: u32,
    /// Maximum concurrent jobs
    pub max_concurrent_jobs: u32,
    /// Price per compute unit (PYRAX per hour)
    pub price_per_hour: u64,
    /// Registration timestamp
    pub registered_at: u64,
    /// Last heartbeat timestamp
    pub last_heartbeat: u64,
    /// Uptime percentage (0-100)
    pub uptime_percent: u32,
    /// Average job completion time (seconds)
    pub avg_completion_time: u64,
    /// Slash count
    pub slash_count: u32,
    /// Is verified by platform
    pub verified: bool,
}

impl ComputeNode {
    /// Create new compute node
    pub fn new(address: Address, endpoint: String, spec: NodeSpec, stake: u64) -> Self {
        Self {
            address,
            endpoint,
            spec,
            status: NodeStatus::Registering,
            stake,
            reputation: 50, // Start with neutral reputation
            jobs_completed: 0,
            jobs_failed: 0,
            total_earnings: 0,
            active_jobs: 0,
            max_concurrent_jobs: MAX_CONCURRENT_JOBS,
            price_per_hour: 0,
            registered_at: current_timestamp(),
            last_heartbeat: current_timestamp(),
            uptime_percent: 100,
            avg_completion_time: 0,
            slash_count: 0,
            verified: false,
        }
    }

    /// Activate node
    pub fn activate(&mut self) -> Result<(), NodeError> {
        if self.stake < MIN_COMPUTE_STAKE {
            return Err(NodeError::InsufficientStake {
                required: MIN_COMPUTE_STAKE,
                available: self.stake,
            });
        }
        self.status = NodeStatus::Active;
        self.last_heartbeat = current_timestamp();
        Ok(())
    }

    /// Pause node
    pub fn pause(&mut self) {
        if self.status == NodeStatus::Active || self.status == NodeStatus::Busy {
            self.status = NodeStatus::Paused;
        }
    }

    /// Resume node
    pub fn resume(&mut self) {
        if self.status == NodeStatus::Paused {
            self.status = if self.active_jobs >= self.max_concurrent_jobs {
                NodeStatus::Busy
            } else {
                NodeStatus::Active
            };
        }
    }

    /// Record heartbeat
    pub fn heartbeat(&mut self) {
        self.last_heartbeat = current_timestamp();
        if self.status == NodeStatus::Offline {
            self.status = if self.active_jobs >= self.max_concurrent_jobs {
                NodeStatus::Busy
            } else {
                NodeStatus::Active
            };
        }
    }

    /// Check if node is online (heartbeat within 5 minutes)
    pub fn is_online(&self) -> bool {
        current_timestamp() - self.last_heartbeat < 300
    }

    /// Accept job
    pub fn accept_job(&mut self) -> Result<(), NodeError> {
        if self.status != NodeStatus::Active {
            return Err(NodeError::NotActive);
        }
        if self.active_jobs >= self.max_concurrent_jobs {
            return Err(NodeError::AtCapacity);
        }

        self.active_jobs += 1;
        if self.active_jobs >= self.max_concurrent_jobs {
            self.status = NodeStatus::Busy;
        }
        Ok(())
    }

    /// Complete job
    pub fn complete_job(&mut self, duration_secs: u64, earnings: u64) {
        self.active_jobs = self.active_jobs.saturating_sub(1);
        self.jobs_completed += 1;
        self.total_earnings += earnings;

        // Update average completion time
        let total_time = self.avg_completion_time * (self.jobs_completed - 1) + duration_secs;
        self.avg_completion_time = total_time / self.jobs_completed;

        // Update reputation (increase for successful job)
        self.reputation = (self.reputation + 1).min(100);

        // Update status
        if self.status == NodeStatus::Busy && self.active_jobs < self.max_concurrent_jobs {
            self.status = NodeStatus::Active;
        }
    }

    /// Fail job
    pub fn fail_job(&mut self) {
        self.active_jobs = self.active_jobs.saturating_sub(1);
        self.jobs_failed += 1;

        // Update reputation (decrease for failed job)
        self.reputation = self.reputation.saturating_sub(2);

        // Update status
        if self.status == NodeStatus::Busy && self.active_jobs < self.max_concurrent_jobs {
            self.status = NodeStatus::Active;
        }
    }

    /// Slash node
    pub fn slash(&mut self, slash_percent: u8) -> u64 {
        let slash_amount = (self.stake as u128 * slash_percent as u128 / 100) as u64;
        self.stake = self.stake.saturating_sub(slash_amount);
        self.slash_count += 1;
        self.reputation = self.reputation.saturating_sub(10);

        if self.slash_count >= 3 || self.stake < MIN_COMPUTE_STAKE {
            self.status = NodeStatus::Slashed;
        }

        slash_amount
    }

    /// Add stake
    pub fn add_stake(&mut self, amount: u64) {
        self.stake += amount;
    }

    /// Request unstake
    pub fn request_unstake(&mut self) -> Result<(), NodeError> {
        if self.active_jobs > 0 {
            return Err(NodeError::HasActiveJobs);
        }
        self.status = NodeStatus::Unbonding;
        Ok(())
    }

    /// Set price
    pub fn set_price(&mut self, price_per_hour: u64) {
        self.price_per_hour = price_per_hour;
    }

    /// Calculate job price
    pub fn calculate_price(&self, duration_secs: u64) -> u64 {
        (self.price_per_hour * duration_secs) / 3600
    }

    /// Get success rate
    pub fn success_rate(&self) -> f64 {
        let total = self.jobs_completed + self.jobs_failed;
        if total == 0 {
            1.0
        } else {
            self.jobs_completed as f64 / total as f64
        }
    }
}

/// Node errors
#[derive(Debug, thiserror::Error)]
pub enum NodeError {
    #[error("Insufficient stake: required {required}, available {available}")]
    InsufficientStake { required: u64, available: u64 },

    #[error("Node not active")]
    NotActive,

    #[error("Node at capacity")]
    AtCapacity,

    #[error("Node has active jobs")]
    HasActiveJobs,

    #[error("Node not found: {0:?}")]
    NotFound(Address),

    #[error("Node already registered: {0:?}")]
    AlreadyRegistered(Address),

    #[error("Invalid endpoint: {0}")]
    InvalidEndpoint(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),
}

/// Compute node pool
pub struct ComputePool {
    /// Nodes by address
    nodes: Arc<RwLock<HashMap<Address, ComputeNode>>>,
    /// Nodes by region
    by_region: Arc<RwLock<HashMap<String, Vec<Address>>>>,
    /// Nodes by capability
    by_capability: Arc<RwLock<HashMap<NodeCapability, Vec<Address>>>>,
    /// Active nodes
    active_nodes: Arc<RwLock<Vec<Address>>>,
    /// Total stake in pool
    total_stake: Arc<RwLock<u64>>,
}

impl ComputePool {
    /// Create new compute pool
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(RwLock::new(HashMap::new())),
            by_region: Arc::new(RwLock::new(HashMap::new())),
            by_capability: Arc::new(RwLock::new(HashMap::new())),
            active_nodes: Arc::new(RwLock::new(Vec::new())),
            total_stake: Arc::new(RwLock::new(0)),
        }
    }

    /// Register new node
    pub fn register(&self, node: ComputeNode) -> Result<(), NodeError> {
        let address = node.address;

        {
            let mut nodes = self.nodes.write();
            if nodes.contains_key(&address) {
                return Err(NodeError::AlreadyRegistered(address));
            }

            // Index by region
            self.by_region.write()
                .entry(node.spec.region.clone())
                .or_insert_with(Vec::new)
                .push(address);

            // Index by capabilities
            for cap in &node.spec.capabilities {
                self.by_capability.write()
                    .entry(*cap)
                    .or_insert_with(Vec::new)
                    .push(address);
            }

            *self.total_stake.write() += node.stake;
            nodes.insert(address, node);
        }

        Ok(())
    }

    /// Get node by address
    pub fn get(&self, address: &Address) -> Option<ComputeNode> {
        self.nodes.read().get(address).cloned()
    }

    /// Update node
    pub fn update(&self, address: &Address, f: impl FnOnce(&mut ComputeNode)) -> Result<(), NodeError> {
        let mut nodes = self.nodes.write();
        let node = nodes.get_mut(address)
            .ok_or_else(|| NodeError::NotFound(*address))?;
        f(node);
        Ok(())
    }

    /// Activate node
    pub fn activate(&self, address: &Address) -> Result<(), NodeError> {
        let mut nodes = self.nodes.write();
        let node = nodes.get_mut(address)
            .ok_or_else(|| NodeError::NotFound(*address))?;
        
        node.activate()?;
        
        drop(nodes);
        self.active_nodes.write().push(*address);
        Ok(())
    }

    /// Record heartbeat
    pub fn heartbeat(&self, address: &Address) -> Result<(), NodeError> {
        self.update(address, |n| n.heartbeat())
    }

    /// Get available nodes for job requirements
    pub fn get_available(&self, requirements: &super::job::ResourceRequirements) -> Vec<ComputeNode> {
        self.nodes.read()
            .values()
            .filter(|n| {
                n.status == NodeStatus::Active &&
                n.is_online() &&
                n.spec.meets_requirements(requirements)
            })
            .cloned()
            .collect()
    }

    /// Get nodes by region
    pub fn get_by_region(&self, region: &str) -> Vec<ComputeNode> {
        let addresses = self.by_region.read()
            .get(region)
            .cloned()
            .unwrap_or_default();

        let nodes = self.nodes.read();
        addresses.iter()
            .filter_map(|a| nodes.get(a).cloned())
            .collect()
    }

    /// Get nodes by capability
    pub fn get_by_capability(&self, capability: NodeCapability) -> Vec<ComputeNode> {
        let addresses = self.by_capability.read()
            .get(&capability)
            .cloned()
            .unwrap_or_default();

        let nodes = self.nodes.read();
        addresses.iter()
            .filter_map(|a| nodes.get(a).cloned())
            .collect()
    }

    /// Check offline nodes and update status
    pub fn check_offline(&self) -> Vec<Address> {
        let mut offline = Vec::new();
        let mut nodes = self.nodes.write();

        for (addr, node) in nodes.iter_mut() {
            if !node.is_online() && node.status == NodeStatus::Active {
                node.status = NodeStatus::Offline;
                offline.push(*addr);
            }
        }

        // Remove from active list
        self.active_nodes.write().retain(|a| !offline.contains(a));

        offline
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        let nodes = self.nodes.read();
        
        let mut by_status = HashMap::new();
        let mut total_compute_power = 0u64;
        let mut total_gpu_memory = 0u64;

        for node in nodes.values() {
            *by_status.entry(node.status).or_insert(0u64) += 1;
            if node.status == NodeStatus::Active || node.status == NodeStatus::Busy {
                total_compute_power += node.spec.compute_score();
                total_gpu_memory += node.spec.total_gpu_memory_mb;
            }
        }

        PoolStats {
            total_nodes: nodes.len() as u64,
            active_nodes: self.active_nodes.read().len() as u64,
            total_stake: *self.total_stake.read(),
            total_compute_power,
            total_gpu_memory_gb: total_gpu_memory / 1024,
            by_status,
        }
    }
}

impl Default for ComputePool {
    fn default() -> Self {
        Self::new()
    }
}

/// Pool statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStats {
    pub total_nodes: u64,
    pub active_nodes: u64,
    pub total_stake: u64,
    pub total_compute_power: u64,
    pub total_gpu_memory_gb: u64,
    pub by_status: HashMap<NodeStatus, u64>,
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

    fn make_node_spec() -> NodeSpec {
        NodeSpec {
            cpu_model: "AMD EPYC 7763".to_string(),
            cpu_cores: 64,
            ram_mb: 256 * 1024,
            storage_mb: 2 * 1024 * 1024,
            storage_type: "NVMe".to_string(),
            gpus: vec![
                GpuInfo {
                    model: "NVIDIA A100".to_string(),
                    vendor: "NVIDIA".to_string(),
                    vram_mb: 80 * 1024,
                    compute_capability: Some("8.0".to_string()),
                    driver_version: "535.104.05".to_string(),
                    cuda_version: Some("12.2".to_string()),
                    index: 0,
                }
            ],
            total_gpu_memory_mb: 80 * 1024,
            bandwidth_mbps: 10000,
            region: "us-east".to_string(),
            capabilities: vec![NodeCapability::GpuCuda, NodeCapability::LLMOptimized],
        }
    }

    #[test]
    fn test_node_creation() {
        let addr = Address([1u8; 20]);
        let spec = make_node_spec();
        let node = ComputeNode::new(addr, "https://node1.example.com".to_string(), spec, MIN_COMPUTE_STAKE);

        assert_eq!(node.status, NodeStatus::Registering);
        assert_eq!(node.reputation, 50);
    }

    #[test]
    fn test_node_activation() {
        let addr = Address([1u8; 20]);
        let spec = make_node_spec();
        let mut node = ComputeNode::new(addr, "https://node1.example.com".to_string(), spec, MIN_COMPUTE_STAKE);

        assert!(node.activate().is_ok());
        assert_eq!(node.status, NodeStatus::Active);
    }

    #[test]
    fn test_node_job_lifecycle() {
        let addr = Address([1u8; 20]);
        let spec = make_node_spec();
        let mut node = ComputeNode::new(addr, "https://node1.example.com".to_string(), spec, MIN_COMPUTE_STAKE);
        node.activate().unwrap();

        // Accept job
        assert!(node.accept_job().is_ok());
        assert_eq!(node.active_jobs, 1);

        // Complete job
        node.complete_job(60, 1_000_000);
        assert_eq!(node.jobs_completed, 1);
        assert_eq!(node.active_jobs, 0);
        assert!(node.reputation > 50);
    }

    #[test]
    fn test_compute_pool() {
        let pool = ComputePool::new();
        let addr = Address([1u8; 20]);
        let spec = make_node_spec();
        let node = ComputeNode::new(addr, "https://node1.example.com".to_string(), spec, MIN_COMPUTE_STAKE);

        assert!(pool.register(node).is_ok());
        assert!(pool.get(&addr).is_some());
        assert!(pool.activate(&addr).is_ok());

        let stats = pool.stats();
        assert_eq!(stats.total_nodes, 1);
    }

    #[test]
    fn test_node_slashing() {
        let addr = Address([1u8; 20]);
        let spec = make_node_spec();
        let mut node = ComputeNode::new(addr, "https://node1.example.com".to_string(), spec, MIN_COMPUTE_STAKE);

        let slashed = node.slash(10);
        assert!(slashed > 0);
        assert!(node.stake < MIN_COMPUTE_STAKE);
        assert_eq!(node.slash_count, 1);
    }
}
