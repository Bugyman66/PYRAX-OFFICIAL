//! OpenTelemetry Tracing for PYRAX Node

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

/// Tracing configuration
#[derive(Debug, Clone)]
pub struct TracingConfig {
    pub service_name: String,
    pub otel_endpoint: Option<String>,
    pub log_level: String,
    pub json_output: bool,
    pub log_file: Option<String>,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            service_name: "pyrax-node".to_string(),
            otel_endpoint: None,
            log_level: "info".to_string(),
            json_output: false,
            log_file: None,
        }
    }
}

/// Initialize the tracing system
pub fn init_tracing(config: TracingConfig) -> Result<(), Box<dyn std::error::Error>> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.log_level));
    
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(config.json_output)
        .with_thread_ids(config.json_output)
        .with_file(config.json_output)
        .with_line_number(config.json_output)
        .boxed();
    
    let subscriber = tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer);
    
    subscriber.init();
    Ok(())
}

/// Span attributes for common operations
pub mod spans {
    use tracing::{info_span, Span};
    
    pub fn block_processing(height: u64, hash: &str) -> Span {
        info_span!("block_processing", height = height, hash = %hash)
    }
    
    pub fn transaction_validation(tx_hash: &str) -> Span {
        info_span!("tx_validation", tx_hash = %tx_hash)
    }
    
    pub fn peer_connection(peer_id: &str, direction: &str) -> Span {
        info_span!("peer_connection", peer_id = %peer_id, direction = %direction)
    }
    
    pub fn rpc_request(method: &str) -> Span {
        info_span!("rpc_request", method = %method)
    }
    
    pub fn mining_round(nonce_start: u64, nonce_end: u64) -> Span {
        info_span!("mining_round", nonce_start = nonce_start, nonce_end = nonce_end)
    }
    
    pub fn sync_batch(start_height: u64, end_height: u64) -> Span {
        info_span!("sync_batch", start = start_height, end = end_height)
    }
}

/// Structured logging helpers
pub mod logs {
    use tracing::{info, warn, error, debug};
    
    pub fn block_received(height: u64, hash: &str, peer: &str) {
        info!(height = height, hash = %hash, peer = %peer, "Block received");
    }
    
    pub fn block_validated(height: u64, hash: &str, txs: usize) {
        info!(height = height, hash = %hash, transactions = txs, "Block validated");
    }
    
    pub fn peer_connected(peer_id: &str, addr: &str, direction: &str) {
        info!(peer_id = %peer_id, addr = %addr, direction = %direction, "Peer connected");
    }
    
    pub fn peer_disconnected(peer_id: &str, reason: &str) {
        info!(peer_id = %peer_id, reason = %reason, "Peer disconnected");
    }
    
    pub fn sync_progress(current: u64, target: u64, percent: f64) {
        info!(current = current, target = target, progress = percent, "Sync progress");
    }
    
    pub fn mining_block_found(height: u64, hash: &str, nonce: u64) {
        info!(height = height, hash = %hash, nonce = nonce, "Block mined!");
    }
    
    pub fn rpc_error(method: &str, error: &str) {
        error!(method = %method, error = %error, "RPC error");
    }
    
    pub fn validation_error(context: &str, error: &str) {
        warn!(context = %context, error = %error, "Validation error");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = TracingConfig::default();
        assert_eq!(config.service_name, "pyrax-node");
        assert_eq!(config.log_level, "info");
    }
}
