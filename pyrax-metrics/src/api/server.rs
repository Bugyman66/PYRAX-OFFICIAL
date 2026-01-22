use crate::config::ApiConfig;
use crate::state::AppState;
use axum::{
    routing::get,
    Router,
    response::Json,
    extract::State,
};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use serde_json::{json, Value};

pub struct ApiServer {
    config: ApiConfig,
    app_state: AppState,
}

impl ApiServer {
    pub fn new(config: ApiConfig, app_state: AppState) -> Self {
        Self { config, app_state }
    }
    
    pub async fn run(&self) -> anyhow::Result<()> {
        let app = Router::new()
            .route("/health", get(health_check))
            .route("/status", get(get_status))
            .route("/api/nodes", get(get_nodes))
            .with_state(self.app_state.clone());
            
        let addr: SocketAddr = self.config.listen_addr.parse()?;
        let listener = TcpListener::bind(addr).await?;
        
        tracing::info!("API server listening on {}", addr);
        axum::serve(listener, app).await?;
        
        Ok(())
    }
}

async fn health_check() -> &'static str {
    "ok"
}

async fn get_status(State(state): State<AppState>) -> Json<Value> {
    let chain_state = state.chain_state();
    let node_statuses = state.node_statuses();
    let discovered = state.discovered_nodes();
    
    // Build chain object with all fields the dashboard expects
    let chain = json!({
        "head_block": chain_state.head_block,
        "height_delta": chain_state.height_delta,
        "nodes_reachable": chain_state.nodes_reachable,
        "nodes_total": chain_state.nodes_total,
        "nodes_synced": chain_state.nodes_synced,
        "online_nodes": chain_state.nodes_reachable,
        "discovered_nodes": discovered.len(),
        "is_stalled": chain_state.is_stalled,
        "time_since_last_block": chain_state.time_since_last_block,
        "network_hashrate": chain_state.network_hashrate,
        "block_rate": chain_state.block_rate,
        "avg_latency_ms": chain_state.avg_latency_ms,
    });
    
    // Build alerts object
    let alerts = json!({
        "stalled": chain_state.is_stalled,
        "fork_detected": state.is_fork_detected(),
    });
    
    // Build nodes array
    let nodes: Vec<Value> = node_statuses.iter().map(|n| {
        json!({
            "endpoint": n.endpoint,
            "reachable": n.reachable,
            "block_height": n.block_height,
            "syncing": n.syncing,
            "latency_ms": n.latency_ms,
            "peer_count": n.peer_count,
        })
    }).collect();
    
    Json(json!({
        "chain": chain,
        "nodes": nodes,
        "alerts": alerts,
    }))
}

async fn get_nodes(State(state): State<AppState>) -> Json<Value> {
    let nodes = state.node_statuses();
    let nodes_json: Vec<Value> = nodes.iter().map(|n| {
        json!({
            "endpoint": n.endpoint,
            "reachable": n.reachable,
            "block_height": n.block_height,
            "syncing": n.syncing,
            "latency_ms": n.latency_ms,
            "peer_count": n.peer_count,
        })
    }).collect();
    Json(json!(nodes_json))
}
