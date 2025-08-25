use super::{Segment, SegmentData};
use crate::billing::{
    block::{find_active_block, identify_session_blocks_with_overrides},
    calculator::{calculate_daily_total, calculate_session_cost, format_remaining_time},
    ModelPricing,
};
use crate::config::{CostSource, InputData, SegmentConfig, SegmentId};
use crate::utils::{
    data_loader::DataLoader, data_loader_fast::FastDataLoader, transcript::extract_session_id,
};
use std::collections::HashMap;
use std::time::Instant;

pub struct CostSegment {
    enabled: bool,
    show_timing: bool,
    use_fast_loader: bool,
    thread_multiplier: Option<f64>,
    cost_source: CostSource,
}

impl CostSegment {
    pub fn new(config: &SegmentConfig) -> Self {
        let cost_source = config
            .options
            .get("cost_source")
            .and_then(|v| v.as_str())
            .and_then(|s| match s {
                "auto" => Some(CostSource::Auto),
                "native" => Some(CostSource::Native),
                "calculated" => Some(CostSource::Calculated),
                "both" => Some(CostSource::Both),
                _ => None,
            })
            .unwrap_or_default();

        Self {
            enabled: config.enabled,
            show_timing: config
                .options
                .get("show_timing")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            use_fast_loader: config
                .options
                .get("fast_loader")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            thread_multiplier: config
                .options
                .get("thread_multiplier")
                .and_then(|v| v.as_f64()),
            cost_source,
        }
    }

    fn collect_with_pricing(&self, input: &InputData) -> SegmentData {
        // Performance timing
        let start = Instant::now();
        let mut timings = Vec::new();

        // Get native cost if available
        let native_cost = input.cost.as_ref().map(|c| c.total_cost_usd);

        // 1. Always load all project data
        let load_start = Instant::now();
        let mut all_entries = if self.use_fast_loader {
            // Use optimized fast loader with optional thread multiplier
            let mut fast_loader = if let Some(multiplier) = self.thread_multiplier {
                FastDataLoader::with_thread_multiplier(multiplier)
            } else {
                FastDataLoader::new()
            };
            fast_loader.load_all_projects()
        } else {
            // Use original loader
            let mut data_loader = DataLoader::new();
            data_loader.load_all_projects()
        };
        timings.push(("L", load_start.elapsed().as_millis()));

        // 2. Get pricing data (use global runtime to handle async)
        let pricing_start = Instant::now();
        let pricing_map =
            crate::utils::block_on(async { ModelPricing::get_pricing_with_fallback().await });
        timings.push(("P", pricing_start.elapsed().as_millis()));

        // 3. Calculate costs for all entries
        let calc_start = Instant::now();
        for entry in &mut all_entries {
            if let Some(pricing) = ModelPricing::get_model_pricing(&pricing_map, &entry.model) {
                entry.cost = Some(pricing.calculate_cost(entry));
            }
        }
        timings.push(("C", calc_start.elapsed().as_millis()));

        // 4. Calculate session and daily costs
        let analyze_start = Instant::now();
        let transcript_path = std::path::Path::new(&input.transcript_path);
        let session_id = extract_session_id(transcript_path);
        let calculated_session_cost =
            calculate_session_cost(&all_entries, &session_id, &pricing_map);
        let daily_total = calculate_daily_total(&all_entries, &pricing_map);
        timings.push(("A", analyze_start.elapsed().as_millis()));

        // Determine which session cost to use based on strategy
        let session_cost = match self.cost_source {
            CostSource::Auto => native_cost.unwrap_or(calculated_session_cost),
            CostSource::Native => native_cost.unwrap_or(0.0),
            CostSource::Calculated => calculated_session_cost,
            CostSource::Both => calculated_session_cost, // Will show both in display
        };

        // 5. Calculate dynamic blocks with override support
        let block_start = Instant::now();
        let blocks = identify_session_blocks_with_overrides(&all_entries);
        let active_block = find_active_block(&blocks);
        timings.push(("B", block_start.elapsed().as_millis()));

        // Build metadata
        let mut metadata = HashMap::new();
        metadata.insert("session_cost".to_string(), format!("{:.2}", session_cost));
        metadata.insert("daily_total".to_string(), format!("{:.2}", daily_total));

        if let Some(block) = &active_block {
            metadata.insert("block_cost".to_string(), format!("{:.2}", block.cost));
            metadata.insert(
                "block_remaining".to_string(),
                format!("{}", block.remaining_minutes),
            );
        }

        // Format primary and secondary text based on cost source
        let primary = match self.cost_source {
            CostSource::Both if native_cost.is_some() => {
                format!(
                    "${:.2} native / ${:.2} calc",
                    native_cost.unwrap(),
                    calculated_session_cost
                )
            }
            _ => format!("${:.2} session", session_cost),
        };

        let secondary = if let Some(block) = active_block {
            format!(
                "${:.2} today · ${:.2} block ({})",
                daily_total,
                block.cost,
                format_remaining_time(block.remaining_minutes)
            )
        } else {
            format!("${:.2} today · No active block", daily_total)
        };

        // Add performance timing to secondary if enabled
        let secondary_with_timing = if self.show_timing {
            let total_ms = start.elapsed().as_millis();
            let timing_str = format!(
                " [{}ms: L{}|P{}|C{}|A{}|B{}]",
                total_ms,
                timings[0].1, // Load
                timings[1].1, // Pricing
                timings[2].1, // Calculate
                timings[3].1, // Analyze
                timings[4].1  // Block
            );
            format!("{}{}", secondary, timing_str)
        } else {
            secondary
        };

        SegmentData {
            primary,
            secondary: secondary_with_timing,
            metadata,
        }
    }
}

impl Segment for CostSegment {
    fn collect(&self, input: &InputData) -> Option<SegmentData> {
        // Fast path: skip all expensive operations when disabled
        if !self.enabled {
            return None;
        }

        // Handle potential errors gracefully
        match std::panic::catch_unwind(|| self.collect_with_pricing(input)) {
            Ok(result) => Some(result),
            Err(_) => {
                // Fallback display on error
                let mut metadata = HashMap::new();
                metadata.insert("error".to_string(), "true".to_string());

                Some(SegmentData {
                    primary: "$0.00 session".to_string(),
                    secondary: "$0.00 today · Error loading data".to_string(),
                    metadata,
                })
            }
        }
    }

    fn id(&self) -> SegmentId {
        SegmentId::Cost
    }
}
