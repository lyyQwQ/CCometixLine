use super::types::Config;
use std::fs;
use std::path::{Path, PathBuf};

pub struct ConfigLoader;

impl ConfigLoader {
    pub fn load() -> Config {
        Config::load().unwrap_or_else(|_| Config::default())
    }

    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Config, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Initialize themes directory and create built-in theme files
    pub fn init_themes() -> Result<(), Box<dyn std::error::Error>> {
        let themes_dir = Self::get_themes_path();

        // Create themes directory
        fs::create_dir_all(&themes_dir)?;

        let builtin_themes = [
            "default",
            "minimal",
            "gruvbox",
            "nord",
            "powerline-dark",
            "powerline-light",
            "powerline-rose-pine",
            "powerline-tokyo-night",
        ];
        let mut created_any = false;

        for theme_name in &builtin_themes {
            let theme_path = themes_dir.join(format!("{}.toml", theme_name));

            if !theme_path.exists() {
                let theme_config = crate::ui::themes::ThemePresets::get_theme(theme_name);
                let content = toml::to_string_pretty(&theme_config)?;
                fs::write(&theme_path, content)?;
                println!("Created theme file: {}", theme_path.display());
                created_any = true;
            }
        }

        if !created_any {
            // println!("All built-in theme files already exist");
        }

        Ok(())
    }

    /// Get the themes directory path (~/.claude/ccline/themes/)
    pub fn get_themes_path() -> PathBuf {
        if let Some(home) = dirs::home_dir() {
            home.join(".claude").join("ccline").join("themes")
        } else {
            PathBuf::from(".claude/ccline/themes")
        }
    }

    /// Ensure themes directory exists and has built-in themes (silent mode)
    pub fn ensure_themes_exist() {
        // Silently ensure themes exist without printing output
        let _ = Self::init_themes_silent();
        // Migrate existing theme files if needed
        let _ = Self::migrate_all_themes();
    }

    /// Initialize themes directory and create built-in theme files (silent mode)
    fn init_themes_silent() -> Result<(), Box<dyn std::error::Error>> {
        let themes_dir = Self::get_themes_path();

        // Create themes directory
        fs::create_dir_all(&themes_dir)?;

        let builtin_themes = [
            "default",
            "minimal",
            "gruvbox",
            "nord",
            "powerline-dark",
            "powerline-light",
            "powerline-rose-pine",
            "powerline-tokyo-night",
        ];

        for theme_name in &builtin_themes {
            let theme_path = themes_dir.join(format!("{}.toml", theme_name));

            if !theme_path.exists() {
                let theme_config = crate::ui::themes::ThemePresets::get_theme(theme_name);
                let content = toml::to_string_pretty(&theme_config)?;
                fs::write(&theme_path, content)?;
            }
        }

        Ok(())
    }

    /// Migrate theme file if it's missing new segments
    pub fn migrate_theme_if_needed(theme_path: &Path) -> Result<bool, Box<dyn std::error::Error>> {
        if !theme_path.exists() {
            return Ok(false);
        }

        let content = fs::read_to_string(theme_path)?;
        let mut config: Config = toml::from_str(&content)?;

        // Check if Cost and BurnRate segments exist
        let has_cost = config
            .segments
            .iter()
            .any(|s| s.id == crate::config::SegmentId::Cost);
        let has_burn_rate = config
            .segments
            .iter()
            .any(|s| s.id == crate::config::SegmentId::BurnRate);

        if has_cost && has_burn_rate {
            return Ok(false); // No migration needed
        }

        // Get the theme name from the file name
        let theme_name = theme_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("default");

        // Get the complete theme configuration from presets
        let complete_theme = crate::ui::themes::ThemePresets::get_theme(theme_name);

        // Add missing segments
        if !has_cost {
            if let Some(cost_segment) = complete_theme
                .segments
                .iter()
                .find(|s| s.id == crate::config::SegmentId::Cost)
            {
                config.segments.push(cost_segment.clone());
            }
        }

        if !has_burn_rate {
            if let Some(burn_rate_segment) = complete_theme
                .segments
                .iter()
                .find(|s| s.id == crate::config::SegmentId::BurnRate)
            {
                config.segments.push(burn_rate_segment.clone());
            }
        }

        // Save the migrated configuration
        let content = toml::to_string_pretty(&config)?;
        fs::write(theme_path, content)?;

        Ok(true) // Migration performed
    }

