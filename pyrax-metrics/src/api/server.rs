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
    let chain = state.chain_state();
    let nodes_count = state.node_statuses().len();
    
    Json(json!({
        "chain": chain,
        "nodes_monitored": nodes_count,
        "fork_detected": state.is_fork_detected(),
        "status": if chain.is_stalled { "stalled" } else { "running" }
    }))
}

async fn get_nodes(State(state): State<AppState>) -> Json<Value> {
    let nodes = state.node_statuses();
    Json(json!(nodes))
}
