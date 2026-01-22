//! API Server
//!
//! Health and status JSON API endpoints.

use crate::config::ApiConfig;
use crate::state::AppState;
use anyhow::Result;
use axum::{
    routing::get,
    Router,
    Json,
    extract::State,
    response::{Html, IntoResponse, Response},
    http::{header, StatusCode},
};
use rust_embed::Embed;
use serde::Serialize;
use std::net::SocketAddr;
use tower_http::cors::{CorsLayer, Any};
use tracing::info;

/// Embedded static files
#[derive(Embed)]
#[folder = "static/"]
struct Assets;

/// Health/Status API server
pub struct ApiServer {
    config: ApiConfig,
    app_state: AppState,
}

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub chain_status: String,
    pub nodes_checked: usize,
    pub nodes_reachable: usize,
    pub head_block: u64,
    pub height_delta: u64,
    pub last_check: String,
}

/// Detailed status response
#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub chain: ChainStatus,
    pub nodes: Vec<NodeInfo>,
    pub alerts: AlertsStatus,
}

#[derive(Debug, Serialize)]
pub struct ChainStatus {
    pub head_block: u64,
    pub lowest_block: u64,
    pub height_delta: u64,
    pub block_rate: f64,
    pub avg_latency_ms: u64,
    pub stalled: bool,
    pub fork_detected: bool,
    // Mining fields
    pub network_hashrate: u64,
    pub difficulty: u64,
    pub total_blocks_found: u64,
    // Discovery fields
    pub discovered_nodes: usize,
    pub online_nodes: usize,
}

#[derive(Debug, Serialize)]
pub struct NodeInfo {
    pub endpoint: String,
    pub height: u64,
    pub reachable: bool,
    pub syncing: bool,
    pub peer_count: u32,
    pub latency_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct AlertsStatus {
    pub stalled: bool,
    pub fork_detected: bool,
    pub nodes_unreachable: usize,
}

impl ApiServer {
    /// Create a new API server
    pub fn new(config: ApiConfig, app_state: AppState) -> Self {
        Self { config, app_state }
    }
    
    /// Run the API server
    pub async fn run(&self) -> Result<()> {
        let app_state = self.app_state.clone();
        
        let mut app = Router::new()
            .route("/", get(dashboard_handler))
            .route("/health", get(health_handler))
            .route("/status", get(status_handler))
            .with_state(app_state);
        
        if self.config.enable_cors {
            app = app.layer(CorsLayer::new().allow_origin(Any));
        }
        
        let addr: SocketAddr = self.config.listen_addr.parse()?;
        info!("API server listening on {}", addr);
        
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;
        
        Ok(())
    }
}

/// Handler for /health endpoint
async fn health_handler(State(state): State<AppState>) -> Json<HealthResponse> {
    let chain_state = state.chain_state();
    let fork_detected = state.is_fork_detected();
    
    let status = if chain_state.is_stalled || fork_detected {
        "degraded"
    } else if chain_state.nodes_reachable == 0 {
        "unhealthy"
    } else {
        "healthy"
    };
    
    let chain_status = if chain_state.is_stalled {
        "stalled"
    } else if fork_detected {
        "forked"
    } else if chain_state.nodes_reachable > 0 {
        "live"
    } else {
        "unknown"
    };
    
    Json(HealthResponse {
        status: status.to_string(),
        chain_status: chain_status.to_string(),
        nodes_checked: chain_state.nodes_total,
        nodes_reachable: chain_state.nodes_reachable,
        head_block: chain_state.head_block,
        height_delta: chain_state.height_delta,
        last_check: chrono::Utc::now().to_rfc3339(),
    })
}

/// Handler for /status endpoint
async fn status_handler(State(state): State<AppState>) -> Json<StatusResponse> {
    let chain_state = state.chain_state();
    let fork_detected = state.is_fork_detected();
    let node_statuses = state.node_statuses();
    let discovered_nodes = state.discovered_nodes();
    
    let nodes: Vec<NodeInfo> = node_statuses.iter().map(|s| NodeInfo {
        endpoint: s.endpoint.clone(),
        height: s.block_height,
        reachable: s.reachable,
        syncing: s.syncing,
        peer_count: s.peer_count,
        latency_ms: s.latency_ms,
    }).collect();
    
    let nodes_unreachable = node_statuses.iter().filter(|s| !s.reachable).count();
    let online_count = discovered_nodes.iter().filter(|n| n.reachable).count();
    
    Json(StatusResponse {
        chain: ChainStatus {
            head_block: chain_state.head_block,
            lowest_block: chain_state.lowest_block,
            height_delta: chain_state.height_delta,
            block_rate: chain_state.block_rate,
            avg_latency_ms: chain_state.avg_latency_ms,
            stalled: chain_state.is_stalled,
            fork_detected,
            network_hashrate: chain_state.network_hashrate,
            difficulty: chain_state.difficulty,
            total_blocks_found: chain_state.total_blocks_found,
            discovered_nodes: discovered_nodes.len(),
            online_nodes: online_count,
        },
        nodes,
        alerts: AlertsStatus {
            stalled: chain_state.is_stalled,
            fork_detected,
            nodes_unreachable,
        },
    })
}

/// Handler for / endpoint - serves the dashboard
async fn dashboard_handler() -> impl IntoResponse {
    match Assets::get("index.html") {
        Some(content) => {
            let body = content.data.into_owned();
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                .header(header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")
                .body(axum::body::Body::from(body))
                .unwrap()
        }
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(axum::body::Body::from("Dashboard not found"))
            .unwrap(),
    }
}
