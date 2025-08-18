use super::{Segment, SegmentData};
use crate::billing::{
    block::{find_active_block, identify_session_blocks_with_overrides},
    calculator::calculate_burn_rate,
    BurnRateThresholds, ModelPricing,
};
use crate::config::{InputData, SegmentId};
use crate::utils::data_loader::DataLoader;
use std::collections::HashMap;

pub struct BurnRateSegment {
    enabled: bool,
    thresholds: BurnRateThresholds,
}

impl BurnRateSegment {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            thresholds: BurnRateThresholds::from_env(),
        }
    }

    fn get_indicator(&self, tokens_per_minute: f64) -> &'static str {
        if tokens_per_minute > self.thresholds.high {
            "\u{ef76}" // 🔥 Fire (Nerd Font)
        } else if tokens_per_minute > self.thresholds.medium {
            "\u{f0e7}" // ⚡ Lightning bolt (Nerd Font)
        } else {
            "\u{f0e4}" // 📊 Dashboard/gauge (Nerd Font)
        }
    }

    fn collect_with_data(&self, _input: &InputData) -> SegmentData {
        // Load all project data globally (like ccusage does)
        let mut data_loader = DataLoader::new();
        let mut all_entries = data_loader.load_all_projects();

        // Get pricing data (use global runtime to handle async)
        let pricing_map =
            crate::utils::block_on(async { ModelPricing::get_pricing_with_fallback().await });

        // Calculate costs for entries
        for entry in &mut all_entries {
            if let Some(pricing) = ModelPricing::get_model_pricing(&pricing_map, &entry.model) {
                entry.cost = Some(pricing.calculate_cost(entry));
            }
        }

        // Find active billing block using dynamic calculation
        let blocks = identify_session_blocks_with_overrides(&all_entries);
        let active_block = find_active_block(&blocks);

        // Calculate burn rate
        let mut metadata = HashMap::new();

        let (primary, secondary) =
            match active_block.and_then(|block| calculate_burn_rate(block, &all_entries)) {
                Some(rate) => {
                    let indicator = self.get_indicator(rate.tokens_per_minute_for_indicator);
                    metadata.insert(
                        "cost_per_hour".to_string(),
                        format!("{:.2}", rate.cost_per_hour),
                    );
                    metadata.insert(
                        "tokens_per_minute".to_string(),
                        format!("{:.1}", rate.tokens_per_minute_for_indicator),
                    );
                    metadata.insert("trend".to_string(), format!("{:?}", rate.trend));

                    (
                        format!("${:.2}/hr", rate.cost_per_hour),
                        indicator.to_string(),
                    )
                }
                None => {
                    metadata.insert("status".to_string(), "no_data".to_string());
                    ("—/hr".to_string(), "\u{f0e4}".to_string())
                }
            };

        SegmentData {
            primary,
            secondary,
            metadata,
        }
    }
}

impl Segment for BurnRateSegment {
    fn collect(&self, input: &InputData) -> Option<SegmentData> {
        if !self.enabled {
            return None;
        }

        // Handle potential errors gracefully
        match std::panic::catch_unwind(|| self.collect_with_data(input)) {
            Ok(result) => Some(result),
            Err(_) => {
                let mut metadata = HashMap::new();
                metadata.insert("error".to_string(), "true".to_string());

                Some(SegmentData {
                    primary: "—/hr".to_string(),
                    secondary: "\u{f0e4}".to_string(),
                    metadata,
                })
            }
        }
    }

    fn id(&self) -> SegmentId {
        SegmentId::BurnRate
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Model, Workspace};

    #[test]
    fn test_burn_rate_segment_disabled() {
        let segment = BurnRateSegment::new(false);
        let input = InputData {
            model: Model {
                display_name: "test-model".to_string(),
            },
            workspace: Workspace {
                current_dir: "/test".to_string(),
            },
            transcript_path: "/test/transcript.jsonl".to_string(),
        };

        assert!(segment.collect(&input).is_none());
    }

    #[test]
    fn test_burn_rate_segment_enabled() {
        let segment = BurnRateSegment::new(true);
        let input = InputData {
            model: Model {
                display_name: "test-model".to_string(),
            },
            workspace: Workspace {
                current_dir: "/test".to_string(),
            },
            transcript_path: "/test/transcript.jsonl".to_string(),
        };

        // Should return Some data when enabled
        assert!(segment.collect(&input).is_some());
    }

    #[test]
    fn test_indicator_selection() {
        let segment = BurnRateSegment::new(true);

        // Test high burn rate
        assert_eq!(segment.get_indicator(6000.0), "\u{ef76}"); // Fire

        // Test medium burn rate
        assert_eq!(segment.get_indicator(3000.0), "\u{f0e7}"); // Lightning

        // Test normal burn rate
        assert_eq!(segment.get_indicator(1000.0), "\u{f0e4}"); // Dashboard
    }
}
