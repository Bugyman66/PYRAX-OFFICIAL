//! BLAKE3 Stratum Mining Server for Stream A
//!
//! Production-ready stratum server for BLAKE3 ASIC mining on Stream A.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{RwLock, broadcast};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, debug, error};

use crate::types::{H256, Address, BlockNumber, UtxoTransaction};
use crate::types::stream_a_block::{StreamABlock, StreamAHeader, StreamABody};
use crate::consensus::Blake3Pow;

/// BLAKE3 Stratum server configuration
#[derive(Debug, Clone)]
pub struct Blake3StratumConfig {
    pub bind_addr: SocketAddr,
    pub default_difficulty: f64,
    pub min_difficulty: f64,
    pub max_difficulty: f64,
    pub target_share_time: u64,
    pub job_timeout: u64,
    pub max_connections: usize,
    pub ban_duration: u64,
    pub extranonce1_size: usize,
    pub extranonce2_size: usize,
    pub coinbase_address: Address,
    pub network_difficulty: u64,
    pub block_reward: u64,
}

impl Default for Blake3StratumConfig {
    fn default() -> Self {
        Self {
            bind_addr: "0.0.0.0:3334".parse().unwrap(),
            default_difficulty: 1.0,
            min_difficulty: 0.0001,
            max_difficulty: 1000000000.0,
            target_share_time: 10,
            job_timeout: 300,
            max_connections: 50000,
            ban_duration: 600,
            extranonce1_size: 4,
            extranonce2_size: 4,
            coinbase_address: Address::ZERO,
            network_difficulty: 1,
            block_reward: 50_00000000,
        }
    }
}

/// BLAKE3 mining job
#[derive(Debug, Clone)]
pub struct Blake3MiningJob {
    pub job_id: String,
    pub height: BlockNumber,
    pub parent_hash: H256,
    pub merkle_root: H256,
    pub utxo_root: H256,
    pub timestamp: u64,
    pub difficulty: u64,
    pub target: H256,
    pub coinbase_tx: UtxoTransaction,
    pub transactions: Vec<UtxoTransaction>,
    pub created_at: Instant,
    pub clean: bool,
    pub header_bytes: Vec<u8>,
}

impl Blake3MiningJob {
    pub fn build_header(&self, nonce: u64, extra_nonce: u64, beneficiary: &Address) -> StreamAHeader {
        StreamAHeader {
            version: 1,
            stream: 0,
            parent_hash: self.parent_hash,
            merkle_root: self.merkle_root,
            utxo_root: self.utxo_root,
            timestamp: self.timestamp,
            difficulty: self.difficulty,
            nonce,
            extra_nonce,
            height: self.height,
            beneficiary: *beneficiary,
        }
    }
    
    pub fn build_block(&self, nonce: u64, extra_nonce: u64, beneficiary: &Address) -> StreamABlock {
        let header = self.build_header(nonce, extra_nonce, beneficiary);
        let mut txs = vec![self.coinbase_tx.clone()];
        txs.extend(self.transactions.clone());
        StreamABlock::new(header, StreamABody::new(txs))
    }
}

/// Connected worker state
#[derive(Debug, Clone)]
pub struct Blake3Worker {
    pub id: u64,
    pub name: String,
    pub address: SocketAddr,
    pub extranonce1: String,
    pub extranonce1_bytes: [u8; 4],
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
    pub share_times: Vec<Instant>,
}

impl Blake3Worker {
    pub fn new(id: u64, address: SocketAddr, extranonce1: String) -> Self {
        let extranonce1_bytes = hex::decode(&extranonce1)
            .map(|b| { let mut arr = [0u8; 4]; arr.copy_from_slice(&b[..4.min(b.len())]); arr })
            .unwrap_or([0u8; 4]);
        Self {
            id, name: String::new(), address, extranonce1, extranonce1_bytes,
            difficulty: 1.0, authorized: false, subscribed: false,
            connected_at: Instant::now(), last_share_at: None,
            shares_accepted: 0, shares_rejected: 0, shares_stale: 0,
            hashrate_estimate: 0.0, beneficiary: None, share_times: Vec::new(),
        }
    }
    
