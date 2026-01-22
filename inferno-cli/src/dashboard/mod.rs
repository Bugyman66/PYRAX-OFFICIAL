// Dashboard module
// Main implementation is in commands/dashboard.rs
// This module provides additional dashboard utilities

pub const DEFAULT_PORT: u16 = 3000; // Changed from 8080 - 8080 is now a stealth P2P port
pub const DEFAULT_BIND: &str = "127.0.0.1";

pub fn get_dashboard_url(bind: &str, port: u16) -> String {
    format!("http://{}:{}", bind, port)
}
