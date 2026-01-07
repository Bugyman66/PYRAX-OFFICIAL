use crate::types::BlockHeader;
use crate::config::ConsensusConfig;

pub fn calculate_next_difficulty(
    parent: &BlockHeader,
    grandparent_timestamp: Option<u64>,
    config: &ConsensusConfig,
) -> u64 {
    // For the first few blocks, use initial difficulty
    if parent.height < 2 {
        return config.initial_difficulty;
    }

    let parent_timestamp = parent.timestamp;
    let grandparent_timestamp = grandparent_timestamp.unwrap_or(parent_timestamp - config.target_block_time);

    let actual_time = parent_timestamp.saturating_sub(grandparent_timestamp);
    let target_time = config.target_block_time;

    // Calculate adjustment ratio
    let ratio = if actual_time == 0 {
        1.0 + config.max_difficulty_change
    } else {
        (target_time as f64) / (actual_time as f64)
    };

    // Clamp ratio to max adjustment
    let clamped_ratio = ratio.clamp(
        1.0 - config.max_difficulty_change,
        1.0 + config.max_difficulty_change,
    );

    // Calculate new difficulty
    let new_difficulty = (parent.difficulty as f64 * clamped_ratio) as u64;

    // Ensure minimum difficulty
    new_difficulty.max(1)
}

pub fn calculate_window_difficulty(
    headers: &[BlockHeader],
    config: &ConsensusConfig,
) -> u64 {
    if headers.len() < 2 {
        return config.initial_difficulty;
    }

    let window_size = config.difficulty_adjustment_window as usize;
    let headers = if headers.len() > window_size {
        &headers[headers.len() - window_size..]
    } else {
        headers
    };

    let first = headers.first().unwrap();
    let last = headers.last().unwrap();

    let actual_time = last.timestamp.saturating_sub(first.timestamp);
    let expected_time = (headers.len() as u64 - 1) * config.target_block_time;

    if actual_time == 0 {
        return last.difficulty.saturating_mul(2).min(last.difficulty + last.difficulty / 4);
    }

    let ratio = (expected_time as f64) / (actual_time as f64);
    let clamped_ratio = ratio.clamp(
        1.0 - config.max_difficulty_change,
        1.0 + config.max_difficulty_change,
    );

    let new_difficulty = (last.difficulty as f64 * clamped_ratio) as u64;
    new_difficulty.max(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Address, H256};

    fn make_header(height: u64, timestamp: u64, difficulty: u64) -> BlockHeader {
        BlockHeader {
            version: 1,
            parent_hash: H256::zero(),
            merkle_root: H256::zero(),
            state_root: H256::zero(),
            timestamp,
            difficulty,
            nonce: 0,
            height,
            extra_nonce: 0,
            beneficiary: Address::ZERO,
        }
    }

    #[test]
    fn test_difficulty_increase_on_fast_blocks() {
        let config = ConsensusConfig::default();
        let parent = make_header(10, 1000, 1_000_000);
        
        // Block came 30 seconds after grandparent (fast)
        let new_diff = calculate_next_difficulty(&parent, Some(970), &config);
        
        // Should increase (ratio = 60/30 = 2, clamped to 1.25)
        assert!(new_diff > parent.difficulty);
    }

    #[test]
    fn test_difficulty_decrease_on_slow_blocks() {
        let config = ConsensusConfig::default();
        let parent = make_header(10, 1000, 1_000_000);
        
        // Block came 120 seconds after grandparent (slow)
        let new_diff = calculate_next_difficulty(&parent, Some(880), &config);
        
        // Should decrease (ratio = 60/120 = 0.5, clamped to 0.75)
        assert!(new_diff < parent.difficulty);
    }
}
