//! JSON-RPC Server for PYRAX
//!
//! Production-ready RPC server using jsonrpsee

use std::sync::Arc;
use std::net::SocketAddr;
use jsonrpsee::server::{ServerBuilder, ServerHandle};
use jsonrpsee::core::{async_trait, RpcResult};
use jsonrpsee::proc_macros::rpc;
use tracing::info;

use super::{RpcError, RpcBlock, RpcTransaction, RpcChainInfo, RpcPeerInfo, RpcMempoolInfo, RpcBlockTemplate, RpcSubmitResult, RpcBalance, RpcUtxo};
use crate::storage::ChainDB;
use crate::types::{H256, Address, Transaction, TxInput, TxOutput, Block, NetworkId, OutPoint};
use crate::mempool::Mempool;

/// PYRAX JSON-RPC API
#[rpc(server)]
pub trait PyraxRpc {
    /// Get chain ID
    #[method(name = "pyrax_chainId")]
    async fn chain_id(&self) -> RpcResult<u32>;

    /// Get chain info including tip, height, UTXO count
    #[method(name = "pyrax_getChainInfo")]
    async fn get_chain_info(&self) -> RpcResult<RpcChainInfo>;

    /// Get block by hash
    #[method(name = "pyrax_getBlockByHash")]
    async fn get_block_by_hash(&self, hash: String, full_txs: bool) -> RpcResult<Option<RpcBlock>>;

    /// Get block by height
    #[method(name = "pyrax_getBlockByNumber")]
    async fn get_block_by_number(&self, height: u64, full_txs: bool) -> RpcResult<Option<RpcBlock>>;

    /// Get transaction by txid
    #[method(name = "pyrax_getTransaction")]
    async fn get_transaction(&self, txid: String) -> RpcResult<Option<RpcTransaction>>;

    /// Get UTXO balance for an address (script pubkey hash)
    #[method(name = "pyrax_getBalance")]
    async fn get_balance(&self, address: String) -> RpcResult<RpcBalance>;

    /// Get UTXOs for an address
    #[method(name = "pyrax_getUtxos")]
    async fn get_utxos(&self, address: String) -> RpcResult<Vec<RpcUtxo>>;

    /// Send raw transaction (hex encoded)
    #[method(name = "pyrax_sendRawTransaction")]
    async fn send_raw_transaction(&self, tx_hex: String) -> RpcResult<RpcSubmitResult>;

    /// Get mempool info
    #[method(name = "pyrax_getMempoolInfo")]
    async fn get_mempool_info(&self) -> RpcResult<RpcMempoolInfo>;

    /// Get block template for mining
    #[method(name = "pyrax_getBlockTemplate")]
    async fn get_block_template(&self) -> RpcResult<RpcBlockTemplate>;

    /// Submit mined block
    #[method(name = "pyrax_submitBlock")]
    async fn submit_block(&self, block_hex: String) -> RpcResult<RpcSubmitResult>;
    
    /// Create a test transaction (devnet only) - spends from one address to another
    #[method(name = "pyrax_createTestTransaction")]
    async fn create_test_transaction(&self, from_address: String, to_address: String, amount: u64) -> RpcResult<RpcSubmitResult>;

    /// Get network peer information
    #[method(name = "pyrax_getNetworkInfo")]
    async fn get_network_info(&self) -> RpcResult<super::RpcNetworkInfo>;

    /// Simple health check - returns immediately without database access
    #[method(name = "pyrax_health")]
    async fn health(&self) -> RpcResult<String>;
}

/// RPC server state
pub struct RpcServerImpl {
    db: Arc<ChainDB>,
    network_id: NetworkId,
    mempool: Option<Arc<Mempool>>,
    peer_registry: Option<crate::p2p::PeerRegistry>,
}

impl RpcServerImpl {
    pub fn new(db: Arc<ChainDB>, network_id: NetworkId) -> Self {
        Self { db, network_id, mempool: None, peer_registry: None }
    }
    
    pub fn with_mempool(db: Arc<ChainDB>, network_id: NetworkId, mempool: Arc<Mempool>) -> Self {
        Self { db, network_id, mempool: Some(mempool), peer_registry: None }
    }

    pub fn with_mempool_and_peers(
        db: Arc<ChainDB>,
        network_id: NetworkId,
        mempool: Arc<Mempool>,
        peer_registry: crate::p2p::PeerRegistry,
    ) -> Self {
        Self { db, network_id, mempool: Some(mempool), peer_registry: Some(peer_registry) }
    }
}

