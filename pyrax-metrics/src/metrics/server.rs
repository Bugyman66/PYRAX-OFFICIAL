//! Metrics Server
//!
//! Prometheus-compatible metrics endpoint.

use crate::config::MetricsConfig;
use crate::state::AppState;
use anyhow::Result;
use axum::{
    routing::get,
    Router,
    Extension,
    extract::State,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;

/// Prometheus metrics server
pub struct MetricsServer {
    config: MetricsConfig,
    app_state: AppState,
}

impl MetricsServer {
    /// Create a new metrics server
    pub fn new(config: MetricsConfig, app_state: AppState) -> Self {
        Self { config, app_state }
    }
    
    /// Run the metrics server
    pub async fn run(&self) -> Result<()> {
        let app_state = self.app_state.clone();
        
        let app = Router::new()
            .route("/metrics", get(metrics_handler))
            .with_state(app_state);
        
        let addr: SocketAddr = self.config.listen_addr.parse()?;
        info!("Metrics server listening on {}", addr);
        
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;
        
        Ok(())
    }
}

/// Handler for /metrics endpoint
async fn metrics_handler(State(state): State<AppState>) -> String {
    let chain_state = state.chain_state();
    let fork_detected = state.is_fork_detected();
    let node_statuses = state.node_statuses();
    
    let mut output = String::new();
    
    // Chain metrics
    output.push_str("# HELP pyrax_chain_head_block Current chain head block\n");
    output.push_str("# TYPE pyrax_chain_head_block gauge\n");
    output.push_str(&format!("pyrax_chain_head_block {}\n", chain_state.head_block));
    
    output.push_str("# HELP pyrax_chain_height_delta Maximum height difference between nodes\n");
    output.push_str("# TYPE pyrax_chain_height_delta gauge\n");
    output.push_str(&format!("pyrax_chain_height_delta {}\n", chain_state.height_delta));
    
    output.push_str("# HELP pyrax_chain_block_rate Blocks per minute\n");
    output.push_str("# TYPE pyrax_chain_block_rate gauge\n");
    output.push_str(&format!("pyrax_chain_block_rate {:.2}\n", chain_state.block_rate));
    
    output.push_str("# HELP pyrax_nodes_total Total configured nodes\n");
    output.push_str("# TYPE pyrax_nodes_total gauge\n");
    output.push_str(&format!("pyrax_nodes_total {}\n", chain_state.nodes_total));
    
    output.push_str("# HELP pyrax_nodes_reachable Currently reachable nodes\n");
    output.push_str("# TYPE pyrax_nodes_reachable gauge\n");
    output.push_str(&format!("pyrax_nodes_reachable {}\n", chain_state.nodes_reachable));
    
    output.push_str("# HELP pyrax_nodes_synced Nodes that are fully synced\n");
    output.push_str("# TYPE pyrax_nodes_synced gauge\n");
    output.push_str(&format!("pyrax_nodes_synced {}\n", chain_state.nodes_synced));
    
    output.push_str("# HELP pyrax_chain_stalled Whether the chain is stalled (1=stalled)\n");
    output.push_str("# TYPE pyrax_chain_stalled gauge\n");
    output.push_str(&format!("pyrax_chain_stalled {}\n", if chain_state.is_stalled { 1 } else { 0 }));
    
    output.push_str("# HELP pyrax_chain_fork_detected Whether a fork is detected (1=forked)\n");
    output.push_str("# TYPE pyrax_chain_fork_detected gauge\n");
    output.push_str(&format!("pyrax_chain_fork_detected {}\n", if fork_detected { 1 } else { 0 }));
    
    output.push_str("# HELP pyrax_avg_latency_ms Average RPC latency in milliseconds\n");
    output.push_str("# TYPE pyrax_avg_latency_ms gauge\n");
    output.push_str(&format!("pyrax_avg_latency_ms {}\n", chain_state.avg_latency_ms));
    
    // Per-node metrics
    output.push_str("# HELP pyrax_node_block_height Block height per node\n");
    output.push_str("# TYPE pyrax_node_block_height gauge\n");
    for status in &node_statuses {
        output.push_str(&format!(
            "pyrax_node_block_height{{endpoint=\"{}\"}} {}\n",
            status.endpoint, status.block_height
        ));
    }
    
    output.push_str("# HELP pyrax_node_latency_ms RPC latency per node in milliseconds\n");
    output.push_str("# TYPE pyrax_node_latency_ms gauge\n");
    for status in &node_statuses {
        output.push_str(&format!(
            "pyrax_node_latency_ms{{endpoint=\"{}\"}} {}\n",
            status.endpoint, status.latency_ms
        ));
    }
    
    output.push_str("# HELP pyrax_node_reachable Whether node is reachable (1=yes)\n");
    output.push_str("# TYPE pyrax_node_reachable gauge\n");
    for status in &node_statuses {
        output.push_str(&format!(
            "pyrax_node_reachable{{endpoint=\"{}\"}} {}\n",
            status.endpoint, if status.reachable { 1 } else { 0 }
        ));
    }

    output.push_str("# HELP pyrax_node_peer_count Number of peers connected to this node\n");
    output.push_str("# TYPE pyrax_node_peer_count gauge\n");
    for status in &node_statuses {
        output.push_str(&format!(
            "pyrax_node_peer_count{{endpoint=\"{}\"}} {}\n",
            status.endpoint, status.peer_count
        ));
    }
    
    // Discovered nodes metrics (from crawler)
    let discovered_nodes = state.discovered_nodes();
    let discovered_count = discovered_nodes.len();
    let online_count = discovered_nodes.iter().filter(|n| n.reachable).count();
    
    output.push_str("# HELP pyrax_discovered_nodes_total Total nodes discovered by crawler\n");
    output.push_str("# TYPE pyrax_discovered_nodes_total gauge\n");
    output.push_str(&format!("pyrax_discovered_nodes_total {}\n", discovered_count));
    
    output.push_str("# HELP pyrax_discovered_nodes_online Discovered nodes that are online/reachable\n");
    output.push_str("# TYPE pyrax_discovered_nodes_online gauge\n");
    output.push_str(&format!("pyrax_discovered_nodes_online {}\n", online_count));
    
    // Per-discovered-node metrics
    if !discovered_nodes.is_empty() {
        output.push_str("# HELP pyrax_discovered_node_height Block height of discovered node\n");
        output.push_str("# TYPE pyrax_discovered_node_height gauge\n");
        for node in &discovered_nodes {
            output.push_str(&format!(
                "pyrax_discovered_node_height{{endpoint=\"{}\",peer_id=\"{}\"}} {}\n",
                node.endpoint, node.peer_id, node.best_height
            ));
        }
    }
    
    output
}
