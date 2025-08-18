use crate::billing::UsageEntry;
use glob::glob;
use std::collections::HashSet;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

pub struct DataLoader {
    project_dirs: Vec<PathBuf>,
}

impl DataLoader {
    pub fn new() -> Self {
        Self {
            project_dirs: Self::find_claude_dirs(),
        }
    }

    /// Find all Claude data directories
    fn find_claude_dirs() -> Vec<PathBuf> {
        let mut dirs = Vec::new();

        // Get home directory
        if let Ok(home) = std::env::var("HOME") {
            // New version path (~/.config/claude/projects)
            let new_path = PathBuf::from(&home).join(".config/claude/projects");
            if new_path.exists() {
                dirs.push(new_path);
            }

            // Legacy path (~/.claude/projects)
            let old_path = PathBuf::from(&home).join(".claude/projects");
            if old_path.exists() {
                dirs.push(old_path);
            }
        }

        // Support custom directories via environment variable
        if let Ok(custom_dirs) = std::env::var("CLAUDE_CONFIG_DIR") {
            for dir in custom_dirs.split(',') {
                let path = PathBuf::from(dir.trim()).join("projects");
                if path.exists() {
                    dirs.push(path);
                }
            }
        }

        dirs
    }

    /// Load all usage data from all projects (optimized serial version)
    pub fn load_all_projects(&mut self) -> Vec<UsageEntry> {
        let mut all_entries = Vec::new();
        let mut seen_hashes = HashSet::new();

        // Scan all project directories
        for dir in &self.project_dirs {
            let pattern = format!("{}/**/*.jsonl", dir.display());
            if let Ok(paths) = glob(&pattern) {
                for path in paths.flatten() {
                    // Extract session_id from filename
                    let session_id = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    // Parse the file using optimized method
                    let entries =
                        self.parse_jsonl_file_optimized(&path, &session_id, &mut seen_hashes);
                    all_entries.extend(entries);
                }
            }
        }

        // Sort by timestamp
        all_entries.sort_by_key(|e| e.timestamp);

        all_entries
    }

    /// Parse a single JSONL file with optimizations
    fn parse_jsonl_file_optimized(
        &self,
        path: &Path,
        session_id: &str,
        seen: &mut HashSet<String>,
    ) -> Vec<UsageEntry> {
        let mut entries = Vec::new();

        // Skip if file doesn't exist or can't be opened
        let file = match fs::File::open(path) {
            Ok(f) => f,
            Err(_) => return entries,
        };

        // Use buffered reader for all files
        let reader = BufReader::new(file);
        for line in reader.lines().map_while(Result::ok) {
            if line.trim().is_empty() {
                continue;
            }
            if let Some(usage_entry) = self.parse_line_optimized(&line, session_id, seen) {
                entries.push(usage_entry);
            }
        }

        entries
    }

    /// Parse a line with optimized JSON parsing
    fn parse_line_optimized(
        &self,
        line: &str,
        session_id: &str,
        seen: &mut HashSet<String>,
    ) -> Option<UsageEntry> {
        // Parse the JSON line using sonic-rs for better performance
        let entry: crate::config::TranscriptEntry = sonic_rs::from_str(line).ok()?;

        // Only process assistant messages with usage data
        if entry.r#type.as_deref() != Some("assistant") {
            return None;
        }

        let message = entry.message.as_ref()?;
        let raw_usage = message.usage.as_ref()?;

        // Deduplication check
        if let (Some(msg_id), Some(req_id)) = (message.id.as_ref(), entry.request_id.as_ref()) {
            let hash = format!("{}:{}", msg_id, req_id);
            if seen.contains(&hash) {
                return None; // Skip duplicate
            }
            seen.insert(hash);
        }

        // Normalize the usage data
        let normalized = raw_usage.clone().normalize();

        // Get model name from message
        let model = message.model.as_deref();

        // Convert to UsageEntry
        crate::utils::transcript::extract_usage_entry(
            &normalized,
            session_id,
            entry.timestamp.as_deref(),
            model,
        )
    }
}

impl Default for DataLoader {
    fn default() -> Self {
        Self::new()
    }
}
