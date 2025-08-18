use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;

use super::ModelPricing;

/// LiteLLM's model pricing and context window data URL
const LITELLM_PRICING_URL: &str =
    "https://raw.githubusercontent.com/BerriAI/litellm/main/model_prices_and_context_window.json";

/// Memory cache TTL in seconds (5 minutes)
const MEMORY_CACHE_TTL_SECONDS: i64 = 300;

/// File cache TTL in seconds (24 hours)
const FILE_CACHE_TTL_SECONDS: i64 = 86400;

/// Pricing cache file path
fn get_cache_file_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
        .join("ccline")
        .join("pricing_cache.json")
}

/// Cached pricing data with timestamp (for memory cache)
struct CachedPricing {
    data: HashMap<String, ModelPricing>,
    fetched_at: DateTime<Utc>,
}

impl CachedPricing {
    fn is_expired(&self) -> bool {
        let age = Utc::now() - self.fetched_at;
        age.num_seconds() > MEMORY_CACHE_TTL_SECONDS
    }
}

/// File cache structure with metadata
#[derive(Debug, Serialize, Deserialize)]
struct FileCachePricing {
    fetched_at: DateTime<Utc>,
    ttl_hours: u32,
    data: HashMap<String, ModelPricing>,
}

impl FileCachePricing {
    fn is_expired(&self) -> bool {
        let age = Utc::now() - self.fetched_at;
        age.num_seconds() > FILE_CACHE_TTL_SECONDS
    }

    /// Load pricing data from file cache
    fn load_from_file() -> Option<Self> {
        let cache_path = get_cache_file_path();
        if !cache_path.exists() {
            return None;
        }

        let content = fs::read_to_string(&cache_path).ok()?;
        let cache: FileCachePricing = serde_json::from_str(&content).ok()?;

        if cache.is_expired() {
            return None;
        }

        Some(cache)
    }

    /// Save pricing data to file cache
    fn save_to_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        let cache_path = get_cache_file_path();

        // Ensure parent directory exists
        if let Some(parent) = cache_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(&cache_path, content)?;

        Ok(())
    }
}

/// Pricing data cache with TTL
static PRICING_CACHE: Lazy<RwLock<Option<CachedPricing>>> = Lazy::new(|| RwLock::new(None));

/// LiteLLM data format
#[derive(Debug, Clone, Deserialize)]
pub struct LiteLLMPricing {
    // Make these optional to handle non-text models (e.g., image generation)
    pub input_cost_per_token: Option<f64>,
    pub output_cost_per_token: Option<f64>,
    #[serde(default)]
    pub cache_creation_input_token_cost: Option<f64>,
    #[serde(default)]
    pub cache_read_input_token_cost: Option<f64>,
}

