// Remote management module
// Main implementation is in commands/remote.rs
// This module provides additional SSH/remote utilities

use std::path::PathBuf;

pub fn get_default_identity_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    
    if let Some(home) = dirs::home_dir() {
        paths.push(home.join(".ssh").join("id_rsa"));
        paths.push(home.join(".ssh").join("id_ed25519"));
        paths.push(home.join(".ssh").join("id_ecdsa"));
    }
    
    paths
}

pub fn parse_connection_string(connection: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = connection.split('@').collect();
    if parts.len() == 2 {
        Some((parts[0].to_string(), parts[1].to_string()))
    } else {
        None
    }
}
