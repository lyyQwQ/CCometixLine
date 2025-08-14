use ccometixline::utils::DataLoader;
use chrono::Local;

fn main() {
    println!("=== Testing DataLoader ===\n");

    // Create data loader
    let loader = DataLoader::new();

    // Load all project data
    println!("Loading all project data...");
    let start = std::time::Instant::now();
    let entries = loader.load_all_projects();
    let elapsed = start.elapsed();

    println!("✅ Loaded {} entries in {:?}\n", entries.len(), elapsed);

    // Show summary statistics
    if !entries.is_empty() {
        // Group by session
        let mut sessions = std::collections::HashMap::new();
        for entry in &entries {
            sessions
                .entry(entry.session_id.clone())
                .or_insert_with(Vec::new)
                .push(entry);
        }

        println!("📊 Summary Statistics:");
        println!("  Total sessions: {}", sessions.len());
        println!("  Total entries: {}", entries.len());

        // Show today's entries
        let today = Local::now().date_naive();
        let today_entries: Vec<_> = entries
            .iter()
            .filter(|e| e.timestamp.date_naive() == today)
            .collect();
        println!("  Today's entries: {}", today_entries.len());

        // Calculate total tokens
        let total_input: u32 = entries.iter().map(|e| e.input_tokens).sum();
        let total_output: u32 = entries.iter().map(|e| e.output_tokens).sum();
        let total_cache_creation: u32 = entries.iter().map(|e| e.cache_creation_tokens).sum();
        let total_cache_read: u32 = entries.iter().map(|e| e.cache_read_tokens).sum();

        println!("\n📈 Token Usage:");
        println!("  Input tokens: {}", format_number(total_input));
        println!("  Output tokens: {}", format_number(total_output));
        println!("  Cache creation: {}", format_number(total_cache_creation));
        println!("  Cache read: {}", format_number(total_cache_read));
        println!(
            "  Total: {}",
            format_number(total_input + total_output + total_cache_creation + total_cache_read)
        );

        // Show first few entries
        println!("\n📝 First 5 entries:");
        for (i, entry) in entries.iter().take(5).enumerate() {
            println!(
                "  {}. Session: {} (first 8 chars)",
                i + 1,
                &entry.session_id[..8.min(entry.session_id.len())]
            );
            println!("     Time: {}", entry.timestamp.format("%Y-%m-%d %H:%M:%S"));
            println!(
                "     Tokens: in={}, out={}, cache_create={}, cache_read={}",
                entry.input_tokens,
                entry.output_tokens,
                entry.cache_creation_tokens,
                entry.cache_read_tokens
            );
        }

        // Show last entry
        if let Some(last) = entries.last() {
            println!("\n📝 Last entry:");
            println!(
                "  Session: {} (first 8 chars)",
                &last.session_id[..8.min(last.session_id.len())]
            );
            println!("  Time: {}", last.timestamp.format("%Y-%m-%d %H:%M:%S"));
            println!(
                "  Tokens: in={}, out={}, cache_create={}, cache_read={}",
                last.input_tokens,
                last.output_tokens,
                last.cache_creation_tokens,
                last.cache_read_tokens
            );
        }
    } else {
        println!("❌ No entries found!");
        println!("\nPossible reasons:");
        println!("  - No Claude projects in ~/.claude/projects or ~/.config/claude/projects");
        println!("  - No JSONL files in project directories");
        println!("  - Files don't contain assistant messages with usage data");
    }

    // Test environment variable support
    println!("\n🔧 Environment Configuration:");
    if let Ok(custom_dir) = std::env::var("CLAUDE_CONFIG_DIR") {
        println!("  CLAUDE_CONFIG_DIR is set: {}", custom_dir);
    } else {
        println!("  CLAUDE_CONFIG_DIR not set (using default paths)");
    }
}

fn format_number(n: u32) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}k", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}
