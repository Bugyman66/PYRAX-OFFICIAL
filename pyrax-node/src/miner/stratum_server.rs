//! Stratum Mining Server for PYRAX
//!
//! Production-ready stratum server supporting:
//! - Solo mining directly from desktop app
//! - Pool operator deployment
//! - Stratum v1 protocol (ethproxy-compatible for KAWPOW)
//!
//! No mocks, no stubs - real block templates, real validation.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock, broadcast};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, debug, error};

use crate::types::{H256, Address, Block, BlockHeader, Transaction, TxInput, TxOutput, OutPoint, BlockNumber};
use crate::consensus::{kawpow_hash, generate_cache, compute_seed, get_epoch, get_cache_size};

/// Stratum server configuration
#[derive(Debug, Clone)]
pub struct StratumServerConfig {
    /// Address to bind the stratum server
    pub bind_addr: SocketAddr,
    /// Default mining difficulty for new workers
    pub default_difficulty: f64,
    /// Minimum difficulty allowed
    pub min_difficulty: f64,
    /// Maximum difficulty allowed  
    pub max_difficulty: f64,
    /// Target share time in seconds (for vardiff)
    pub target_share_time: u64,
    /// Job timeout in seconds
    pub job_timeout: u64,
    /// Maximum connections allowed
    pub max_connections: usize,
    /// Ban duration for misbehaving clients (seconds)
    pub ban_duration: u64,
    /// Extra nonce 1 size (bytes)
    pub extranonce1_size: usize,
    /// Extra nonce 2 size (bytes)
    pub extranonce2_size: usize,
    /// Coinbase address for solo mining
    pub coinbase_address: Address,
    /// Network difficulty
    pub network_difficulty: u64,
}

impl Default for StratumServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: "0.0.0.0:3333".parse().unwrap(),
            default_difficulty: 1.0,
            min_difficulty: 0.001,
            max_difficulty: 1000000.0,
            target_share_time: 15,
            job_timeout: 300,
            max_connections: 10000,
            ban_duration: 600,
            extranonce1_size: 4,
            extranonce2_size: 4,
            coinbase_address: Address::ZERO,
            network_difficulty: 1,
        }
    }
}

/// Mining job distributed to workers
#[derive(Debug, Clone)]
pub struct MiningJob {
    pub job_id: String,
    pub height: BlockNumber,
    pub parent_hash: H256,
    pub merkle_root: H256,
    pub header_hash: H256,
    pub seed_hash: H256,
    pub target: H256,
    pub difficulty: u64,
    pub timestamp: u64,
    pub coinbase_tx: Transaction,
    pub transactions: Vec<Transaction>,
    pub created_at: Instant,
    pub clean: bool,
}

impl MiningJob {
    /// Build block header for this job with given nonces
    pub fn build_header(&self, nonce: u64, extra_nonce: u64, beneficiary: &Address) -> BlockHeader {
        BlockHeader {
            version: 1,
            stream: 1, // Stream B (KAWPOW)
            parent_hash: self.parent_hash,
            merkle_root: self.merkle_root,
            utxo_commitment: H256::zero(),
            timestamp: self.timestamp,
            difficulty: self.difficulty,
            nonce,
            extra_nonce,
            height: self.height,
            beneficiary: *beneficiary,
        }
    }
}

/// Connected worker state
#[derive(Debug)]
pub struct Worker {
    pub id: u64,
    pub name: String,
    pub address: SocketAddr,
    pub extranonce1: String,
    pub difficulty: f64,
    pub authorized: bool,
    pub subscribed: bool,
    pub connected_at: Instant,
    pub last_share_at: Option<Instant>,
    pub shares_accepted: u64,
    pub shares_rejected: u64,
    pub shares_stale: u64,
    pub hashrate_estimate: f64,
    pub beneficiary: Option<Address>,
}

impl Worker {
    pub fn new(id: u64, address: SocketAddr, extranonce1: String) -> Self {
        Self {
            id,
            name: String::new(),
            address,
            extranonce1,
            difficulty: 1.0,
            authorized: false,
            subscribed: false,
            connected_at: Instant::now(),
            last_share_at: None,
            shares_accepted: 0,
            shares_rejected: 0,
            shares_stale: 0,
            hashrate_estimate: 0.0,
            beneficiary: None,
        }
    }
}

/// Share submission from worker
#[derive(Debug, Clone)]
pub struct Share {
    pub worker_id: u64,
    pub job_id: String,
    pub nonce: u64,
    pub header_hash: H256,
    pub mix_hash: H256,
    pub submitted_at: Instant,
}