    pub fn update_hashrate(&mut self, difficulty: f64) {
        let now = Instant::now();
        self.share_times.retain(|t| now.duration_since(*t).as_secs() < 300);
        self.share_times.push(now);
        if self.share_times.len() >= 2 {
            let duration = now.duration_since(*self.share_times.first().unwrap()).as_secs_f64();
            if duration > 0.0 {
                self.hashrate_estimate = (self.share_times.len() as f64 * difficulty * 4294967296.0) / duration;
            }
        }
    }
}

/// Share submission
#[derive(Debug, Clone)]
pub struct Blake3Share {
    pub worker_id: u64,
    pub job_id: String,
    pub extranonce2: String,
    pub nonce: u64,
    pub ntime: u64,
    pub submitted_at: Instant,
}

/// Share validation result
#[derive(Debug, Clone)]
pub enum Blake3ShareResult {
    Accepted,
    AcceptedBlock(H256),
    Rejected(String),
    Stale,
}

/// Block template
#[derive(Debug, Clone)]
pub struct Blake3BlockTemplate {
    pub height: BlockNumber,
    pub parent_hash: H256,
    pub utxo_root: H256,
    pub timestamp: u64,
    pub difficulty: u64,
    pub transactions: Vec<UtxoTransaction>,
    pub coinbase_value: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct StratumRequest {
    id: Option<serde_json::Value>,
    method: String,
    params: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct StratumResponse {
    id: Option<serde_json::Value>,
    result: Option<serde_json::Value>,
    error: Option<StratumError>,
}

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
pub struct Blake3ServerStats {
    pub workers_connected: AtomicU64,
    pub workers_total: AtomicU64,
    pub shares_accepted: AtomicU64,
    pub shares_rejected: AtomicU64,
    pub shares_stale: AtomicU64,
    pub blocks_found: AtomicU64,
    pub total_hashrate: AtomicU64,
    pub last_block_time: AtomicU64,
    pub last_block_height: AtomicU64,
}

pub type Blake3BlockSubmitFn = Box<dyn Fn(StreamABlock) -> Result<H256, String> + Send + Sync>;
pub type Blake3TemplateProviderFn = Box<dyn Fn() -> Option<Blake3BlockTemplate> + Send + Sync>;

/// Production BLAKE3 Stratum Mining Server for Stream A
pub struct Blake3StratumServer {
    config: Blake3StratumConfig,
    workers: Arc<RwLock<HashMap<u64, Blake3Worker>>>,
    jobs: Arc<RwLock<HashMap<String, Blake3MiningJob>>>,
    current_job: Arc<RwLock<Option<Blake3MiningJob>>>,
    stats: Arc<Blake3ServerStats>,
    next_worker_id: AtomicU64,
    next_extranonce1: AtomicU64,
    running: Arc<AtomicBool>,
    job_broadcast: broadcast::Sender<Blake3MiningJob>,
    block_submit_fn: Arc<RwLock<Option<Blake3BlockSubmitFn>>>,
    template_provider: Arc<RwLock<Option<Blake3TemplateProviderFn>>>,
}

impl Blake3StratumServer {
    pub fn new(config: Blake3StratumConfig) -> Self {
        let (job_broadcast, _) = broadcast::channel(32);
        Self {
            config,
            workers: Arc::new(RwLock::new(HashMap::new())),
            jobs: Arc::new(RwLock::new(HashMap::new())),
            current_job: Arc::new(RwLock::new(None)),
            stats: Arc::new(Blake3ServerStats::default()),
            next_worker_id: AtomicU64::new(1),
            next_extranonce1: AtomicU64::new(rand::random::<u64>() & 0xFFFFFFFF),
            running: Arc::new(AtomicBool::new(false)),
            job_broadcast,
            block_submit_fn: Arc::new(RwLock::new(None)),
            template_provider: Arc::new(RwLock::new(None)),
        }
    }
    
    pub async fn set_block_submit_fn(&self, f: Blake3BlockSubmitFn) {
        *self.block_submit_fn.write().await = Some(f);
    }
    
    pub async fn set_template_provider(&self, f: Blake3TemplateProviderFn) {
        *self.template_provider.write().await = Some(f);
    }
    
    pub fn stats(&self) -> &Blake3ServerStats { &self.stats }
    pub async fn current_job(&self) -> Option<Blake3MiningJob> { self.current_job.read().await.clone() }
    pub async fn workers(&self) -> Vec<Blake3Worker> { self.workers.read().await.values().cloned().collect() }
    pub fn is_running(&self) -> bool { self.running.load(Ordering::SeqCst) }
    pub fn stop(&self) { self.running.store(false, Ordering::SeqCst); }
    
    fn generate_extranonce1(&self) -> String {
        let val = self.next_extranonce1.fetch_add(1, Ordering::SeqCst);
        format!("{:08x}", val & 0xFFFFFFFF)
    }
    
    fn generate_job_id(&self, height: BlockNumber) -> String {
        let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
        format!("{:08x}{:08x}", height, ts & 0xFFFFFFFF)
    }
    
    pub async fn create_job(&self, template: Blake3BlockTemplate, clean: bool) -> Blake3MiningJob {
        let job_id = self.generate_job_id(template.height);
        let coinbase_tx = UtxoTransaction::coinbase(template.height, template.coinbase_value, &self.config.coinbase_address);
        let mut all_txs = vec![coinbase_tx.clone()];
        all_txs.extend(template.transactions.clone());
        let merkle_root = compute_utxo_merkle_root(&all_txs);
        let target = Blake3Pow::difficulty_to_target(template.difficulty);
        let header_bytes = encode_header_for_mining(
            template.parent_hash, merkle_root, template.utxo_root,
            template.timestamp, template.difficulty, template.height, self.config.coinbase_address,
        );
        
        let job = Blake3MiningJob {
            job_id: job_id.clone(), height: template.height, parent_hash: template.parent_hash,
            merkle_root, utxo_root: template.utxo_root, timestamp: template.timestamp,
            difficulty: template.difficulty, target, coinbase_tx, transactions: template.transactions,
            created_at: Instant::now(), clean, header_bytes,
        };
        
        {
            let mut jobs = self.jobs.write().await;
            jobs.insert(job_id, job.clone());
            if jobs.len() > 20 {
                let mut to_remove: Vec<_> = jobs.iter().map(|(k, v)| (k.clone(), v.created_at)).collect();
                to_remove.sort_by_key(|(_, t)| *t);
                for (key, _) in to_remove.iter().take(jobs.len() - 20) { jobs.remove(key); }
            }
        }
        *self.current_job.write().await = Some(job.clone());
        job
    }
    
    pub fn broadcast_job(&self, job: Blake3MiningJob) { let _ = self.job_broadcast.send(job); }
    
    pub async fn validate_share(&self, share: &Blake3Share, worker: &Blake3Worker) -> Blake3ShareResult {
        let job = {
            let jobs = self.jobs.read().await;
            match jobs.get(&share.job_id) { Some(j) => j.clone(), None => return Blake3ShareResult::Stale }
        };
        if job.created_at.elapsed() > Duration::from_secs(self.config.job_timeout) {
            return Blake3ShareResult::Stale;
        }
        let extranonce2_bytes = hex::decode(&share.extranonce2).unwrap_or_default();
        let extra_nonce = combine_extranonces(&worker.extranonce1_bytes, &extranonce2_bytes);
        let pow_hash = Blake3Pow::hash(&job.header_bytes, share.nonce, extra_nonce);
        let share_target = difficulty_to_target_f64(worker.difficulty);
        
        if !hash_meets_target(&pow_hash, &share_target) {
            self.stats.shares_rejected.fetch_add(1, Ordering::Relaxed);
            return Blake3ShareResult::Rejected("Below share target".to_string());
        }
        self.stats.shares_accepted.fetch_add(1, Ordering::Relaxed);
        
        if hash_meets_target(&pow_hash, &job.target) {
            info!("🎉 BLAKE3 BLOCK FOUND at height {}! Hash: 0x{}", job.height, hex::encode(pow_hash.as_bytes()));
            self.stats.blocks_found.fetch_add(1, Ordering::Relaxed);
            self.stats.last_block_height.store(job.height, Ordering::Relaxed);
            self.stats.last_block_time.store(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(), Ordering::Relaxed);
            
            let beneficiary = worker.beneficiary.unwrap_or(self.config.coinbase_address);
            let block = job.build_block(share.nonce, extra_nonce, &beneficiary);
            if !block.verify_pow() {
                error!("Block PoW verification failed!");
                return Blake3ShareResult::Rejected("PoW verification failed".to_string());
            }
            if let Some(submit_fn) = self.block_submit_fn.read().await.as_ref() {
                match submit_fn(block) {
                    Ok(hash) => { info!("✅ Block submitted: 0x{}", hex::encode(hash.as_bytes())); return Blake3ShareResult::AcceptedBlock(hash); }
                    Err(e) => { error!("❌ Block submission failed: {}", e); }
                }
            }
            return Blake3ShareResult::AcceptedBlock(pow_hash);
        }
        Blake3ShareResult::Accepted
    }
    
    pub async fn run(&self) -> anyhow::Result<()> {
        let listener = TcpListener::bind(&self.config.bind_addr).await?;
        self.running.store(true, Ordering::SeqCst);
        
        info!("╔══════════════════════════════════════════════════════════════════╗");
        info!("║       PYRAX Stream A - BLAKE3 ASIC Stratum Server                ║");
        info!("╠══════════════════════════════════════════════════════════════════╣");
        info!("║  Algorithm: BLAKE3 PoW | Stream: A | Block Time: 10s             ║");
        info!("║  Bind: {:?} | Diff: {} | Max: {}", self.config.bind_addr, self.config.default_difficulty, self.config.max_connections);
        info!("╚══════════════════════════════════════════════════════════════════╝");
        
        let server = self.clone_internals();
        tokio::spawn(async move { Blake3StratumInternals::job_update_loop(server).await; });
        
        let stats = Arc::clone(&self.stats);
        let workers = Arc::clone(&self.workers);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(30)).await;
                let total: f64 = workers.read().await.values().map(|w| w.hashrate_estimate).sum();
                stats.total_hashrate.store(total as u64, Ordering::Relaxed);
            }
        });
        
        while self.running.load(Ordering::SeqCst) {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    if self.stats.workers_connected.load(Ordering::Relaxed) >= self.config.max_connections as u64 {
                        warn!("Max connections reached, rejecting {}", addr);
                        continue;
                    }
                    let server = self.clone_internals();
                    tokio::spawn(async move {
                        if let Err(e) = Blake3StratumInternals::handle_connection(server, stream, addr).await {
                            debug!("Connection {} closed: {}", addr, e);
                        }
                    });
                }
                Err(e) => error!("Accept error: {}", e),
            }
        }
        Ok(())
    }
    
    fn clone_internals(&self) -> Blake3StratumInternals {
        Blake3StratumInternals {
            config: self.config.clone(),
            workers: Arc::clone(&self.workers),
            jobs: Arc::clone(&self.jobs),
            current_job: Arc::clone(&self.current_job),
            stats: Arc::clone(&self.stats),
            next_worker_id: &self.next_worker_id as *const AtomicU64,
            next_extranonce1: &self.next_extranonce1 as *const AtomicU64,
            running: Arc::clone(&self.running),
            job_broadcast: self.job_broadcast.clone(),
            block_submit_fn: Arc::clone(&self.block_submit_fn),
            template_provider: Arc::clone(&self.template_provider),
        }
    }
}

