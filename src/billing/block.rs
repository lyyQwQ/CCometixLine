use crate::billing::{BillingBlock, UsageEntry};
use chrono::{DateTime, Duration, Timelike, Utc};
use std::collections::HashMap;

/// Identify 5-hour billing blocks from usage entries
pub fn identify_session_blocks(entries: &[UsageEntry]) -> Vec<BillingBlock> {
    if entries.is_empty() {
        return Vec::new();
    }

    // Group entries by their 5-hour block
    let mut blocks_map: HashMap<DateTime<Utc>, Vec<&UsageEntry>> = HashMap::new();

    for entry in entries {
        let block_start = get_block_start(entry.timestamp);
        blocks_map.entry(block_start).or_default().push(entry);
    }

    // Convert to BillingBlock objects
    let mut blocks: Vec<BillingBlock> = blocks_map
        .into_iter()
        .map(|(start_time, block_entries)| {
            let end_time = start_time + Duration::hours(5);
            let now = Utc::now();

            // Calculate total tokens and sessions
            let mut session_ids = std::collections::HashSet::new();
            let mut total_tokens = 0u32;
            let mut total_cost = 0.0;

            for entry in &block_entries {
                session_ids.insert(&entry.session_id);
                total_tokens += entry.input_tokens
                    + entry.output_tokens
                    + entry.cache_creation_tokens
                    + entry.cache_read_tokens;

                // Add cost if available
                if let Some(cost) = entry.cost {
                    total_cost += cost;
                }
            }

            // Check if block is currently active
            let is_active = now >= start_time && now <= end_time;

            // Calculate remaining minutes
            let remaining_minutes = if is_active {
                ((end_time - now).num_seconds() / 60).max(0)
            } else {
                0
            };

            BillingBlock {
                start_time,
                end_time,
                cost: total_cost,
                remaining_minutes,
                is_active,
                session_count: session_ids.len(),
                total_tokens,
            }
        })
        .collect();

    // Sort by start time
    blocks.sort_by_key(|b| b.start_time);

    // Merge consecutive blocks that should be part of the same session
    merge_consecutive_blocks(blocks)
}

/// Get the start time of the 5-hour block for a given timestamp
fn get_block_start(timestamp: DateTime<Utc>) -> DateTime<Utc> {
    // Round down to the nearest hour
    let hour = timestamp.hour();
    let block_hour = (hour / 5) * 5; // 0, 5, 10, 15, 20

    timestamp
        .with_hour(block_hour)
        .unwrap()
        .with_minute(0)
        .unwrap()
        .with_second(0)
        .unwrap()
        .with_nanosecond(0)
        .unwrap()
}

/// Merge consecutive blocks that are part of the same session
fn merge_consecutive_blocks(mut blocks: Vec<BillingBlock>) -> Vec<BillingBlock> {
    if blocks.len() <= 1 {
        return blocks;
    }

    let mut merged = Vec::new();
    let mut current_block = blocks.remove(0);

    for block in blocks {
        // Check if this block is consecutive (starts when the previous ends)
        if block.start_time == current_block.end_time {
            // Merge the blocks
            current_block.end_time = block.end_time;
            current_block.cost += block.cost;
            current_block.total_tokens += block.total_tokens;
            current_block.session_count = current_block.session_count.max(block.session_count);
            current_block.is_active = current_block.is_active || block.is_active;
            if block.is_active {
                current_block.remaining_minutes = block.remaining_minutes;
            }
        } else {
            // Gap between blocks - save current and start new
            merged.push(current_block);
            current_block = block;
        }
    }

    // Don't forget the last block
    merged.push(current_block);

    merged
}

/// Find the currently active billing block
pub fn find_active_block(blocks: &[BillingBlock]) -> Option<&BillingBlock> {
    blocks.iter().find(|b| b.is_active)
}

/// Get blocks from the last N days
pub fn get_recent_blocks(blocks: &[BillingBlock], days: i64) -> Vec<&BillingBlock> {
    let cutoff = Utc::now() - Duration::days(days);
    blocks.iter().filter(|b| b.start_time >= cutoff).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_block_start() {
        let dt = DateTime::parse_from_rfc3339("2024-01-15T07:30:45Z")
            .unwrap()
            .with_timezone(&Utc);
        let block_start = get_block_start(dt);

        assert_eq!(block_start.hour(), 5);
        assert_eq!(block_start.minute(), 0);
        assert_eq!(block_start.second(), 0);

        // Test different hours
        let dt2 = DateTime::parse_from_rfc3339("2024-01-15T13:45:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let block_start2 = get_block_start(dt2);
        assert_eq!(block_start2.hour(), 10);

        let dt3 = DateTime::parse_from_rfc3339("2024-01-15T23:59:59Z")
            .unwrap()
            .with_timezone(&Utc);
        let block_start3 = get_block_start(dt3);
        assert_eq!(block_start3.hour(), 20);
    }

    #[test]
    fn test_identify_session_blocks() {
        let now = Utc::now();
        let entries = vec![
            UsageEntry {
                timestamp: now - Duration::hours(2),
                input_tokens: 100,
                output_tokens: 50,
                cache_creation_tokens: 0,
                cache_read_tokens: 0,
                model: "test".to_string(),
                cost: Some(1.0),
                session_id: "session1".to_string(),
            },
            UsageEntry {
                timestamp: now - Duration::hours(1),
                input_tokens: 200,
                output_tokens: 100,
                cache_creation_tokens: 0,
                cache_read_tokens: 0,
                model: "test".to_string(),
                cost: Some(2.0),
                session_id: "session1".to_string(),
            },
        ];

        let blocks = identify_session_blocks(&entries);
        assert!(!blocks.is_empty());

        let active_block = blocks.iter().find(|b| b.is_active);
        assert!(active_block.is_some());

        if let Some(block) = active_block {
            assert_eq!(block.total_tokens, 450); // 100+50+200+100
            assert_eq!(block.cost, 3.0); // 1.0+2.0
            assert_eq!(block.session_count, 1);
            assert!(block.remaining_minutes > 0);
        }
    }

    #[test]
    fn test_merge_consecutive_blocks() {
        let start1 = DateTime::parse_from_rfc3339("2024-01-15T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let start2 = start1 + Duration::hours(5);

        let blocks = vec![
            BillingBlock {
                start_time: start1,
                end_time: start1 + Duration::hours(5),
                cost: 10.0,
                remaining_minutes: 0,
                is_active: false,
                session_count: 1,
                total_tokens: 1000,
            },
            BillingBlock {
                start_time: start2,
                end_time: start2 + Duration::hours(5),
                cost: 20.0,
                remaining_minutes: 0,
                is_active: false,
                session_count: 1,
                total_tokens: 2000,
            },
        ];

        let merged = merge_consecutive_blocks(blocks);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].cost, 30.0);
        assert_eq!(merged[0].total_tokens, 3000);
        assert_eq!(merged[0].end_time, start1 + Duration::hours(10));
    }
}