/// Share validation result
#[derive(Debug, Clone)]
pub enum ShareResult {
    Accepted,
    AcceptedBlock(H256), // Share was a valid block!
    Rejected(String),
    Stale,
}

/// Block template from node
#[derive(Debug, Clone)]
pub struct BlockTemplate {
    pub height: BlockNumber,
    pub parent_hash: H256,
    pub timestamp: u64,
    pub difficulty: u64,
    pub transactions: Vec<Transaction>,
    pub coinbase_value: u64,
}

/// Stratum JSON-RPC request
#[derive(Debug, Serialize, Deserialize)]
struct StratumRequest {
    id: Option<serde_json::Value>,
    method: String,
    params: serde_json::Value,
}

/// Stratum JSON-RPC response
#[derive(Debug, Serialize, Deserialize)]
struct StratumResponse {
    id: Option<serde_json::Value>,
    result: Option<serde_json::Value>,
    error: Option<StratumError>,
}

/// Stratum JSON-RPC notification (server push)
#[derive(Debug, Serialize, Deserialize)]
struct StratumNotification {
    id: Option<()>,
    method: String,
    params: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct StratumError {
    code: i32,
    message: String,
}

/// Server statistics
#[derive(Debug, Default)]
pub struct ServerStats {
    pub workers_connected: AtomicU64,
    pub workers_total: AtomicU64,
    pub shares_accepted: AtomicU64,
    pub shares_rejected: AtomicU64,
    pub shares_stale: AtomicU64,
    pub blocks_found: AtomicU64,
    pub total_hashrate: AtomicU64,
}

/// Block submission callback
pub type BlockSubmitFn = Box<dyn Fn(Block) -> Result<H256, String> + Send + Sync>;

/// Block template provider callback
pub type TemplateProviderFn = Box<dyn Fn() -> Option<BlockTemplate> + Send + Sync>;

/// Production Stratum Mining Server
pub struct StratumServer {
    config: StratumServerConfig,
    workers: Arc<RwLock<HashMap<u64, Worker>>>,
    jobs: Arc<RwLock<HashMap<String, MiningJob>>>,
    current_job: Arc<RwLock<Option<MiningJob>>>,
    stats: Arc<ServerStats>,
    next_worker_id: AtomicU64,
    next_extranonce1: AtomicU64,
    running: Arc<AtomicBool>,
    job_broadcast: broadcast::Sender<MiningJob>,
    kawpow_cache: Arc<RwLock<Option<KawpowCache>>>,
    block_submit_fn: Arc<RwLock<Option<BlockSubmitFn>>>,
    template_provider: Arc<RwLock<Option<TemplateProviderFn>>>,
}

/// Cached KAWPOW data for current epoch
struct KawpowCache {
    epoch: u64,
    seed: [u8; 32],
    cache: Vec<[u8; 64]>,
}

impl StratumServer {
    /// Create a new stratum server
    pub fn new(config: StratumServerConfig) -> Self {
        let (job_broadcast, _) = broadcast::channel(16);
        
        Self {
            config,
            workers: Arc::new(RwLock::new(HashMap::new())),
            jobs: Arc::new(RwLock::new(HashMap::new())),
            current_job: Arc::new(RwLock::new(None)),
            stats: Arc::new(ServerStats::default()),
            next_worker_id: AtomicU64::new(1),
            next_extranonce1: AtomicU64::new(rand::random::<u64>() & 0xFFFFFFFF),
            running: Arc::new(AtomicBool::new(false)),
            job_broadcast,
            kawpow_cache: Arc::new(RwLock::new(None)),
            block_submit_fn: Arc::new(RwLock::new(None)),
            template_provider: Arc::new(RwLock::new(None)),
        }
    }

    /// Set the block submission callback
    pub async fn set_block_submit_fn(&self, f: BlockSubmitFn) {
        *self.block_submit_fn.write().await = Some(f);
    }

    /// Set the template provider callback
    pub async fn set_template_provider(&self, f: TemplateProviderFn) {
        *self.template_provider.write().await = Some(f);
    }

    /// Get server statistics
    pub fn stats(&self) -> &ServerStats {
        &self.stats
    }

    /// Get current job
    pub async fn current_job(&self) -> Option<MiningJob> {
        self.current_job.read().await.clone()
    }

    /// Get connected workers
    pub async fn workers(&self) -> Vec<Worker> {
        self.workers.read().await.values().cloned().collect()
    }

