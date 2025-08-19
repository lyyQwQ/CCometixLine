use crate::billing::UsageEntry;
use crate::config::TranscriptEntry;
use ignore::WalkBuilder;
use memchr::memchr_iter;
use memmap2::Mmap;
use rayon::prelude::*;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Optimized data loader using parallel I/O and memory mapping
pub struct FastDataLoader {
    project_dirs: Vec<PathBuf>,
}

/// Buffer type for file reading
enum FileBuf {
    Owned(Vec<u8>),
    Mapped(Mmap),
}

impl FileBuf {
    /// Get the underlying byte slice
    fn as_bytes(&self) -> &[u8] {
        match self {
            FileBuf::Owned(v) => v,
            FileBuf::Mapped(m) => m,
        }
    }
}

impl FastDataLoader {
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

    /// Collect all JSONL file paths using optimized directory traversal
    fn collect_paths(&self) -> Vec<PathBuf> {
        let mut all_paths = Vec::new();

        for dir in &self.project_dirs {
            if !dir.exists() {
                continue;
            }

            let walker = WalkBuilder::new(dir)
                .hidden(false)
                .follow_links(false)
                .standard_filters(false)
                .build();

            for entry in walker.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
                    all_paths.push(path.to_path_buf());
                }
            }
        }

        all_paths
    }

    /// Load all usage data using parallel processing
    pub fn load_all_projects(&mut self) -> Vec<UsageEntry> {
        let paths = self.collect_paths();

        if paths.is_empty() {
            return Vec::new();
        }

        // Global deduplication set (thread-safe)
        let seen_hashes = Arc::new(Mutex::new(HashSet::<String>::with_capacity(10000)));

        // Configure thread pool for optimal I/O parallelism
        // Use global thread pool configuration
        rayon::ThreadPoolBuilder::new()
            .num_threads(std::cmp::min(
                std::thread::available_parallelism()
                    .map(|n| n.get())
                    .unwrap_or(8)
                    * 2,
                16,
            ))
            .build_global()
            .ok(); // Ignore if already configured

        // Process files in parallel using global thread pool
        let all_entries: Vec<UsageEntry> = paths
            .par_iter()
            .flat_map(|path| {
                // Extract session_id from filename
                let session_id = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                // Process single file
                self.process_file(path, &session_id, seen_hashes.clone())
                    .unwrap_or_default()
            })
            .collect();

        // Sort by timestamp
        let mut sorted_entries = all_entries;
        sorted_entries.sort_by_key(|e| e.timestamp);

        sorted_entries
    }

    /// Process a single file with optimized reading
    fn process_file(
        &self,
        path: &Path,
        session_id: &str,
        seen_hashes: Arc<Mutex<HashSet<String>>>,
    ) -> io::Result<Vec<UsageEntry>> {
        let mut entries = Vec::new();

        // Read file using optimal strategy
        let buffer = Self::read_file_fast(path)?;
        let bytes = buffer.as_bytes();

        // Process each line
        Self::for_each_line(bytes, |line| {
            if line.is_empty() {
                return;
            }

            // Parse JSON and extract usage
            if let Some(usage_entry) = self.parse_line(line, session_id, seen_hashes.clone()) {
                entries.push(usage_entry);
            }
        });

        Ok(entries)
    }

    /// Read file using optimal strategy based on size
    fn read_file_fast(path: &Path) -> io::Result<FileBuf> {
        let metadata = fs::metadata(path)?;
        let size = metadata.len() as usize;

        // Small files: read directly into memory
        if size <= 64 * 1024 {
            Ok(FileBuf::Owned(fs::read(path)?))
        } else {
            // Large files: use memory mapping
            let file = File::open(path)?;
            let mmap = unsafe { Mmap::map(&file)? };
            Ok(FileBuf::Mapped(mmap))
        }
    }

    /// Iterate over lines in a byte buffer efficiently
    fn for_each_line(buffer: &[u8], mut callback: impl FnMut(&[u8])) {
        let mut start = 0;

        // Use memchr to find newlines efficiently
        for newline_pos in memchr_iter(b'\n', buffer) {
            let mut end = newline_pos;

            // Handle CRLF
            if end > start && buffer[end - 1] == b'\r' {
                end -= 1;
            }

            if end > start {
                callback(&buffer[start..end]);
            }

            start = newline_pos + 1;
        }

        // Handle last line without newline
        if start < buffer.len() {
            callback(&buffer[start..]);
        }
    }

    /// Parse a single line and extract usage entry
    fn parse_line(
        &self,
        line: &[u8],
        session_id: &str,
        seen_hashes: Arc<Mutex<HashSet<String>>>,
    ) -> Option<UsageEntry> {
        // Parse JSON using sonic-rs
        let entry: TranscriptEntry = sonic_rs::from_slice(line).ok()?;

        // Only process assistant messages with usage data
        if entry.r#type.as_deref() != Some("assistant") {
            return None;
        }

        let message = entry.message.as_ref()?;
        let raw_usage = message.usage.as_ref()?;

        // Deduplication check
        if let (Some(msg_id), Some(req_id)) = (message.id.as_ref(), entry.request_id.as_ref()) {
            let hash = format!("{}:{}", msg_id, req_id);

            let mut seen = seen_hashes.lock().unwrap();
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

impl Default for FastDataLoader {
    fn default() -> Self {
        Self::new()
    }
}