    /// Migrate all theme files in the themes directory
    pub fn migrate_all_themes() -> Result<u32, Box<dyn std::error::Error>> {
        let themes_dir = Self::get_themes_path();
        let mut migrated_count = 0;

        if let Ok(entries) = fs::read_dir(&themes_dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".toml") {
                        let theme_path = entry.path();
                        if Self::migrate_theme_if_needed(&theme_path)? {
                            migrated_count += 1;
                        }
                    }
                }
            }
        }

        Ok(migrated_count)
    }
}

impl Config {
    /// Load configuration from default location
    pub fn load() -> Result<Config, Box<dyn std::error::Error>> {
        // Ensure themes directory exists and has built-in themes
        ConfigLoader::ensure_themes_exist();

        let config_path = Self::get_config_path();

        if !config_path.exists() {
            return Ok(Config::default());
        }

        let content = fs::read_to_string(config_path)?;
        let mut config: Config = toml::from_str(&content)?;

        // Migrate config if needed
        if Self::migrate_config_if_needed(&mut config)? {
            // Save the migrated config
            config.save()?;
        }

        Ok(config)
    }

    /// Migrate config to add missing segments
    fn migrate_config_if_needed(config: &mut Config) -> Result<bool, Box<dyn std::error::Error>> {
        // Check if Cost and BurnRate segments exist
        let has_cost = config
            .segments
            .iter()
            .any(|s| s.id == crate::config::SegmentId::Cost);
        let has_burn_rate = config
            .segments
            .iter()
            .any(|s| s.id == crate::config::SegmentId::BurnRate);

        if has_cost && has_burn_rate {
            return Ok(false); // No migration needed
        }

        // Get the default theme configuration to get the missing segments
        let default_config = crate::ui::themes::ThemePresets::get_default();

        // Add missing segments
        if !has_cost {
            if let Some(cost_segment) = default_config
                .segments
                .iter()
                .find(|s| s.id == crate::config::SegmentId::Cost)
            {
                config.segments.push(cost_segment.clone());
            }
        }

        if !has_burn_rate {
            if let Some(burn_rate_segment) = default_config
                .segments
                .iter()
                .find(|s| s.id == crate::config::SegmentId::BurnRate)
            {
                config.segments.push(burn_rate_segment.clone());
            }
        }

        Ok(!has_cost || !has_burn_rate) // Return true if migration was performed
    }

    /// Save configuration to default location
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path();

        // Ensure config directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        fs::write(config_path, content)?;
        Ok(())
    }

    /// Get the default config file path (~/.claude/ccline/config.toml)
    fn get_config_path() -> PathBuf {
        if let Some(home) = dirs::home_dir() {
            home.join(".claude").join("ccline").join("config.toml")
        } else {
            PathBuf::from(".claude/ccline/config.toml")
        }
    }

    /// Initialize config directory and create default config
    pub fn init() -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path();

        // Create directory
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Initialize themes directory and built-in themes
        ConfigLoader::init_themes()?;

        // Create default config if it doesn't exist
        if !config_path.exists() {
            let default_config = Config::default();
            default_config.save()?;
            println!("Created config at {}", config_path.display());
        } else {
            println!("Config already exists at {}", config_path.display());
        }

        Ok(())
    }

    /// Validate configuration
    pub fn check(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Basic validation
        if self.segments.is_empty() {
            return Err("No segments configured".into());
        }

        // Validate segment IDs are unique
        let mut seen_ids = std::collections::HashSet::new();
        for segment in &self.segments {
            if !seen_ids.insert(segment.id) {
                return Err(format!("Duplicate segment ID: {:?}", segment.id).into());
            }
        }

        Ok(())
    }

    /// Print configuration as TOML
    pub fn print(&self) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        println!("{}", content);
        Ok(())
    }
}
