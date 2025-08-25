use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Main config structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub style: StyleConfig,
    pub segments: Vec<SegmentConfig>,
    pub theme: String,
    #[serde(default)]
    pub global: GlobalConfig,
}

// Default implementation moved to ui/themes/presets.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    #[serde(default = "default_context_limit")]
    pub context_limit: u32,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            context_limit: default_context_limit(),
        }
    }
}

impl GlobalConfig {
    /// Validate the global configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.context_limit == 0 {
            return Err("Context limit must be greater than 0".to_string());
        }
        Ok(())
    }
}

fn default_context_limit() -> u32 {
    200000
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleConfig {
    pub mode: StyleMode,
    pub separator: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StyleMode {
    Plain,     // emoji + 颜色
    NerdFont,  // Nerd Font 图标 + 颜色
    Powerline, // 未来支持
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentConfig {
    pub id: SegmentId,
    pub enabled: bool,
    pub icon: IconConfig,
    pub colors: ColorConfig,
    pub styles: TextStyleConfig,
    pub options: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconConfig {
    pub plain: String,
    pub nerd_font: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorConfig {
    pub icon: Option<AnsiColor>,
    pub text: Option<AnsiColor>,
    pub background: Option<AnsiColor>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TextStyleConfig {
    pub text_bold: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnsiColor {
    Color16 { c16: u8 },
    Color256 { c256: u8 },
    Rgb { r: u8, g: u8, b: u8 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SegmentId {
    Model,
    Directory,
    Git,
    Usage,
    Update,
    Cost,
    BurnRate,
}

// Cost source strategy for CostSegment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CostSource {
    #[default]
    Auto, // Prefer native cost from Claude Code, fallback to calculated
    Native,     // Only use native cost from Claude Code
    Calculated, // Always calculate from tokens
    Both,       // Show both native and calculated costs
}

// Legacy compatibility structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SegmentsConfig {
    pub directory: bool,
    pub git: bool,
    pub model: bool,
    pub usage: bool,
    #[serde(default = "default_true")]
    pub cost: bool,
    #[serde(default = "default_true")]
    pub burn_rate: bool,
}

fn default_true() -> bool {
    true
}

// Data structures compatible with existing main.rs
#[derive(Deserialize)]
pub struct Model {
    pub display_name: String,
}

#[derive(Deserialize)]
pub struct Workspace {
    pub current_dir: String,
}

#[derive(Deserialize)]
pub struct InputData {
    pub model: Model,
    pub workspace: Workspace,
    pub transcript_path: String,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub cost: Option<SessionCost>,
}

// Session cost information from Claude Code
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SessionCost {
    pub total_cost_usd: f64,
    #[serde(default)]
    pub total_duration_ms: Option<u64>,
    #[serde(default)]
    pub total_api_duration_ms: Option<u64>,
    #[serde(default)]
    pub total_lines_added: Option<u32>,
    #[serde(default)]
    pub total_lines_removed: Option<u32>,
}

// OpenAI-style nested token details
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct PromptTokensDetails {
    #[serde(default)]
    pub cached_tokens: Option<u32>,
    #[serde(default)]
    pub audio_tokens: Option<u32>,
}

// Raw usage data from different LLM providers (flexible parsing)
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct RawUsage {
    // Common input token naming variants
    #[serde(default, alias = "prompt_tokens")]
    pub input_tokens: Option<u32>,

    // Common output token naming variants
    #[serde(default, alias = "completion_tokens")]
    pub output_tokens: Option<u32>,

    // Total tokens (some providers only provide this)
    #[serde(default)]
    pub total_tokens: Option<u32>,

    // Anthropic-style cache fields
    #[serde(default, alias = "cache_creation_prompt_tokens")]
    pub cache_creation_input_tokens: Option<u32>,

    #[serde(default, alias = "cache_read_prompt_tokens")]
    pub cache_read_input_tokens: Option<u32>,

    // OpenAI-style nested details
    #[serde(default)]
    pub prompt_tokens_details: Option<PromptTokensDetails>,

    // Completion token details (OpenAI)
    #[serde(default)]
    pub completion_tokens_details: Option<HashMap<String, u32>>,

    // Catch unknown fields for future compatibility and debugging
    #[serde(flatten, skip_serializing)]
    pub extra: HashMap<String, serde_json::Value>,
}

// Normalized internal representation after processing
#[derive(Debug, Clone, Serialize, Default, PartialEq)]
pub struct NormalizedUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
    pub cache_creation_input_tokens: u32,
    pub cache_read_input_tokens: u32,

    // Metadata for debugging and analysis
    pub calculation_source: String,
    pub raw_data_available: Vec<String>,
}

impl NormalizedUsage {
    /// Get tokens that count toward context window
    /// This includes all tokens that consume context window space
    /// Output tokens from this turn will become input tokens in the next turn
    pub fn context_tokens(&self) -> u32 {
        self.input_tokens
            + self.cache_creation_input_tokens
            + self.cache_read_input_tokens
            + self.output_tokens
    }

    /// Get total tokens for cost calculation
    /// Priority: use total_tokens if available, otherwise sum all components
    pub fn total_for_cost(&self) -> u32 {
        if self.total_tokens > 0 {
            self.total_tokens
        } else {
            self.input_tokens
                + self.output_tokens
                + self.cache_creation_input_tokens
                + self.cache_read_input_tokens
        }
    }

    /// Get the most appropriate token count for general display
    /// For OpenAI format: use total_tokens directly
    /// For Anthropic format: use context_tokens (input + cache)
    pub fn display_tokens(&self) -> u32 {
        // For Claude/Anthropic format: prefer input-related tokens for context window display
        let context = self.context_tokens();
        if context > 0 {
            return context;
        }

        // For OpenAI format: use total_tokens when no input breakdown available
        if self.total_tokens > 0 {
            return self.total_tokens;
        }

        // Fallback to any available tokens
        self.input_tokens.max(self.output_tokens)
    }
}

impl Config {
    /// Check if current config matches the specified theme preset
    pub fn matches_theme(&self, theme_name: &str) -> bool {
        let theme_preset = crate::ui::themes::ThemePresets::get_theme(theme_name);

        // Compare style config
        if self.style.mode != theme_preset.style.mode
            || self.style.separator != theme_preset.style.separator
        {
            return false;
        }

        // Compare segments count and order
        if self.segments.len() != theme_preset.segments.len() {
            return false;
        }

        // Compare each segment config
        for (current, preset) in self.segments.iter().zip(theme_preset.segments.iter()) {
            if !self.segment_matches(current, preset) {
                return false;
            }
        }

        true
    }

    /// Check if current config has been modified from the selected theme
    pub fn is_modified_from_theme(&self) -> bool {
        !self.matches_theme(&self.theme)
    }

    /// Compare two segment configs for equality
    fn segment_matches(&self, current: &SegmentConfig, preset: &SegmentConfig) -> bool {
        current.id == preset.id
            && current.enabled == preset.enabled
            && current.icon.plain == preset.icon.plain
            && current.icon.nerd_font == preset.icon.nerd_font
            && self.color_matches(&current.colors.icon, &preset.colors.icon)
            && self.color_matches(&current.colors.text, &preset.colors.text)
            && self.color_matches(&current.colors.background, &preset.colors.background)
            && current.styles.text_bold == preset.styles.text_bold
            && current.options == preset.options
    }

    /// Compare two optional colors for equality
    fn color_matches(&self, current: &Option<AnsiColor>, preset: &Option<AnsiColor>) -> bool {
        match (current, preset) {
            (None, None) => true,
            (Some(c1), Some(c2)) => c1 == c2,
            _ => false,
        }
    }
}

impl PartialEq for AnsiColor {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (AnsiColor::Color16 { c16: a }, AnsiColor::Color16 { c16: b }) => a == b,
            (AnsiColor::Color256 { c256: a }, AnsiColor::Color256 { c256: b }) => a == b,
            (
                AnsiColor::Rgb {
                    r: r1,
                    g: g1,
                    b: b1,
                },
                AnsiColor::Rgb {
                    r: r2,
                    g: g2,
                    b: b2,
                },
            ) => r1 == r2 && g1 == g2 && b1 == b2,
            _ => false,
        }
    }
}

impl RawUsage {
    /// Convert raw usage data to normalized format with intelligent token inference
    pub fn normalize(self) -> NormalizedUsage {
        let mut result = NormalizedUsage::default();
        let mut sources = Vec::new();

        // Collect available raw data fields
        let mut available_fields = Vec::new();
        if self.input_tokens.is_some() {
            available_fields.push("input_tokens".to_string());
        }
        if self.output_tokens.is_some() {
            available_fields.push("output_tokens".to_string());
        }
        if self.total_tokens.is_some() {
            available_fields.push("total_tokens".to_string());
        }
        if self.cache_creation_input_tokens.is_some() {
            available_fields.push("cache_creation".to_string());
        }
        if self.cache_read_input_tokens.is_some() {
            available_fields.push("cache_read".to_string());
        }

        result.raw_data_available = available_fields;

        // Extract directly available values
        let input = self.input_tokens.unwrap_or(0);
        let output = self.output_tokens.unwrap_or(0);
        let total = self.total_tokens.unwrap_or(0);

        // Handle cache tokens with fallback to OpenAI nested format
        let cache_read = self
            .cache_read_input_tokens
            .or_else(|| {
                self.prompt_tokens_details
                    .as_ref()
                    .and_then(|d| d.cached_tokens)
            })
            .unwrap_or(0);

        let cache_creation = self.cache_creation_input_tokens.unwrap_or(0);

        // Token calculation logic - prioritize total_tokens for OpenAI format
        let total_value = if total > 0 {
            sources.push("total_tokens_direct".to_string());
            total
        } else if input > 0 || output > 0 || cache_read > 0 || cache_creation > 0 {
            let calculated = input + output + cache_read + cache_creation;
            sources.push("total_from_components".to_string());
            calculated
        } else {
            0
        };

        // Assignment
        result.input_tokens = input;
        result.output_tokens = output;
        result.total_tokens = total_value;
        result.cache_creation_input_tokens = cache_creation;
        result.cache_read_input_tokens = cache_read;
        result.calculation_source = sources.join("+");

        result
    }
}

// Legacy alias for backward compatibility
pub type Usage = RawUsage;

#[derive(Deserialize)]
pub struct Message {
    #[serde(default)]
    pub id: Option<String>,
    pub usage: Option<Usage>,
    pub model: Option<String>,
}

#[derive(Deserialize)]
pub struct TranscriptEntry {
    pub r#type: Option<String>,
    pub message: Option<Message>,
    #[serde(default, alias = "requestId")]
    pub request_id: Option<String>,
    #[serde(default)]
    pub timestamp: Option<String>,
    #[serde(default, alias = "costUSD")]
    pub cost_usd: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_config_default() {
        let config = GlobalConfig::default();
        assert_eq!(config.context_limit, 200000);
    }

    #[test]
    fn test_global_config_validate_valid() {
        let config = GlobalConfig {
            context_limit: 100000,
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_global_config_validate_zero() {
        let config = GlobalConfig { context_limit: 0 };
        assert!(config.validate().is_err());
        assert_eq!(
            config.validate().unwrap_err(),
            "Context limit must be greater than 0"
        );
    }

    #[test]
    fn test_global_config_validate_small_value() {
        // Even 1 is valid, we only check for 0
        let config = GlobalConfig { context_limit: 1 };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_global_config_validate_large_value() {
        let config = GlobalConfig {
            context_limit: u32::MAX,
        };
        assert!(config.validate().is_ok());
    }
}