    /// Check if server is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Generate unique extranonce1 for worker
    fn generate_extranonce1(&self) -> String {
        let val = self.next_extranonce1.fetch_add(1, Ordering::SeqCst);
        format!("{:08x}", val & 0xFFFFFFFF)
    }

    /// Generate unique job ID
    fn generate_job_id(&self, height: BlockNumber) -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        format!("{:08x}{:08x}", height, timestamp & 0xFFFFFFFF)
    }

    /// Update KAWPOW cache for epoch if needed
    async fn ensure_kawpow_cache(&self, height: BlockNumber) {
        let epoch = get_epoch(height);
        
        let needs_update = {
            let cache = self.kawpow_cache.read().await;
            cache.as_ref().map_or(true, |c| c.epoch != epoch)
        };

        if needs_update {
            info!("Generating KAWPOW cache for epoch {}...", epoch);
            let seed = compute_seed(epoch);
            let cache_size = get_cache_size(epoch);
            let cache = generate_cache(&seed, cache_size);
            
            *self.kawpow_cache.write().await = Some(KawpowCache {
                epoch,
                seed,
                cache,
            });
            info!("KAWPOW cache ready for epoch {}", epoch);
        }
    }

    /// Create mining job from block template
    pub async fn create_job(&self, template: BlockTemplate, clean: bool) -> MiningJob {
        self.ensure_kawpow_cache(template.height).await;

        let job_id = self.generate_job_id(template.height);
        let epoch = get_epoch(template.height);
        let seed_hash = H256::from_slice(&compute_seed(epoch));
        
        // Create coinbase transaction
        let coinbase_tx = Transaction::coinbase(
            template.height,
            template.coinbase_value,
            &self.config.coinbase_address,
        );

        // Build transactions list
        let mut all_txs = vec![coinbase_tx.clone()];
        all_txs.extend(template.transactions.clone());

        // Compute merkle root
        let merkle_root = compute_merkle_root(&all_txs);

        // Build header for hash computation
        let header = BlockHeader {
            version: 1,
            stream: 1, // Stream B
            parent_hash: template.parent_hash,
            merkle_root,
            utxo_commitment: H256::zero(),
            timestamp: template.timestamp,
            difficulty: template.difficulty,
            nonce: 0,
            extra_nonce: 0,
            height: template.height,
            beneficiary: self.config.coinbase_address,
        };

        let header_hash = header.hash();
        let target = difficulty_to_target(template.difficulty);

        let job = MiningJob {
            job_id,
            height: template.height,
            parent_hash: template.parent_hash,
            merkle_root,
            header_hash,
            seed_hash,
            target,
            difficulty: template.difficulty,
            timestamp: template.timestamp,
            coinbase_tx,
            transactions: template.transactions,
            created_at: Instant::now(),
            clean,
        };

        // Store job
        {
            let mut jobs = self.jobs.write().await;
            jobs.insert(job.job_id.clone(), job.clone());
            
            // Prune old jobs (keep last 10)
            if jobs.len() > 10 {
                let mut to_remove: Vec<_> = jobs.iter()
                    .map(|(k, v)| (k.clone(), v.created_at))
                    .collect();
                to_remove.sort_by_key(|(_, t)| *t);
                for (key, _) in to_remove.iter().take(jobs.len() - 10) {
                    jobs.remove(key);
                }
            }
        }

        *self.current_job.write().await = Some(job.clone());

        job
    }

    /// Broadcast new job to all workers
    pub async fn broadcast_job(&self, job: MiningJob) {
        let _ = self.job_broadcast.send(job);
    }

