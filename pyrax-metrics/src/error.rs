//! PYRAX Chain Observer - Error Types
//!
//! Custom error types for the metrics service.

use thiserror::Error;

/// Main error type for the chain observer
#[derive(Error, Debug)]
pub enum ObserverError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),
    
    /// RPC communication error
    #[error("RPC error for {endpoint}: {message}")]
    Rpc {
        endpoint: String,
        message: String,
    },
    
    /// Timeout error
    #[error("Request timeout for {endpoint}")]
    Timeout {
        endpoint: String,
    },
    
    /// Node unreachable
    #[error("Node unreachable: {endpoint}")]
    Unreachable {
        endpoint: String,
    },
    
    /// Parse error
    #[error("Failed to parse response: {0}")]
    Parse(String),
    
    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl ObserverError {
    /// Create an RPC error
    pub fn rpc(endpoint: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Rpc {
            endpoint: endpoint.into(),
            message: message.into(),
        }
    }
    
    /// Create a timeout error
    pub fn timeout(endpoint: impl Into<String>) -> Self {
        Self::Timeout {
            endpoint: endpoint.into(),
        }
    }
    
    /// Create an unreachable error
    pub fn unreachable(endpoint: impl Into<String>) -> Self {
        Self::Unreachable {
            endpoint: endpoint.into(),
        }
    }
}

/// Result type alias for observer operations
pub type Result<T> = std::result::Result<T, ObserverError>;