struct Blake3StratumInternals {
    config: Blake3StratumConfig,
    workers: Arc<RwLock<HashMap<u64, Blake3Worker>>>,
    jobs: Arc<RwLock<HashMap<String, Blake3MiningJob>>>,
    current_job: Arc<RwLock<Option<Blake3MiningJob>>>,
    stats: Arc<Blake3ServerStats>,
    next_worker_id: *const AtomicU64,
    next_extranonce1: *const AtomicU64,
    running: Arc<AtomicBool>,
    job_broadcast: broadcast::Sender<Blake3MiningJob>,
    block_submit_fn: Arc<RwLock<Option<Blake3BlockSubmitFn>>>,
    template_provider: Arc<RwLock<Option<Blake3TemplateProviderFn>>>,
}

unsafe impl Send for Blake3StratumInternals {}
unsafe impl Sync for Blake3StratumInternals {}

impl Blake3StratumInternals {
    async fn job_update_loop(server: Blake3StratumInternals) {
        let mut last_height: BlockNumber = 0;
        let mut last_parent = H256::zero();
        loop {
            if !server.running.load(Ordering::SeqCst) { break; }
            if let Some(provider) = server.template_provider.read().await.as_ref() {
                if let Some(template) = provider() {
                    let clean = template.height != last_height || template.parent_hash != last_parent;
                    last_height = template.height;
                    last_parent = template.parent_hash;
                    let job = server.create_job_internal(template, clean).await;
                    if clean { info!("New BLAKE3 block template at height {}", job.height); }
                    let _ = server.job_broadcast.send(job);
                }
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }
    
    async fn create_job_internal(&self, template: Blake3BlockTemplate, clean: bool) -> Blake3MiningJob {
        let job_id = format!("{:08x}{:08x}", template.height, 
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64 & 0xFFFFFFFF);
        let coinbase_tx = UtxoTransaction::coinbase(template.height, template.coinbase_value, &self.config.coinbase_address);
        let mut all_txs = vec![coinbase_tx.clone()];
        all_txs.extend(template.transactions.clone());
        let merkle_root = compute_utxo_merkle_root(&all_txs);
        let target = Blake3Pow::difficulty_to_target(template.difficulty);
        let header_bytes = encode_header_for_mining(
            template.parent_hash, merkle_root, template.utxo_root,
            template.timestamp, template.difficulty, template.height, self.config.coinbase_address,
        );
        let job = Blake3MiningJob {
            job_id: job_id.clone(), height: template.height, parent_hash: template.parent_hash,
            merkle_root, utxo_root: template.utxo_root, timestamp: template.timestamp,
            difficulty: template.difficulty, target, coinbase_tx, transactions: template.transactions,
            created_at: Instant::now(), clean, header_bytes,
        };
        { let mut jobs = self.jobs.write().await; jobs.insert(job_id, job.clone()); }
        *self.current_job.write().await = Some(job.clone());
        job
    }
    
    async fn handle_connection(server: Blake3StratumInternals, stream: TcpStream, addr: SocketAddr) -> anyhow::Result<()> {
        let worker_id = unsafe { (*server.next_worker_id).fetch_add(1, Ordering::SeqCst) };
        let extranonce1 = unsafe { format!("{:08x}", (*server.next_extranonce1).fetch_add(1, Ordering::SeqCst) & 0xFFFFFFFF) };
        server.workers.write().await.insert(worker_id, Blake3Worker::new(worker_id, addr, extranonce1.clone()));
        server.stats.workers_connected.fetch_add(1, Ordering::Relaxed);
        server.stats.workers_total.fetch_add(1, Ordering::Relaxed);
        info!("BLAKE3 Worker {} connected from {}", worker_id, addr);
        
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut job_rx = server.job_broadcast.subscribe();
        
        let result = async {
            let mut line = String::new();
            loop {
                tokio::select! {
                    result = reader.read_line(&mut line) => {
                        match result {
                            Ok(0) => break,
                            Ok(_) => {
                                if let Some(response) = server.handle_message(worker_id, &line, &extranonce1).await {
                                    let mut json = serde_json::to_string(&response)?;
                                    json.push('\n');
                                    writer.write_all(json.as_bytes()).await?;
                                }
                                line.clear();
                            }
                            Err(e) => return Err(e.into()),
                        }
                    }
                    Ok(job) = job_rx.recv() => {
                        let workers = server.workers.read().await;
                        if let Some(worker) = workers.get(&worker_id) {
                            if worker.authorized {
                                let notif = server.create_job_notification(&job);
                                let mut json = serde_json::to_string(&notif)?;
                                json.push('\n');
                                writer.write_all(json.as_bytes()).await?;
                            }
                        }
                    }
                }
            }
            Ok(())
        }.await;
        
        server.workers.write().await.remove(&worker_id);
        server.stats.workers_connected.fetch_sub(1, Ordering::Relaxed);
        info!("BLAKE3 Worker {} disconnected", worker_id);
        result
    }
    
    async fn handle_message(&self, worker_id: u64, line: &str, extranonce1: &str) -> Option<StratumResponse> {
        let line = line.trim();
        if line.is_empty() { return None; }
        let request: StratumRequest = match serde_json::from_str(line) {
            Ok(r) => r,
            Err(e) => {
                warn!("Invalid JSON from worker {}: {}", worker_id, e);
                return Some(StratumResponse { id: None, result: None, error: Some(StratumError { code: -32700, message: "Parse error".to_string() }) });
            }
        };
        debug!("Worker {} request: {} {:?}", worker_id, request.method, request.params);
        match request.method.as_str() {
            "mining.subscribe" => self.handle_subscribe(worker_id, request, extranonce1).await,
            "mining.authorize" => self.handle_authorize(worker_id, request).await,
            "mining.submit" => self.handle_submit(worker_id, request).await,
            "mining.extranonce.subscribe" => Some(StratumResponse { id: request.id, result: Some(serde_json::Value::Bool(true)), error: None }),
            _ => Some(StratumResponse { id: request.id, result: None, error: Some(StratumError { code: -32601, message: format!("Method not found: {}", request.method) }) }),
        }
    }
    
    async fn handle_subscribe(&self, worker_id: u64, request: StratumRequest, extranonce1: &str) -> Option<StratumResponse> {
        let mut workers = self.workers.write().await;
        if let Some(worker) = workers.get_mut(&worker_id) {
            worker.subscribed = true;
            let result = serde_json::json!([[["mining.notify", format!("{:x}", worker_id)], ["mining.set_difficulty", format!("{:x}", worker_id)]], extranonce1, self.config.extranonce2_size]);
            info!("Worker {} subscribed, extranonce1: {}", worker_id, extranonce1);
            Some(StratumResponse { id: request.id, result: Some(result), error: None })
        } else { None }
    }
    
    async fn handle_authorize(&self, worker_id: u64, request: StratumRequest) -> Option<StratumResponse> {
        let params = request.params.as_array();
        let worker_name = params.and_then(|p| p.first()).and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
        let (beneficiary, name) = if worker_name.contains('.') {
            let parts: Vec<&str> = worker_name.splitn(2, '.').collect();
            (parse_address(parts[0]), parts.get(1).unwrap_or(&"default").to_string())
        } else { (parse_address(&worker_name), worker_name.clone()) };
        
        let mut workers = self.workers.write().await;
        if let Some(worker) = workers.get_mut(&worker_id) {
            worker.authorized = true;
            worker.name = name.clone();
            worker.beneficiary = beneficiary;
            worker.difficulty = self.config.default_difficulty;
            info!("Worker {} authorized: {} (beneficiary: {:?})", worker_id, name, beneficiary.map(|a| format!("0x{}", hex::encode(a.as_bytes()))));
        }
        Some(StratumResponse { id: request.id, result: Some(serde_json::Value::Bool(true)), error: None })
    }
    
    async fn handle_submit(&self, worker_id: u64, request: StratumRequest) -> Option<StratumResponse> {
        let params = match request.params.as_array() {
            Some(p) if p.len() >= 5 => p,
            _ => return Some(StratumResponse { id: request.id, result: None, error: Some(StratumError { code: -1, message: "Invalid params".to_string() }) }),
        };
        let share = Blake3Share {
            worker_id,
            job_id: params[1].as_str().unwrap_or("").to_string(),
            extranonce2: params[2].as_str().unwrap_or("").to_string(),
            ntime: parse_hex_u64(params[3].as_str().unwrap_or("0")),
            nonce: parse_hex_u64(params[4].as_str().unwrap_or("0")),
            submitted_at: Instant::now(),
        };
        let worker = { self.workers.read().await.get(&worker_id).cloned() };
        let worker = match worker {
            Some(w) => w,
            None => return Some(StratumResponse { id: request.id, result: None, error: Some(StratumError { code: -1, message: "Worker not found".to_string() }) }),
        };
        let result = self.validate_share_internal(&share, &worker).await;
        {
            let mut workers = self.workers.write().await;
            if let Some(w) = workers.get_mut(&worker_id) {
                w.last_share_at = Some(Instant::now());
                match &result {
                    Blake3ShareResult::Accepted | Blake3ShareResult::AcceptedBlock(_) => { w.shares_accepted += 1; w.update_hashrate(w.difficulty); }
                    Blake3ShareResult::Rejected(_) => w.shares_rejected += 1,
                    Blake3ShareResult::Stale => w.shares_stale += 1,
                }
            }
        }
        match result {
            Blake3ShareResult::Accepted | Blake3ShareResult::AcceptedBlock(_) => Some(StratumResponse { id: request.id, result: Some(serde_json::Value::Bool(true)), error: None }),
            Blake3ShareResult::Rejected(msg) => Some(StratumResponse { id: request.id, result: None, error: Some(StratumError { code: -1, message: msg }) }),
            Blake3ShareResult::Stale => Some(StratumResponse { id: request.id, result: None, error: Some(StratumError { code: 21, message: "Job not found (stale)".to_string() }) }),
        }
    }
    
    async fn validate_share_internal(&self, share: &Blake3Share, worker: &Blake3Worker) -> Blake3ShareResult {
        let job = { let jobs = self.jobs.read().await; match jobs.get(&share.job_id) { Some(j) => j.clone(), None => return Blake3ShareResult::Stale } };
        if job.created_at.elapsed() > Duration::from_secs(self.config.job_timeout) { return Blake3ShareResult::Stale; }
        let extranonce2_bytes = hex::decode(&share.extranonce2).unwrap_or_default();
        let extra_nonce = combine_extranonces(&worker.extranonce1_bytes, &extranonce2_bytes);
        let pow_hash = Blake3Pow::hash(&job.header_bytes, share.nonce, extra_nonce);
        let share_target = difficulty_to_target_f64(worker.difficulty);
        if !hash_meets_target(&pow_hash, &share_target) {
            self.stats.shares_rejected.fetch_add(1, Ordering::Relaxed);
            return Blake3ShareResult::Rejected("Below share target".to_string());
        }
        self.stats.shares_accepted.fetch_add(1, Ordering::Relaxed);
        if hash_meets_target(&pow_hash, &job.target) {
            info!("🎉 BLAKE3 BLOCK FOUND! Height: {}, Hash: 0x{}", job.height, hex::encode(pow_hash.as_bytes()));
            self.stats.blocks_found.fetch_add(1, Ordering::Relaxed);
            let beneficiary = worker.beneficiary.unwrap_or(self.config.coinbase_address);
            let block = job.build_block(share.nonce, extra_nonce, &beneficiary);
            if !block.verify_pow() { return Blake3ShareResult::Rejected("PoW verification failed".to_string()); }
            if let Some(submit_fn) = self.block_submit_fn.read().await.as_ref() {
                match submit_fn(block) {
                    Ok(hash) => { info!("✅ Block submitted: 0x{}", hex::encode(hash.as_bytes())); return Blake3ShareResult::AcceptedBlock(hash); }
                    Err(e) => error!("❌ Block submission failed: {}", e),
                }
            }
            return Blake3ShareResult::AcceptedBlock(pow_hash);
        }
        Blake3ShareResult::Accepted
    }
    
    fn create_job_notification(&self, job: &Blake3MiningJob) -> StratumNotification {
        let header_hash = hex::encode(blake3::hash(&job.header_bytes).as_bytes());
        StratumNotification {
            id: None,
            method: "mining.notify".to_string(),
            params: serde_json::json!([job.job_id, format!("0x{}", hex::encode(job.parent_hash.as_bytes())), format!("0x{}", header_hash), format!("0x{}", hex::encode(job.target.as_bytes())), format!("{:x}", job.timestamp), job.clean]),
        }
    }
}

// Helper functions
fn compute_utxo_merkle_root(transactions: &[UtxoTransaction]) -> H256 {
    if transactions.is_empty() { return H256::zero(); }
    let mut hashes: Vec<H256> = transactions.iter().map(|tx| tx.txid()).collect();
    while hashes.len() > 1 {
        if hashes.len() % 2 == 1 { hashes.push(*hashes.last().unwrap()); }
        hashes = hashes.chunks(2).map(|pair| {
            let mut combined = [0u8; 64];
            combined[..32].copy_from_slice(pair[0].as_bytes());
            combined[32..].copy_from_slice(pair[1].as_bytes());
            H256::from_slice(blake3::hash(&combined).as_bytes())
        }).collect();
    }
    hashes[0]
}

fn encode_header_for_mining(parent_hash: H256, merkle_root: H256, utxo_root: H256, timestamp: u64, difficulty: u64, height: BlockNumber, beneficiary: Address) -> Vec<u8> {
    let mut buf = Vec::with_capacity(150);
    buf.extend_from_slice(&1u32.to_le_bytes());
    buf.push(0); // Stream A
    buf.extend_from_slice(parent_hash.as_bytes());
    buf.extend_from_slice(merkle_root.as_bytes());
    buf.extend_from_slice(utxo_root.as_bytes());
    buf.extend_from_slice(&timestamp.to_le_bytes());
    buf.extend_from_slice(&difficulty.to_le_bytes());
    buf.extend_from_slice(&height.to_le_bytes());
    buf.extend_from_slice(beneficiary.as_bytes());
    buf
}

fn difficulty_to_target_f64(difficulty: f64) -> H256 {
    if difficulty <= 0.0 { return H256([0xff; 32]); }
    Blake3Pow::difficulty_to_target((difficulty * 1_000_000.0) as u64)
}

fn hash_meets_target(hash: &H256, target: &H256) -> bool { hash.as_bytes() <= target.as_bytes() }

fn combine_extranonces(extranonce1: &[u8; 4], extranonce2: &[u8]) -> u64 {
    let mut bytes = [0u8; 8];
    bytes[..4].copy_from_slice(extranonce1);
    if extranonce2.len() >= 4 { bytes[4..8].copy_from_slice(&extranonce2[..4]); }
    else { bytes[4..4 + extranonce2.len()].copy_from_slice(extranonce2); }
    u64::from_le_bytes(bytes)
}

fn parse_hex_u64(s: &str) -> u64 { u64::from_str_radix(s.strip_prefix("0x").unwrap_or(s), 16).unwrap_or(0) }

fn parse_address(s: &str) -> Option<Address> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    if s.len() != 40 { return None; }
    let bytes = hex::decode(s).ok()?;
    if bytes.len() != 20 { return None; }
    Some(Address::from_slice(&bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_blake3_stratum_config_default() {
        let config = Blake3StratumConfig::default();
        assert_eq!(config.bind_addr, "0.0.0.0:3334".parse::<SocketAddr>().unwrap());
        assert_eq!(config.block_reward, 50_00000000);
    }
    
    #[test]
    fn test_combine_extranonces() {
        let en1 = [0x01, 0x02, 0x03, 0x04];
        let en2 = vec![0x05, 0x06, 0x07, 0x08];
        let result = combine_extranonces(&en1, &en2);
        assert_eq!(result, 0x0807060504030201);
    }
    
    #[test]
    fn test_parse_address() {
        let addr = parse_address("0x742d35Cc6634C0532925a3b844Bc9e7595f1dE5a");
        assert!(addr.is_some());
        let invalid = parse_address("invalid");
        assert!(invalid.is_none());
    }
    
    #[test]
    fn test_hash_meets_target() {
        let low_hash = H256([0x00; 32]);
        let high_hash = H256([0xff; 32]);
        let mid_target = H256([0x7f; 32]);
        assert!(hash_meets_target(&low_hash, &mid_target));
        assert!(!hash_meets_target(&high_hash, &mid_target));
    }
    
    #[test]
    fn test_difficulty_to_target() {
        let easy_target = difficulty_to_target_f64(0.001);
        let hard_target = difficulty_to_target_f64(1000.0);
        assert!(easy_target.as_bytes() > hard_target.as_bytes());
    }
}
