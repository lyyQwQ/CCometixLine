use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "ccline")]
#[command(version, about = "High-performance Claude Code StatusLine")]
pub struct Cli {
    /// Enter TUI configuration mode
    #[arg(short = 'c', long = "config")]
    pub config: bool,

    /// Set theme
    #[arg(short = 't', long = "theme")]
    pub theme: Option<String>,

    /// Print current configuration
    #[arg(long = "print")]
    pub print: bool,

    /// Initialize config file
    #[arg(long = "init")]
    pub init: bool,

    /// Check configuration
    #[arg(long = "check")]
    pub check: bool,

    /// Check for updates
    #[arg(short = 'u', long = "update")]
    pub update: bool,

    /// Set block start time for today (formats: 0-23, HH:MM, ISO timestamp)
    #[arg(long, value_name = "TIME")]
    pub set_block_start: Option<String>,

    /// Clear block start override for today
    #[arg(long)]
    pub clear_block_start: bool,

    /// Show current block override status
    #[arg(long)]
    pub show_block_status: bool,

    /// Set context window limit for usage calculation (in tokens)
    #[arg(long = "context-limit", value_name = "TOKENS")]
    pub context_limit: Option<u32>,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
