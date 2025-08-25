use super::{Segment, SegmentData};
use crate::config::{GlobalConfig, InputData, SegmentId, TranscriptEntry};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct UsageSegment {
    context_limit: u32,
}

impl UsageSegment {
    pub fn new(global_config: &GlobalConfig) -> Self {
        Self {
            context_limit: global_config.context_limit,
        }
    }
}

impl Segment for UsageSegment {
    fn collect(&self, input: &InputData) -> Option<SegmentData> {
        let context_used_token = if input.transcript_path == "mock_preview" {
            // Hardcoded mock data for preview
            150000
        } else {
            parse_transcript_usage(&input.transcript_path)
        };

        // Safe division to prevent panic on zero
        let context_used_rate = if self.context_limit > 0 {
            (context_used_token as f64 / self.context_limit as f64) * 100.0
        } else {
            0.0
        };

        let percentage_display = if context_used_rate.fract() == 0.0 {
            format!("{:.0}%", context_used_rate)
        } else {
            format!("{:.1}%", context_used_rate)
        };

        let tokens_display = if context_used_token >= 1000 {
            let k_value = context_used_token as f64 / 1000.0;
            if k_value.fract() == 0.0 {
                format!("{}k", k_value as u32)
            } else {
                format!("{:.1}k", k_value)
            }
        } else {
            context_used_token.to_string()
        };

        let mut metadata = HashMap::new();
        metadata.insert("tokens".to_string(), context_used_token.to_string());
        metadata.insert("percentage".to_string(), context_used_rate.to_string());
        metadata.insert("limit".to_string(), self.context_limit.to_string());

        Some(SegmentData {
            primary: format!("{} · {} tokens", percentage_display, tokens_display),
            secondary: String::new(),
            metadata,
        })
    }

    fn id(&self) -> SegmentId {
        SegmentId::Usage
    }
}

fn parse_transcript_usage<P: AsRef<Path>>(transcript_path: P) -> u32 {
    let file = match fs::File::open(&transcript_path) {
        Ok(file) => file,
        Err(_) => return 0,
    };

    let reader = BufReader::new(file);
    let lines: Vec<String> = reader
        .lines()
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_default();

    for line in lines.iter().rev() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Ok(entry) = serde_json::from_str::<TranscriptEntry>(line) {
            if entry.r#type.as_deref() == Some("assistant") {
                if let Some(message) = &entry.message {
                    if let Some(raw_usage) = &message.usage {
                        let normalized = raw_usage.clone().normalize();
                        return normalized.display_tokens();
                    }
                }
            }
        }
    }

    0
}
