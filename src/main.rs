use ccometixline::cli::Cli;
use ccometixline::config::{BlockOverrideManager, Config, InputData};
use ccometixline::core::{collect_all_segments, StatusLineGenerator};
use chrono::{Local, NaiveDate, Utc};
use std::io;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse_args();

    // Handle configuration commands
    if cli.init {
        Config::init()?;
        return Ok(());
    }

    if cli.print {
        let mut config = Config::load().unwrap_or_else(|_| Config::default());

        // Apply theme override if provided
        if let Some(theme) = cli.theme {
            config = ccometixline::ui::themes::ThemePresets::get_theme(&theme);
        }

        config.print()?;
        return Ok(());
    }

    if cli.check {
        let config = Config::load()?;
        config.check()?;
        println!("✓ Configuration valid");
        return Ok(());
    }

    if cli.config {
        #[cfg(feature = "tui")]
        {
            ccometixline::ui::run_configurator()?;
        }
        #[cfg(not(feature = "tui"))]
        {
            eprintln!("TUI feature is not enabled. Please install with --features tui");
            std::process::exit(1);
        }
        return Ok(());
    }

    if cli.update {
        #[cfg(feature = "self-update")]
        {
            println!("Update feature not implemented in new architecture yet");
        }
        #[cfg(not(feature = "self-update"))]
        {
            println!("Update check not available (self-update feature disabled)");
        }
        return Ok(());
    }

    // Handle block start time management
    if cli.set_block_start.is_some() || cli.clear_block_start || cli.show_block_status {
        handle_block_management(&cli)?;
        return Ok(());
    }

    // Handle context limit setting
    if let Some(context_limit) = cli.context_limit {
        if context_limit == 0 {
            eprintln!("Error: Context limit must be greater than 0");
            std::process::exit(1);
        }

        let mut config = Config::load().unwrap_or_else(|_| Config::default());
        config.global.context_limit = context_limit;

        // Validate the configuration
        if let Err(e) = config.global.validate() {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }

        config.save()?;
        println!("Context limit set to {} tokens", context_limit);
        return Ok(());
    }

    // Load configuration
    let mut config = Config::load().unwrap_or_else(|_| Config::default());

    // Apply theme override if provided
    if let Some(theme) = cli.theme {
        config = ccometixline::ui::themes::ThemePresets::get_theme(&theme);
    }

    // Read Claude Code data from stdin
    let stdin = io::stdin();
    let input: InputData = serde_json::from_reader(stdin.lock())?;

    // Collect segment data
    let segments_data = collect_all_segments(&config, &input);

    // Render statusline
    let generator = StatusLineGenerator::new(config);
    let statusline = generator.generate(segments_data);

    println!("{}", statusline);

    Ok(())
}

/// Handle block start time management CLI commands
fn handle_block_management(cli: &Cli) -> io::Result<()> {
    let mut manager = match BlockOverrideManager::new() {
        Ok(manager) => manager,
        Err(e) => {
            eprintln!("Error: Failed to initialize block override manager: {}", e);
            return Err(io::Error::other(e));
        }
    };

    // Load existing configuration
    if let Err(e) = manager.load() {
        eprintln!("Warning: Failed to load existing configuration: {}", e);
    }

    let today = Local::now().date_naive();

    // Handle set block start time
    if let Some(time_input) = &cli.set_block_start {
        match BlockOverrideManager::parse_time_input(time_input) {
            Ok(start_time) => {
                let notes = Some(format!(
                    "Set via CLI at {}",
                    Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
                ));

                match manager.set_override(today, start_time, "manual".to_string(), notes) {
                    Ok(()) => {
                        let local_start_time = start_time.with_timezone(&Local);
                        println!(
                            "✓ Block start time set to {} ({} local) for {}",
                            start_time.format("%Y-%m-%d %H:%M UTC"),
                            local_start_time.format("%H:%M %Z"),
                            today.format("%Y-%m-%d")
                        );
                        println!("  Configuration saved to: {:?}", manager.get_config_path());
                    }
                    Err(e) => {
                        eprintln!("Error: Failed to set block start time: {}", e);
                        return Err(io::Error::other(e));
                    }
                }
            }
            Err(e) => {
                eprintln!("Error: Invalid time format: {}", e);
                let now_local = Local::now();
                eprintln!(
                    "Valid formats: single hour (0-23), HH:MM, or ISO timestamp (YYYY-MM-DDTHH:MM:SSZ)"
                );
                eprintln!(
                    "Times are interpreted as local time (current: {})",
                    now_local.format("%H:%M %Z")
                );
                return Err(io::Error::new(io::ErrorKind::InvalidInput, e));
            }
        }
    }

    // Handle clear block start
    if cli.clear_block_start {
        match manager.clear_override(today) {
            Ok(true) => {
                println!(
                    "✓ Block start time override cleared for {}",
                    today.format("%Y-%m-%d")
                );
            }
            Ok(false) => {
                println!(
                    "ℹ No block start time override was set for {}",
                    today.format("%Y-%m-%d")
                );
            }
            Err(e) => {
                eprintln!("Error: Failed to clear block start time: {}", e);
                return Err(io::Error::other(e));
            }
        }
    }

    // Handle show block status
    if cli.show_block_status {
        println!("Block Override Status:");
        println!("  Configuration file: {:?}", manager.get_config_path());
        println!("  Total overrides: {}", manager.override_count());

        if let Some(override_config) = manager.get_override(today) {
            println!("\n  Today ({}):", today.format("%Y-%m-%d"));
            println!("    ✓ Override active");
            let local_start_time = override_config.start_time.with_timezone(&Local);
            println!(
                "    ⏰ Block starts at: {} ({} local)",
                override_config.start_time.format("%H:%M UTC"),
                local_start_time.format("%H:%M %Z")
            );
            println!("    📝 Source: {}", override_config.source);
            println!(
                "    🕐 Created: {}",
                override_config.created_at.format("%Y-%m-%d %H:%M UTC")
            );
            if let Some(ref notes) = override_config.notes {
                println!("    📋 Notes: {}", notes);
            }
        } else {
            println!("\n  Today ({}):", today.format("%Y-%m-%d"));
            println!("    ⏱️ No override set (will use automatic detection)");
        }

        // Show recent overrides for context
        let all_dates = manager.get_all_dates();
        if !all_dates.is_empty() {
            println!("\n  Recent overrides:");
            let mut sorted_dates = all_dates;
            sorted_dates.sort();
            for date_str in sorted_dates.iter().rev().take(5) {
                if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                    if let Some(override_config) = manager.get_override(date) {
                        println!(
                            "    {} -> {} ({})",
                            date.format("%Y-%m-%d"),
                            override_config.start_time.format("%H:%M UTC"),
                            override_config.source
                        );
                    }
                }
            }
        }
    }

    Ok(())
}
