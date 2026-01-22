//! Stream Monitor Module
//!
//! Monitors individual stream health (A/B/C).

use crate::config::StreamsConfig;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Stream identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stream {
    A,
    B,
    C,
}

impl Stream {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "a" => Some(Stream::A),
            "b" => Some(Stream::B),
            "c" => Some(Stream::C),
            _ => None,
        }
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            Stream::A => "a",
            Stream::B => "b",
            Stream::C => "c",
        }
    }
}

/// State of a single stream
#[derive(Debug, Clone, Default)]
pub struct StreamState {
    pub block_height: u64,
    pub block_rate: f64,
    pub hash_rate: u64,
    pub active_miners: u32,
    pub is_stalled: bool,
    pub last_block_time: Option<Instant>,
}

/// Stream monitor
pub struct StreamMonitor {
    config: StreamsConfig,
    streams: HashMap<Stream, StreamState>,
}

impl StreamMonitor {
    /// Create a new stream monitor
    pub fn new(config: StreamsConfig) -> Self {
        let mut streams = HashMap::new();
        
        for id in &config.stream_ids {
            if let Some(stream) = Stream::from_str(id) {
                streams.insert(stream, StreamState::default());
            }
        }
        
        Self { config, streams }
    }
    
    /// Update stream state
    pub fn update_stream(&mut self, stream: Stream, height: u64, hash_rate: u64, miners: u32) {
        // Compute stall status first (before mutable borrow)
        let is_stalled = self.check_stalled(stream);
        
        if let Some(state) = self.streams.get_mut(&stream) {
            // Calculate block rate
            if height > state.block_height {
                if let Some(last_time) = state.last_block_time {
                    let elapsed = last_time.elapsed().as_millis() as f64;
                    let blocks = (height - state.block_height) as f64;
                    state.block_rate = (blocks / elapsed) * 60_000.0; // blocks per minute
                }
                state.last_block_time = Some(Instant::now());
            }
            
            state.block_height = height;
            state.hash_rate = hash_rate;
            state.active_miners = miners;
            state.is_stalled = is_stalled;
        }
    }
    
    /// Get stream state
    pub fn get_stream(&self, stream: Stream) -> Option<&StreamState> {
        self.streams.get(&stream)
    }
    
    /// Get all stream states
    pub fn all_streams(&self) -> &HashMap<Stream, StreamState> {
        &self.streams
    }
    
    /// Calculate maximum height delta between streams
    pub fn height_delta(&self) -> u64 {
        let heights: Vec<u64> = self.streams.values().map(|s| s.block_height).collect();
        if heights.is_empty() {
            return 0;
        }
        let max = heights.iter().max().unwrap_or(&0);
        let min = heights.iter().min().unwrap_or(&0);
        max.saturating_sub(*min)
    }
    
    /// Check if a stream is stalled
    fn check_stalled(&self, stream: Stream) -> bool {
        if let Some(state) = self.streams.get(&stream) {
            if let Some(last_time) = state.last_block_time {
                let threshold = match stream {
                    Stream::A => Duration::from_millis(self.config.stream_a_block_time_ms * 6),
                    Stream::B => Duration::from_millis(self.config.stream_b_block_time_ms * 3),
                    Stream::C => Duration::from_secs(600), // 10 minutes for checkpoints
                };
                return last_time.elapsed() > threshold;
            }
        }
        false
    }
}