#[async_trait]
impl PyraxRpcServer for RpcServerImpl {
    async fn chain_id(&self) -> RpcResult<u32> {
        Ok(self.network_id.0)
    }

    async fn get_chain_info(&self) -> RpcResult<RpcChainInfo> {
        let tip = self.db.get_tip();
        let utxo_count = self.db.utxo_count().unwrap_or(0);

        Ok(RpcChainInfo {
            chain_id: self.network_id.0,
            network: self.network_id.name().to_string(),
            best_block_hash: format!("0x{}", hex::encode(&tip.hash.0)),
            best_block_height: tip.height,
            genesis_hash: self.db.genesis_hash()
                .map(|h| format!("0x{}", hex::encode(&h.0)))
                .unwrap_or_else(|_| "0x0".to_string()),
            difficulty: tip.total_difficulty,
            utxo_count,
            syncing: false,
        })
    }

    async fn get_block_by_hash(&self, hash: String, full_txs: bool) -> RpcResult<Option<RpcBlock>> {
        let hash = parse_hash(&hash)?;
        
        match self.db.get_block(&hash) {
            Ok(Some(block)) => {
                let mut rpc_block = RpcBlock::from_block(&block);
                if full_txs {
                    rpc_block = rpc_block.with_transactions(&block);
                }
                Ok(Some(rpc_block))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(RpcError::InternalError(e.to_string()).into()),
        }
    }

    async fn get_block_by_number(&self, height: u64, full_txs: bool) -> RpcResult<Option<RpcBlock>> {
        match self.db.get_block_by_height(height) {
            Ok(Some(block)) => {
                let mut rpc_block = RpcBlock::from_block(&block);
                if full_txs {
                    rpc_block = rpc_block.with_transactions(&block);
                }
                Ok(Some(rpc_block))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(RpcError::InternalError(e.to_string()).into()),
        }
    }

    async fn get_transaction(&self, txid: String) -> RpcResult<Option<RpcTransaction>> {
        let txid = parse_hash(&txid)?;
        
        match self.db.get_transaction(&txid) {
            Ok(Some(tx)) => {
                Ok(Some(RpcTransaction::from_tx(&tx, None, None, None)))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(RpcError::InternalError(e.to_string()).into()),
        }
    }

    async fn get_balance(&self, address: String) -> RpcResult<RpcBalance> {
        let addr = parse_address(&address)?;
        
        // Get UTXOs for this address by scanning the UTXO set
        let utxos = self.db.get_utxos_for_address(&addr)
            .map_err(|e| RpcError::InternalError(e.to_string()))?;
        
        let balance: u64 = utxos.iter().map(|(_, u)| u.output.value).sum();
        let utxo_list: Vec<RpcUtxo> = utxos.iter().map(|(outpoint, utxo)| {
            RpcUtxo {
                txid: format!("0x{}", hex::encode(&outpoint.txid.0)),
                vout: outpoint.vout,
                value: utxo.output.value,
                script_pubkey: format!("0x{}", hex::encode(&utxo.output.script_pubkey)),
                height: utxo.height,
                coinbase: utxo.is_coinbase,
            }
        }).collect();
        
        Ok(RpcBalance {
            address,
            balance,
            utxo_count: utxo_list.len(),
            utxos: utxo_list,
        })
    }

    async fn get_utxos(&self, address: String) -> RpcResult<Vec<RpcUtxo>> {
        let addr = parse_address(&address)?;
        
        // Get UTXOs for this address by scanning the UTXO set
        let utxos = self.db.get_utxos_for_address(&addr)
            .map_err(|e| RpcError::InternalError(e.to_string()))?;
        
        let utxo_list: Vec<RpcUtxo> = utxos.iter().map(|(outpoint, utxo)| {
            RpcUtxo {
                txid: format!("0x{}", hex::encode(&outpoint.txid.0)),
                vout: outpoint.vout,
                value: utxo.output.value,
                script_pubkey: format!("0x{}", hex::encode(&utxo.output.script_pubkey)),
                height: utxo.height,
                coinbase: utxo.is_coinbase,
            }
        }).collect();
        
        Ok(utxo_list)
    }

    async fn send_raw_transaction(&self, tx_hex: String) -> RpcResult<RpcSubmitResult> {
        let tx_hex = tx_hex.strip_prefix("0x").unwrap_or(&tx_hex);
        let tx_bytes = hex::decode(tx_hex)
            .map_err(|_| RpcError::InvalidParams("Invalid hex".to_string()))?;
        
        let tx: Transaction = bincode::deserialize(&tx_bytes)
            .map_err(|e| RpcError::InvalidParams(format!("Invalid transaction: {}", e)))?;

        let txid = tx.txid();
        
        // Validate inputs exist in UTXO set and calculate input value
        let mut input_value = 0u64;
        for input in &tx.inputs {
            if input.is_coinbase() {
                return Ok(RpcSubmitResult {
                    accepted: false,
                    hash: None,
                    error: Some("Coinbase transactions not allowed".to_string()),
                });
            }
            
            match self.db.get_utxo(&input.previous_output) {
                Ok(Some(utxo)) => {
                    input_value += utxo.output.value;
                }
                Ok(None) => {
                    return Ok(RpcSubmitResult {
                        accepted: false,
                        hash: None,
                        error: Some(format!("UTXO not found: {}:{}", input.previous_output.txid, input.previous_output.vout)),
                    });
                }
                Err(e) => {
                    return Ok(RpcSubmitResult {
                        accepted: false,
                        hash: None,
                        error: Some(format!("Database error: {}", e)),
                    });
                }
            }
        }
        
        // Check output value doesn't exceed input
        let output_value = tx.total_output();
        if output_value > input_value {
            return Ok(RpcSubmitResult {
                accepted: false,
                hash: None,
                error: Some(format!("Output value {} exceeds input value {}", output_value, input_value)),
            });
        }
        
        // Add to mempool if available
        if let Some(ref mempool) = self.mempool {
            if let Err(e) = mempool.add(tx, input_value) {
                return Ok(RpcSubmitResult {
                    accepted: false,
                    hash: None,
                    error: Some(format!("Mempool error: {}", e)),
                });
            }
            info!("Transaction {} added to mempool", txid);
        } else {
            info!("Transaction {} validated (no mempool)", txid);
        }
        
        Ok(RpcSubmitResult {
            accepted: true,
            hash: Some(format!("0x{}", hex::encode(&txid.0))),
            error: None,
        })
    }

    async fn get_mempool_info(&self) -> RpcResult<RpcMempoolInfo> {
        let (size, bytes) = if let Some(ref mp) = self.mempool {
            (mp.len(), mp.total_bytes())
        } else {
            (0, 0)
        };
        Ok(RpcMempoolInfo {
            size,
            bytes,
        })
    }

    async fn get_block_template(&self) -> RpcResult<RpcBlockTemplate> {
        let tip = self.db.get_tip();
        let block_reward = 50 * 100_000_000; // 50 PYRAX in satoshis

        Ok(RpcBlockTemplate {
            height: tip.height + 1,
            parent_hash: format!("0x{}", hex::encode(&tip.hash.0)),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            difficulty: 1, // Devnet difficulty
            target: "0x00ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_string(),
            transactions: vec![],
            coinbase_value: block_reward,
        })
    }

    async fn submit_block(&self, block_hex: String) -> RpcResult<RpcSubmitResult> {
        let block_hex = block_hex.strip_prefix("0x").unwrap_or(&block_hex);
        let block_bytes = hex::decode(block_hex)
            .map_err(|_| RpcError::InvalidParams("Invalid hex".to_string()))?;

        let block: Block = bincode::deserialize(&block_bytes)
            .map_err(|e| RpcError::InvalidParams(format!("Invalid block: {}", e)))?;

        let hash = block.hash();

        match self.db.commit_block(&block) {
            Ok(()) => {
                info!("RPC: Block {} accepted at height {}", hash, block.height());
                Ok(RpcSubmitResult {
                    accepted: true,
                    hash: Some(format!("0x{}", hex::encode(&hash.0))),
                    error: None,
                })
            }
            Err(e) => Ok(RpcSubmitResult {
                accepted: false,
                hash: None,
                error: Some(e.to_string()),
            }),
        }
    }
    
    async fn create_test_transaction(&self, from_address: String, to_address: String, amount: u64) -> RpcResult<RpcSubmitResult> {
        // Only allow on devnet
        if self.network_id != NetworkId::DEVNET {
            return Ok(RpcSubmitResult {
                accepted: false,
                hash: None,
                error: Some("Test transactions only allowed on devnet".to_string()),
            });
        }
        
        let from_addr = parse_address(&from_address)?;
        let to_addr = parse_address(&to_address)?;
        
        // Get UTXOs for the from address
        let utxos = self.db.get_utxos_for_address(&from_addr)
            .map_err(|e| RpcError::InternalError(e.to_string()))?;
        
        if utxos.is_empty() {
            return Ok(RpcSubmitResult {
                accepted: false,
                hash: None,
                error: Some("No UTXOs available for from_address".to_string()),
            });
        }
        
        // Select first UTXO that covers the amount + fee
        let fee = 1000u64;
        let required = amount + fee;
        
        let (outpoint, utxo) = utxos.into_iter()
            .find(|(_, u)| u.output.value >= required)
            .ok_or_else(|| RpcError::InvalidParams("No UTXO with sufficient value".to_string()))?;
        
        let change = utxo.output.value - required;
        
        // Create transaction with dummy signature (test only - real txs need proper signing)
        let input = TxInput::new(outpoint, vec![0u8; 65]); // Dummy signature for test
        
        let mut outputs = vec![TxOutput::p2pkh(amount, &to_addr)];
        if change > 0 {
            outputs.push(TxOutput::p2pkh(change, &from_addr));
        }
        
        let tx = Transaction::new(vec![input], outputs);
        let txid = tx.txid();
        
        // Add to mempool if available
        if let Some(ref mempool) = self.mempool {
            let input_value = utxo.output.value;
            if let Err(e) = mempool.add(tx, input_value) {
                return Ok(RpcSubmitResult {
                    accepted: false,
                    hash: None,
                    error: Some(format!("Mempool error: {}", e)),
                });
            }
            info!("Test transaction {} added to mempool (from: {}, to: {}, amount: {})", 
                txid, from_address, to_address, amount);
            
            Ok(RpcSubmitResult {
                accepted: true,
                hash: Some(format!("0x{}", hex::encode(&txid.0))),
                error: None,
            })
        } else {
            Ok(RpcSubmitResult {
                accepted: false,
                hash: None,
                error: Some("Mempool not available".to_string()),
            })
        }
    }

    async fn get_network_info(&self) -> RpcResult<super::RpcNetworkInfo> {
        if let Some(ref registry) = self.peer_registry {
            let peers = registry.get_peers().await;
            let local_peer_id = registry.local_peer_id().await;
            let listen_addresses = registry.listen_addresses().await;

            let rpc_peers: Vec<super::RpcPeerInfo> = peers.iter().map(|p| {
                super::RpcPeerInfo {
                    peer_id: p.peer_id.clone(),
                    address: p.address.clone(),
                    ip: p.ip.clone(),
                    port: p.port,
                    protocol: format!("/pyrax/{}/1.0.0", self.network_id.name()),
                    direction: p.direction.to_string(),
                    connected_secs: p.connected_at.elapsed().as_secs(),
                    last_seen: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64 - p.last_seen.elapsed().as_millis() as u64,
                    version: p.client_version.clone(),
                    block_height: p.best_height,
                }
            }).collect();

            // Get extended P2P stats from registry
            let metrics = registry.get_metrics().await;
            
            // Convert mesh connections to RPC format
            let rpc_mesh_connections: Vec<super::RpcMeshConnection> = metrics.mesh_connections.iter().map(|c| {
                super::RpcMeshConnection {
                    peer_a: c.peer_a.clone(),
                    peer_b: c.peer_b.clone(),
                    topic: c.topic.clone(),
                    connection_type: c.connection_type.clone(),
                }
            }).collect();
            
            let rpc_relay_circuits: Vec<super::RpcRelayCircuit> = metrics.relay_circuits.iter().map(|c| {
                super::RpcRelayCircuit {
                    src_peer: c.src_peer.clone(),
                    dst_peer: c.dst_peer.clone(),
                    established_at: c.established_at,
                }
            }).collect();
            
            Ok(super::RpcNetworkInfo {
                peer_count: rpc_peers.len(),
                peers: rpc_peers,
                local_peer_id,
                listen_addresses,
                // Extended P2P stats
                inbound_peers: metrics.inbound_peers,
                outbound_peers: metrics.outbound_peers,
                target_peers: metrics.target_peers,
                max_peers: metrics.max_peers,
                dial_attempts: metrics.dial_attempts,
                dial_successes: metrics.dial_successes,
                dial_failures: metrics.dial_failures,
                average_rtt_ms: metrics.average_rtt_ms,
                network_state: metrics.network_state.clone(),
                nat_status: metrics.nat_status.clone(),
                mesh_peers: metrics.mesh_peers,
                gossip_peers: metrics.gossip_peers,
                // Mesh topology for visualizer
                mesh_connections: rpc_mesh_connections,
                relay_circuits: rpc_relay_circuits,
            })
        } else {
            Ok(super::RpcNetworkInfo {
                peer_count: 0,
                peers: vec![],
                local_peer_id: String::new(),
                listen_addresses: vec![],
                // Default extended stats
                inbound_peers: 0,
                outbound_peers: 0,
                target_peers: 50,
                max_peers: 60,
                dial_attempts: 0,
                dial_successes: 0,
                dial_failures: 0,
                average_rtt_ms: None,
                network_state: "Disconnected".to_string(),
                nat_status: "Unknown".to_string(),
                mesh_peers: 0,
                gossip_peers: 0,
                mesh_connections: vec![],
                relay_circuits: vec![],
            })
        }
    }

    async fn health(&self) -> RpcResult<String> {
        Ok("ok".to_string())
    }
}

/// Start the RPC server
pub async fn start_server(
    addr: &str,
    db: Arc<ChainDB>,
    network_id: NetworkId,
) -> Result<ServerHandle, Box<dyn std::error::Error + Send + Sync>> {
    let addr: SocketAddr = addr.parse()?;
    
    let server = ServerBuilder::default()
        .build(addr)
        .await?;

    let rpc = RpcServerImpl::new(db, network_id);
    let handle = server.start(rpc.into_rpc());

    info!("JSON-RPC server started on http://{}", addr);
    Ok(handle)
}

/// Start the RPC server with mempool support
pub async fn start_server_with_mempool(
    addr: &str,
    db: Arc<ChainDB>,
    network_id: NetworkId,
    mempool: Arc<Mempool>,
) -> Result<ServerHandle, Box<dyn std::error::Error + Send + Sync>> {
    let addr: SocketAddr = addr.parse()?;
    
    let server = ServerBuilder::default()
        .build(addr)
        .await?;

    let rpc = RpcServerImpl::with_mempool(db, network_id, mempool);
    let handle = server.start(rpc.into_rpc());

    info!("JSON-RPC server started on http://{} (with mempool)", addr);
    Ok(handle)
}

/// Start the RPC server with mempool and P2P peer registry
pub async fn start_server_with_peers(
    addr: &str,
    db: Arc<ChainDB>,
    network_id: NetworkId,
    mempool: Arc<Mempool>,
    peer_registry: crate::p2p::PeerRegistry,
) -> Result<ServerHandle, Box<dyn std::error::Error + Send + Sync>> {
    let addr: SocketAddr = addr.parse()?;
    
    let server = ServerBuilder::default()
        .build(addr)
        .await?;

    let rpc = RpcServerImpl::with_mempool_and_peers(db, network_id, mempool, peer_registry);
    let handle = server.start(rpc.into_rpc());

    info!("JSON-RPC server started on http://{} (with mempool + P2P)", addr);
    Ok(handle)
}

/// Start the Staking RPC server for Stream C
pub async fn start_staking_server(
    addr: &str,
    staking_service: Arc<crate::services::staking::StakingService>,
) -> Result<ServerHandle, Box<dyn std::error::Error + Send + Sync>> {
    use super::staking_rpc::{StakingRpcImpl, StakingRpcServer};
    
    let addr: SocketAddr = addr.parse()?;
    
    let server = ServerBuilder::default()
        .build(addr)
        .await?;

    let rpc = StakingRpcImpl::new(staking_service);
    let handle = server.start(rpc.into_rpc());

    info!("Stream C Staking RPC server started on http://{}", addr);
    Ok(handle)
}

fn parse_hash(s: &str) -> Result<H256, RpcError> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    let bytes = hex::decode(s)
        .map_err(|_| RpcError::InvalidParams("Invalid hash hex".to_string()))?;
    if bytes.len() != 32 {
        return Err(RpcError::InvalidParams("Hash must be 32 bytes".to_string()));
    }
    Ok(H256::from_slice(&bytes))
}

fn parse_address(s: &str) -> Result<Address, RpcError> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    let bytes = hex::decode(s)
        .map_err(|_| RpcError::InvalidParams("Invalid address hex".to_string()))?;
    if bytes.len() != 20 {
        return Err(RpcError::InvalidParams("Address must be 20 bytes".to_string()));
    }
    Ok(Address::from_slice(&bytes))
}