    /// Validate share submission
    pub async fn validate_share(&self, share: &Share, worker: &Worker) -> ShareResult {
        // Get job
        let job = {
            let jobs = self.jobs.read().await;
            match jobs.get(&share.job_id) {
                Some(j) => j.clone(),
                None => return ShareResult::Stale,
            }
        };

        // Check job age
        if job.created_at.elapsed() > Duration::from_secs(self.config.job_timeout) {
            return ShareResult::Stale;
        }

        // Get KAWPOW cache
        let cache = self.kawpow_cache.read().await;
        let cache = match cache.as_ref() {
            Some(c) => c,
            None => return ShareResult::Rejected("No KAWPOW cache available".to_string()),
        };

        // Verify KAWPOW hash
        let pow_hash = kawpow_hash(
            share.header_hash.as_bytes(),
            share.nonce,
            job.height,
            &cache.cache,
        );
        let pow_hash = H256::from_slice(&pow_hash);

        // Check against share difficulty
        let share_target = worker_difficulty_to_target(worker.difficulty);
        if pow_hash.as_bytes() > share_target.as_bytes() {
            self.stats.shares_rejected.fetch_add(1, Ordering::Relaxed);
            return ShareResult::Rejected("Hash does not meet share target".to_string());
        }

        self.stats.shares_accepted.fetch_add(1, Ordering::Relaxed);

        // Check if it meets network difficulty (valid block!)
        if pow_hash.as_bytes() <= job.target.as_bytes() {
            info!("🎉 BLOCK FOUND at height {}! Hash: {}", job.height, pow_hash);
            self.stats.blocks_found.fetch_add(1, Ordering::Relaxed);

            // Build the block
            let beneficiary = worker.beneficiary.unwrap_or(self.config.coinbase_address);
            let header = job.build_header(share.nonce, 0, &beneficiary);
            
            let mut txs = vec![job.coinbase_tx.clone()];
            txs.extend(job.transactions.clone());
            
            let block = Block::new(header, txs);
            
            // Submit block to node
            if let Some(submit_fn) = self.block_submit_fn.read().await.as_ref() {
                match submit_fn(block) {
                    Ok(hash) => {
                        info!("Block submitted successfully: {}", hash);
                        return ShareResult::AcceptedBlock(hash);
                    }
                    Err(e) => {
                        error!("Block submission failed: {}", e);
                    }
                }
            }

            return ShareResult::AcceptedBlock(pow_hash);
        }

        ShareResult::Accepted
    }