impl ModelPricing {
    /// Fetch pricing data with three-tier caching (memory -> file -> network)
    pub async fn fetch_pricing() -> Result<HashMap<String, ModelPricing>, Box<dyn std::error::Error>>
    {
        // Tier 1: Check memory cache first
        if let Some(cached) = PRICING_CACHE.read().unwrap().as_ref() {
            if !cached.is_expired() {
                return Ok(cached.data.clone());
            }
        }

        // Tier 2: Check file cache
        if let Some(file_cache) = FileCachePricing::load_from_file() {
            // Update memory cache from file
            let pricing = file_cache.data.clone();
            *PRICING_CACHE.write().unwrap() = Some(CachedPricing {
                data: pricing.clone(),
                fetched_at: file_cache.fetched_at,
            });
            return Ok(pricing);
        }

        // Tier 3: Fetch from network
        let response = reqwest::get(LITELLM_PRICING_URL).await?;
        let data: HashMap<String, LiteLLMPricing> = response.json().await?;

        // Convert to internal format, only keep Claude models with valid pricing
        let mut pricing = HashMap::new();
        let mut total_models = 0;
        let mut claude_models = 0;
        let mut valid_claude_models = 0;

        for (model_name, litellm_pricing) in data {
            total_models += 1;

            // Check if it's a Claude model
            if model_name.starts_with("claude-") || model_name.contains("claude") {
                claude_models += 1;

                // Only process models with valid token pricing (skip image generation models etc.)
                if let (Some(input_cost), Some(output_cost)) = (
                    litellm_pricing.input_cost_per_token,
                    litellm_pricing.output_cost_per_token,
                ) {
                    valid_claude_models += 1;
                    pricing.insert(
                        model_name.clone(),
                        ModelPricing {
                            model_name,
                            // Convert to cost per 1k tokens
                            input_cost_per_1k: input_cost * 1000.0,
                            output_cost_per_1k: output_cost * 1000.0,
                            cache_creation_cost_per_1k: litellm_pricing
                                .cache_creation_input_token_cost
                                .map(|c| c * 1000.0)
                                .unwrap_or(0.0),
                            cache_read_cost_per_1k: litellm_pricing
                                .cache_read_input_token_cost
                                .map(|c| c * 1000.0)
                                .unwrap_or(0.0),
                        },
                    );
                }
            }
        }

        // Only show debug info if CCLINE_DEBUG is set
        if std::env::var("CCLINE_DEBUG").is_ok() {
            eprintln!(
                "LiteLLM: Fetched {} total models, {} Claude models, {} with valid pricing",
                total_models, claude_models, valid_claude_models
            );
        }

        let now = Utc::now();

        // Save to file cache
        let file_cache = FileCachePricing {
            fetched_at: now,
            ttl_hours: 24,
            data: pricing.clone(),
        };

        if let Err(e) = file_cache.save_to_file() {
            eprintln!("Warning: Failed to save pricing cache to file: {}", e);
        }

        // Update memory cache
        *PRICING_CACHE.write().unwrap() = Some(CachedPricing {
            data: pricing.clone(),
            fetched_at: now,
        });

        Ok(pricing)
    }

    /// Get pricing with fallback
    pub async fn get_pricing_with_fallback() -> HashMap<String, ModelPricing> {
        match Self::fetch_pricing().await {
            Ok(pricing) => pricing,
            Err(e) => {
                eprintln!("Failed to fetch pricing from LiteLLM: {}", e);
                eprintln!("Using fallback pricing data");
                Self::fallback_pricing()
            }
        }
    }

    /// Fallback pricing data for offline use
    fn fallback_pricing() -> HashMap<String, ModelPricing> {
        let mut m = HashMap::new();

        // Claude 4 models (corrected per-token pricing)
        m.insert(
            "claude-sonnet-4-20250514".to_string(),
            ModelPricing {
                model_name: "claude-sonnet-4-20250514".to_string(),
                input_cost_per_1k: 0.003,  // $0.003/1k tokens = $3/1M tokens
                output_cost_per_1k: 0.015, // $0.015/1k tokens = $15/1M tokens
                cache_creation_cost_per_1k: 0.00375, // $0.00375/1k tokens = $3.75/1M tokens
                cache_read_cost_per_1k: 0.0003, // $0.0003/1k tokens = $0.30/1M tokens
            },
        );

        m.insert(
            "claude-opus-4-1-20250805".to_string(),
            ModelPricing {
                model_name: "claude-opus-4-1-20250805".to_string(),
                input_cost_per_1k: 0.015, // $0.015/1k tokens = $15/1M tokens
                output_cost_per_1k: 0.075, // $0.075/1k tokens = $75/1M tokens
                cache_creation_cost_per_1k: 0.01875, // $0.01875/1k tokens = $18.75/1M tokens
                cache_read_cost_per_1k: 0.0015, // $0.0015/1k tokens = $1.5/1M tokens
            },
        );

        m.insert(
            "claude-opus-4-1".to_string(),
            ModelPricing {
                model_name: "claude-opus-4-1".to_string(),
                input_cost_per_1k: 0.015, // $0.015/1k tokens = $15/1M tokens
                output_cost_per_1k: 0.075, // $0.075/1k tokens = $75/1M tokens
                cache_creation_cost_per_1k: 0.01875, // $0.01875/1k tokens = $18.75/1M tokens
                cache_read_cost_per_1k: 0.0015, // $0.0015/1k tokens = $1.5/1M tokens
            },
        );

        // Claude 3.5 models (corrected per-token pricing)
        m.insert(
            "claude-3-5-sonnet-20241022".to_string(),
            ModelPricing {
                model_name: "claude-3-5-sonnet-20241022".to_string(),
                input_cost_per_1k: 0.003,  // $0.003/1k tokens = $3/1M tokens
                output_cost_per_1k: 0.015, // $0.015/1k tokens = $15/1M tokens
                cache_creation_cost_per_1k: 0.00375, // $0.00375/1k tokens = $3.75/1M tokens
                cache_read_cost_per_1k: 0.0003, // $0.0003/1k tokens = $0.30/1M tokens
            },
        );

        m.insert(
            "claude-3-5-sonnet".to_string(),
            ModelPricing {
                model_name: "claude-3-5-sonnet".to_string(),
                input_cost_per_1k: 0.003,  // $0.003/1k tokens = $3/1M tokens
                output_cost_per_1k: 0.015, // $0.015/1k tokens = $15/1M tokens
                cache_creation_cost_per_1k: 0.00375, // $0.00375/1k tokens = $3.75/1M tokens
                cache_read_cost_per_1k: 0.0003, // $0.0003/1k tokens = $0.30/1M tokens
            },
        );

        // Claude 3 models (corrected per-token pricing)
        m.insert(
            "claude-3-opus-20240229".to_string(),
            ModelPricing {
                model_name: "claude-3-opus-20240229".to_string(),
                input_cost_per_1k: 0.015, // $0.015/1k tokens = $15/1M tokens
                output_cost_per_1k: 0.075, // $0.075/1k tokens = $75/1M tokens
                cache_creation_cost_per_1k: 0.01875, // $0.01875/1k tokens = $18.75/1M tokens
                cache_read_cost_per_1k: 0.0015, // $0.0015/1k tokens = $1.50/1M tokens
            },
        );

        m.insert(
            "claude-3-5-haiku-20241022".to_string(),
            ModelPricing {
                model_name: "claude-3-5-haiku-20241022".to_string(),
                input_cost_per_1k: 0.0008, // $0.0008/1k tokens = $0.80/1M tokens
                output_cost_per_1k: 0.004, // $0.004/1k tokens = $4/1M tokens
                cache_creation_cost_per_1k: 0.001, // $0.001/1k tokens = $1/1M tokens
                cache_read_cost_per_1k: 0.00008, // $0.00008/1k tokens = $0.08/1M tokens
            },
        );

        m
    }

    /// Get pricing for a specific model with fuzzy matching
    pub fn get_model_pricing<'a>(
        pricing_map: &'a HashMap<String, ModelPricing>,
        model_name: &str,
    ) -> Option<&'a ModelPricing> {
        // Try exact match first
        if let Some(pricing) = pricing_map.get(model_name) {
            return Some(pricing);
        }

        // Try fuzzy matching
        let model_lower = model_name.to_lowercase();

        // Look for the most specific match
        pricing_map
            .iter()
            .filter(|(key, _)| {
                let key_lower = key.to_lowercase();
                model_lower.contains(&key_lower) || key_lower.contains(&model_lower)
            })
            .max_by_key(|(key, _)| key.len()) // Prefer longer (more specific) matches
            .map(|(_, pricing)| pricing)
    }
}

/// Clear the pricing cache (useful for testing)
pub fn clear_pricing_cache() {
    *PRICING_CACHE.write().unwrap() = None;
}