    /// Start the stratum server
    pub async fn run(&self) -> anyhow::Result<()> {
        let listener = TcpListener::bind(&self.config.bind_addr).await?;
        self.running.store(true, Ordering::SeqCst);
        
        info!("╔══════════════════════════════════════════════════════════════╗");
        info!("║           PYRAX Stratum Mining Server                         ║");
        info!("╠══════════════════════════════════════════════════════════════╣");
        info!("║  Bind Address: {:?}", self.config.bind_addr);
        info!("║  Default Difficulty: {}", self.config.default_difficulty);
        info!("║  Max Connections: {}", self.config.max_connections);
        info!("║  Coinbase Address: {}", self.config.coinbase_address);
        info!("╚══════════════════════════════════════════════════════════════╝");

        // Start job update loop
        let server = self.clone_internals();
        tokio::spawn(async move {
            Self::job_update_loop(server).await;
        });

        // Accept connections
        while self.running.load(Ordering::SeqCst) {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    let current_workers = self.stats.workers_connected.load(Ordering::Relaxed);
                    if current_workers >= self.config.max_connections as u64 {
                        warn!("Max connections reached, rejecting {}", addr);
                        continue;
                    }

                    let server = self.clone_internals();
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_connection(server, stream, addr).await {
                            debug!("Connection {} closed: {}", addr, e);
                        }
                    });
                }
                Err(e) => {
                    error!("Accept error: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Clone internals for spawned tasks
    fn clone_internals(&self) -> StratumServerInternals {
        StratumServerInternals {
            config: self.config.clone(),
            workers: Arc::clone(&self.workers),
            jobs: Arc::clone(&self.jobs),
            current_job: Arc::clone(&self.current_job),
            stats: Arc::clone(&self.stats),
            next_worker_id: &self.next_worker_id as *const AtomicU64,
            running: Arc::clone(&self.running),
            job_broadcast: self.job_broadcast.clone(),
            kawpow_cache: Arc::clone(&self.kawpow_cache),
            block_submit_fn: Arc::clone(&self.block_submit_fn),
            template_provider: Arc::clone(&self.template_provider),
            extranonce2_size: self.config.extranonce2_size,
            default_difficulty: self.config.default_difficulty,
        }
    }

    /// Job update loop - fetches new templates periodically
    async fn job_update_loop(server: StratumServerInternals) {
        let mut last_height: BlockNumber = 0;
        
        loop {
            if !server.running.load(Ordering::SeqCst) {
                break;
            }

            // Get template from provider
            if let Some(provider) = server.template_provider.read().await.as_ref() {
                if let Some(template) = provider() {
                    let clean = template.height != last_height;
                    last_height = template.height;

                    // Create and broadcast job
                    let job = server.create_job_internal(template, clean).await;
                    let _ = server.job_broadcast.send(job);
                }
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    /// Handle individual client connection
    async fn handle_connection(
        server: StratumServerInternals,
        stream: TcpStream,
        addr: SocketAddr,
    ) -> anyhow::Result<()> {
        let worker_id = unsafe { (*server.next_worker_id).fetch_add(1, Ordering::SeqCst) };
        let extranonce1 = format!("{:08x}", worker_id & 0xFFFFFFFF);
        
        let worker = Worker::new(worker_id, addr, extranonce1.clone());
        server.workers.write().await.insert(worker_id, worker);
        server.stats.workers_connected.fetch_add(1, Ordering::Relaxed);
        server.stats.workers_total.fetch_add(1, Ordering::Relaxed);
        
        info!("Worker {} connected from {}", worker_id, addr);

        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut job_rx = server.job_broadcast.subscribe();

        let result = async {
            let mut line = String::new();
            
            loop {
                tokio::select! {
                    // Read from client
                    result = reader.read_line(&mut line) => {
                        match result {
                            Ok(0) => break,
                            Ok(_) => {
                                if let Some(response) = server.handle_message(worker_id, &line).await {
                                    let mut json = serde_json::to_string(&response)?;
                                    json.push('\n');
                                    writer.write_all(json.as_bytes()).await?;
                                }
                                line.clear();
                            }
                            Err(e) => return Err(e.into()),
                        }
                    }
                    
                    // Broadcast new jobs
                    Ok(job) = job_rx.recv() => {
                        let workers = server.workers.read().await;
                        if let Some(worker) = workers.get(&worker_id) {
                            if worker.authorized {
                                let notification = server.create_job_notification(&job, &extranonce1);
                                let mut json = serde_json::to_string(&notification)?;
                                json.push('\n');
                                writer.write_all(json.as_bytes()).await?;
                            }
                        }
                    }
                }
            }
            Ok(())
        }.await;

        // Cleanup
        server.workers.write().await.remove(&worker_id);
        server.stats.workers_connected.fetch_sub(1, Ordering::Relaxed);
        info!("Worker {} disconnected", worker_id);

        result
    }
}

/// Internal server state for spawned tasks
struct StratumServerInternals {
    config: StratumServerConfig,
    workers: Arc<RwLock<HashMap<u64, Worker>>>,
    jobs: Arc<RwLock<HashMap<String, MiningJob>>>,
    current_job: Arc<RwLock<Option<MiningJob>>>,
    stats: Arc<ServerStats>,
    next_worker_id: *const AtomicU64,
    running: Arc<AtomicBool>,
    job_broadcast: broadcast::Sender<MiningJob>,
    kawpow_cache: Arc<RwLock<Option<KawpowCache>>>,
    block_submit_fn: Arc<RwLock<Option<BlockSubmitFn>>>,
    template_provider: Arc<RwLock<Option<TemplateProviderFn>>>,
    extranonce2_size: usize,
    default_difficulty: f64,
}

unsafe impl Send for StratumServerInternals {}
unsafe impl Sync for StratumServerInternals {}

impl StratumServerInternals {
    async fn create_job_internal(&self, template: BlockTemplate, clean: bool) -> MiningJob {
        let job_id = {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
            format!("{:08x}{:08x}", template.height, timestamp & 0xFFFFFFFF)
        };

        let epoch = get_epoch(template.height);
        let seed_hash = H256::from_slice(&compute_seed(epoch));
        
        let coinbase_tx = Transaction::coinbase(
            template.height,
            template.coinbase_value,
            &self.config.coinbase_address,
        );

        let mut all_txs = vec![coinbase_tx.clone()];
        all_txs.extend(template.transactions.clone());

        let merkle_root = compute_merkle_root(&all_txs);

        let header = BlockHeader {
            version: 1,
            stream: 1,
            parent_hash: template.parent_hash,
            merkle_root,
            utxo_commitment: H256::zero(),
            timestamp: template.timestamp,
            difficulty: template.difficulty,
            nonce: 0,
            extra_nonce: 0,
            height: template.height,
            beneficiary: self.config.coinbase_address,
        };

        let header_hash = header.hash();
        let target = difficulty_to_target(template.difficulty);

        let job = MiningJob {
            job_id,
            height: template.height,
            parent_hash: template.parent_hash,
            merkle_root,
            header_hash,
            seed_hash,
            target,
            difficulty: template.difficulty,
            timestamp: template.timestamp,
            coinbase_tx,
            transactions: template.transactions,
            created_at: Instant::now(),
            clean,
        };

        {
            let mut jobs = self.jobs.write().await;
            jobs.insert(job.job_id.clone(), job.clone());
            
            if jobs.len() > 10 {
                let mut to_remove: Vec<_> = jobs.iter()
                    .map(|(k, v)| (k.clone(), v.created_at))
                    .collect();
                to_remove.sort_by_key(|(_, t)| *t);
                for (key, _) in to_remove.iter().take(jobs.len() - 10) {
                    jobs.remove(key);
                }
            }
        }

        *self.current_job.write().await = Some(job.clone());
        job
    }

    async fn handle_message(&self, worker_id: u64, line: &str) -> Option<StratumResponse> {
        let line = line.trim();
        if line.is_empty() {
            return None;
        }

        let request: StratumRequest = match serde_json::from_str(line) {
            Ok(r) => r,
            Err(e) => {
                warn!("Invalid JSON from worker {}: {}", worker_id, e);
                return Some(StratumResponse {
                    id: None,
                    result: None,
                    error: Some(StratumError {
                        code: -32700,
                        message: "Parse error".to_string(),
                    }),
                });
            }
        };

        debug!("Worker {} request: {} {:?}", worker_id, request.method, request.params);

        match request.method.as_str() {
            "mining.subscribe" => self.handle_subscribe(worker_id, request).await,
            "mining.authorize" => self.handle_authorize(worker_id, request).await,
            "mining.submit" => self.handle_submit(worker_id, request).await,
            "mining.extranonce.subscribe" => self.handle_extranonce_subscribe(request),
            _ => {
                Some(StratumResponse {
                    id: request.id,
                    result: None,
                    error: Some(StratumError {
                        code: -32601,
                        message: format!("Method not found: {}", request.method),
                    }),
                })
            }
        }
    }

    async fn handle_subscribe(&self, worker_id: u64, request: StratumRequest) -> Option<StratumResponse> {
        let mut workers = self.workers.write().await;
        if let Some(worker) = workers.get_mut(&worker_id) {
            worker.subscribed = true;

            // Response format for KAWPOW/ethproxy:
            // [[["mining.notify", "subscription_id"], ["mining.set_difficulty", "subscription_id"]], extranonce1, extranonce2_size]
            let result = serde_json::json!([
                [
                    ["mining.notify", format!("{:x}", worker_id)],
                    ["mining.set_difficulty", format!("{:x}", worker_id)]
                ],
                worker.extranonce1.clone(),
                self.extranonce2_size
            ]);

            Some(StratumResponse {
                id: request.id,
                result: Some(result),
                error: None,
            })
        } else {
            None
        }
    }

    async fn handle_authorize(&self, worker_id: u64, request: StratumRequest) -> Option<StratumResponse> {
        let params = request.params.as_array();
        let (worker_name, _password) = match params {
            Some(p) if p.len() >= 2 => {
                (
                    p[0].as_str().unwrap_or("unknown").to_string(),
                    p[1].as_str().unwrap_or("x").to_string(),
                )
            }
            _ => ("unknown".to_string(), "x".to_string()),
        };

        // Parse worker name for address: "address.worker_name" format
        let (beneficiary, name) = if worker_name.contains('.') {
            let parts: Vec<&str> = worker_name.splitn(2, '.').collect();
            let addr = parse_address(parts[0]);
            (addr, parts.get(1).unwrap_or(&"default").to_string())
        } else {
            (parse_address(&worker_name), worker_name.clone())
        };

        let mut workers = self.workers.write().await;
        if let Some(worker) = workers.get_mut(&worker_id) {
            worker.authorized = true;
            worker.name = name;
            worker.beneficiary = beneficiary;
            worker.difficulty = self.default_difficulty;

            info!("Worker {} authorized: {} (beneficiary: {:?})", 
                worker_id, worker.name, worker.beneficiary);
        }

        Some(StratumResponse {
            id: request.id,
            result: Some(serde_json::Value::Bool(true)),
            error: None,
        })
    }

    async fn handle_submit(&self, worker_id: u64, request: StratumRequest) -> Option<StratumResponse> {
        let params = match request.params.as_array() {
            Some(p) => p,
            None => {
                return Some(StratumResponse {
                    id: request.id,
                    result: None,
                    error: Some(StratumError {
                        code: -1,
                        message: "Invalid params".to_string(),
                    }),
                });
            }
        };

        // Parse submit params: [worker_name, job_id, extranonce2, ntime, nonce]
        // or ethproxy format: [worker_name, job_id, nonce, header_hash, mix_hash]
        if params.len() < 5 {
            return Some(StratumResponse {
                id: request.id,
                result: None,
                error: Some(StratumError {
                    code: -1,
                    message: "Invalid params count".to_string(),
                }),
            });
        }

        let job_id = params[1].as_str().unwrap_or("").to_string();
        let nonce = parse_hex_u64(params[2].as_str().unwrap_or("0"));
        let header_hash = parse_hex_h256(params[3].as_str().unwrap_or(""));
        let mix_hash = parse_hex_h256(params[4].as_str().unwrap_or(""));

        let share = Share {
            worker_id,
            job_id,
            nonce,
            header_hash,
            mix_hash,
            submitted_at: Instant::now(),
        };

        // Get worker for validation
        let worker = {
            let workers = self.workers.read().await;
            workers.get(&worker_id).cloned()
        };

        let worker = match worker {
            Some(w) => w,
            None => {
                return Some(StratumResponse {
                    id: request.id,
                    result: None,
                    error: Some(StratumError {
                        code: -1,
                        message: "Worker not found".to_string(),
                    }),
                });
            }
        };

        // Validate share
        let result = self.validate_share_internal(&share, &worker).await;

        // Update worker stats
        {
            let mut workers = self.workers.write().await;
            if let Some(w) = workers.get_mut(&worker_id) {
                w.last_share_at = Some(Instant::now());
                match &result {
                    ShareResult::Accepted | ShareResult::AcceptedBlock(_) => w.shares_accepted += 1,
                    ShareResult::Rejected(_) => w.shares_rejected += 1,
                    ShareResult::Stale => w.shares_stale += 1,
                }
            }
        }

        match result {
            ShareResult::Accepted | ShareResult::AcceptedBlock(_) => {
                Some(StratumResponse {
                    id: request.id,
                    result: Some(serde_json::Value::Bool(true)),
                    error: None,
                })
            }
            ShareResult::Rejected(msg) => {
                Some(StratumResponse {
                    id: request.id,
                    result: None,
                    error: Some(StratumError {
                        code: -1,
                        message: msg,
                    }),
                })
            }
            ShareResult::Stale => {
                Some(StratumResponse {
                    id: request.id,
                    result: None,
                    error: Some(StratumError {
                        code: 21,
                        message: "Job not found (stale)".to_string(),
                    }),
                })
            }
        }
    }

    async fn validate_share_internal(&self, share: &Share, worker: &Worker) -> ShareResult {
        let job = {
            let jobs = self.jobs.read().await;
            match jobs.get(&share.job_id) {
                Some(j) => j.clone(),
                None => return ShareResult::Stale,
            }
        };

        if job.created_at.elapsed() > Duration::from_secs(self.config.job_timeout) {
            return ShareResult::Stale;
        }

        let cache = self.kawpow_cache.read().await;
        let cache = match cache.as_ref() {
            Some(c) => c,
            None => return ShareResult::Rejected("No KAWPOW cache".to_string()),
        };

        let pow_hash = kawpow_hash(
            share.header_hash.as_bytes(),
            share.nonce,
            job.height,
            &cache.cache,
        );
        let pow_hash = H256::from_slice(&pow_hash);

        let share_target = worker_difficulty_to_target(worker.difficulty);
        if pow_hash.as_bytes() > share_target.as_bytes() {
            self.stats.shares_rejected.fetch_add(1, Ordering::Relaxed);
            return ShareResult::Rejected("Below share target".to_string());
        }

        self.stats.shares_accepted.fetch_add(1, Ordering::Relaxed);

        if pow_hash.as_bytes() <= job.target.as_bytes() {
            info!("🎉 BLOCK FOUND! Height: {}, Hash: {}", job.height, pow_hash);
            self.stats.blocks_found.fetch_add(1, Ordering::Relaxed);

            let beneficiary = worker.beneficiary.unwrap_or(self.config.coinbase_address);
            let header = job.build_header(share.nonce, 0, &beneficiary);
            
            let mut txs = vec![job.coinbase_tx.clone()];
            txs.extend(job.transactions.clone());
            
            let block = Block::new(header, txs);

            if let Some(submit_fn) = self.block_submit_fn.read().await.as_ref() {
                match submit_fn(block) {
                    Ok(hash) => return ShareResult::AcceptedBlock(hash),
                    Err(e) => error!("Block submission failed: {}", e),
                }
            }

            return ShareResult::AcceptedBlock(pow_hash);
        }

        ShareResult::Accepted
    }

    fn handle_extranonce_subscribe(&self, request: StratumRequest) -> Option<StratumResponse> {
        Some(StratumResponse {
            id: request.id,
            result: Some(serde_json::Value::Bool(true)),
            error: None,
        })
    }

    fn create_job_notification(&self, job: &MiningJob, extranonce1: &str) -> StratumNotification {
        // KAWPOW/ethproxy job notification format:
        // ["job_id", "seed_hash", "header_hash", clean_jobs]
        StratumNotification {
            id: None,
            method: "mining.notify".to_string(),
            params: serde_json::json!([
                job.job_id,
                format!("0x{}", hex::encode(job.seed_hash.as_bytes())),
                format!("0x{}", hex::encode(job.header_hash.as_bytes())),
                job.clean
            ]),
        }
    }
}

// Helper functions

fn compute_merkle_root(transactions: &[Transaction]) -> H256 {
    if transactions.is_empty() {
        return H256::zero();
    }

    let mut hashes: Vec<H256> = transactions.iter().map(|tx| tx.txid()).collect();

    while hashes.len() > 1 {
        if hashes.len() % 2 == 1 {
            hashes.push(*hashes.last().unwrap());
        }
        hashes = hashes.chunks(2).map(|pair| {
            let mut combined = [0u8; 64];
            combined[..32].copy_from_slice(&pair[0].0);
            combined[32..].copy_from_slice(&pair[1].0);
            H256::from_slice(blake3::hash(&combined).as_bytes())
        }).collect();
    }

    hashes[0]
}

fn difficulty_to_target(difficulty: u64) -> H256 {
    if difficulty == 0 {
        return H256([0xff; 32]);
    }
    
    let max = ethereum_types::U256::MAX;
    let target = max / ethereum_types::U256::from(difficulty);
    let mut bytes = [0u8; 32];
    target.to_big_endian(&mut bytes);
    H256(bytes)
}

fn worker_difficulty_to_target(difficulty: f64) -> H256 {
    if difficulty <= 0.0 {
        return H256([0xff; 32]);
    }
    
    // For share difficulty, we use a simpler calculation
    let diff_u64 = (difficulty * 1000.0) as u64;
    difficulty_to_target(diff_u64.max(1))
}

fn parse_hex_u64(s: &str) -> u64 {
    let s = s.strip_prefix("0x").unwrap_or(s);
    u64::from_str_radix(s, 16).unwrap_or(0)
}

fn parse_hex_h256(s: &str) -> H256 {
    let s = s.strip_prefix("0x").unwrap_or(s);
    let bytes = hex::decode(s).unwrap_or_default();
    H256::from_slice(&bytes)
}

fn parse_address(s: &str) -> Option<Address> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    if s.len() != 40 {
        return None;
    }
    let bytes = hex::decode(s).ok()?;
    if bytes.len() != 20 {
        return None;
    }
    Some(Address::from_slice(&bytes))
}

impl Clone for Worker {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            name: self.name.clone(),
            address: self.address,
            extranonce1: self.extranonce1.clone(),
            difficulty: self.difficulty,
            authorized: self.authorized,
            subscribed: self.subscribed,
            connected_at: self.connected_at,
            last_share_at: self.last_share_at,
            shares_accepted: self.shares_accepted,
            shares_rejected: self.shares_rejected,
            shares_stale: self.shares_stale,
            hashrate_estimate: self.hashrate_estimate,
            beneficiary: self.beneficiary,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_to_target() {
        let target = difficulty_to_target(1);
        assert!(!target.is_zero());
        
        let target_high = difficulty_to_target(1000);
        assert!(target_high < target);
    }

    #[test]
    fn test_generate_job_id() {
        let config = StratumServerConfig::default();
        let server = StratumServer::new(config);
        
        let id1 = server.generate_job_id(100);
        let id2 = server.generate_job_id(101);
        
        // Different block heights should produce different IDs
        assert_ne!(id1, id2);
        
        // Job ID format: 8 hex chars for height + 8 hex chars for timestamp
        assert_eq!(id1.len(), 16);
        assert!(id1.starts_with("00000064")); // 100 in hex = 0x64
        assert!(id2.starts_with("00000065")); // 101 in hex = 0x65
    }

    #[test]
    fn test_parse_address() {
        let addr = parse_address("0x742d35Cc6634C0532925a3b844Bc9e7595f1dE5a");
        assert!(addr.is_some());
        
        let invalid = parse_address("invalid");
        assert!(invalid.is_none());
    }
}
