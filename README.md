# CCometixLine

[English](README.md) | [中文](README.zh.md)

A high-performance Claude Code statusline tool written in Rust with Git integration and real-time usage tracking.

![Language:Rust](https://img.shields.io/static/v1?label=Language&message=Rust&color=orange&style=flat-square)
![License:MIT](https://img.shields.io/static/v1?label=License&message=MIT&color=blue&style=flat-square)

## Screenshots

![CCometixLine](assets/img1.png)

The statusline shows: Model | Directory | Git Branch Status | Usage | Cost Statistics | Burn Rate

## Features

- **High performance** with Rust native speed
- **Git integration** with branch, status, and tracking info  
- **Model display** with simplified Claude model names
- **Usage tracking** based on transcript analysis
- **Cost tracking** with session, daily, and billing block statistics (disabled by default)
- **Burn rate monitoring** for real-time consumption patterns (disabled by default)
- **Directory display** showing current workspace
- **Minimal design** using Nerd Font icons
- **Simple configuration** via command line options
- **Environment variable control** for feature customization

## Installation

### Quick Install (Recommended)

Install via npm (works on all platforms):

```bash
# Install globally
npm install -g @cometix/ccline

# Or using yarn
yarn global add @cometix/ccline

# Or using pnpm
pnpm add -g @cometix/ccline
```

Use npm mirror for faster download:
```bash
npm install -g @cometix/ccline --registry https://registry.npmmirror.com
```

After installation:
- ✅ Global command `ccline` is available everywhere  
- ✅ Automatically configured for Claude Code at `~/.claude/ccline/ccline`
- ✅ Ready to use immediately!

### Update

```bash
npm update -g @cometix/ccline
```

### Manual Installation

Alternatively, download from [Releases](https://github.com/Haleclipse/CCometixLine/releases):

#### Linux

#### Option 1: Dynamic Binary (Recommended)
```bash
mkdir -p ~/.claude/ccline
wget https://github.com/Haleclipse/CCometixLine/releases/latest/download/ccline-linux-x64.tar.gz
tar -xzf ccline-linux-x64.tar.gz
cp ccline ~/.claude/ccline/
chmod +x ~/.claude/ccline/ccline
```
*Requires: Ubuntu 22.04+, CentOS 9+, Debian 11+, RHEL 9+ (glibc 2.35+)*

#### Option 2: Static Binary (Universal Compatibility)
```bash
mkdir -p ~/.claude/ccline
wget https://github.com/Haleclipse/CCometixLine/releases/latest/download/ccline-linux-x64-static.tar.gz
tar -xzf ccline-linux-x64-static.tar.gz
cp ccline ~/.claude/ccline/
chmod +x ~/.claude/ccline/ccline
```
*Works on any Linux distribution (static, no dependencies)*

### macOS (Intel)

```bash  
mkdir -p ~/.claude/ccline
wget https://github.com/Haleclipse/CCometixLine/releases/latest/download/ccline-macos-x64.tar.gz
tar -xzf ccline-macos-x64.tar.gz
cp ccline ~/.claude/ccline/
chmod +x ~/.claude/ccline/ccline
```

### macOS (Apple Silicon)

```bash
mkdir -p ~/.claude/ccline  
wget https://github.com/Haleclipse/CCometixLine/releases/latest/download/ccline-macos-arm64.tar.gz
tar -xzf ccline-macos-arm64.tar.gz
cp ccline ~/.claude/ccline/
chmod +x ~/.claude/ccline/ccline
```

### Windows

```powershell
# Create directory and download
New-Item -ItemType Directory -Force -Path "$env:USERPROFILE\.claude\ccline"
Invoke-WebRequest -Uri "https://github.com/Haleclipse/CCometixLine/releases/latest/download/ccline-windows-x64.zip" -OutFile "ccline-windows-x64.zip"
Expand-Archive -Path "ccline-windows-x64.zip" -DestinationPath "."
Move-Item "ccline.exe" "$env:USERPROFILE\.claude\ccline\"
```

### Claude Code Configuration

Add to your Claude Code `settings.json`:

**Linux/macOS:**
```json
{
  "statusLine": {
    "type": "command", 
    "command": "~/.claude/ccline/ccline",
    "padding": 0
  }
}
```

**Windows:**
```json
{
  "statusLine": {
    "type": "command", 
    "command": "%USERPROFILE%\\.claude\\ccline\\ccline.exe",
    "padding": 0
  }
}
```

### Build from Source

```bash
git clone https://github.com/Haleclipse/CCometixLine.git
cd CCometixLine
cargo build --release

# Linux/macOS
mkdir -p ~/.claude/ccline
cp target/release/ccometixline ~/.claude/ccline/ccline
chmod +x ~/.claude/ccline/ccline

# Windows (PowerShell)
New-Item -ItemType Directory -Force -Path "$env:USERPROFILE\.claude\ccline"
copy target\release\ccometixline.exe "$env:USERPROFILE\.claude\ccline\ccline.exe"
```

## Usage

```bash
# Basic usage (displays all enabled segments)
ccline

# Show help
ccline --help

# Print default configuration  
ccline --print-config

# TUI configuration mode (planned)
ccline --configure

# Billing block management
ccline --set-block-start <time>    # Set billing block start time for today
ccline --clear-block-start          # Clear block start time override
ccline --show-block-status          # Show current block status
```

### Billing Block Synchronization

Solve the problem of billing blocks not syncing when switching between devices with the same account:

```bash
# Set block start time to 10am on device A
ccline --set-block-start 10

# Supported time formats:
ccline --set-block-start 10        # 10:00 (24-hour format)
ccline --set-block-start 10:30     # 10:30
ccline --set-block-start "10:30"   # With quotes works too

# View current settings
ccline --show-block-status

# Clear settings, restore automatic calculation
ccline --clear-block-start
```

## Default Segments

Displays: `Model | Directory | Git Branch Status | Usage`

Note: Cost Statistics and Burn Rate segments are available but disabled by default for optimal performance.

### Performance Debugging

Enable timing statistics for the Cost segment to analyze performance:

**Option 1: Using TUI** (Recommended)
```bash
ccline --config
# Navigate to Cost segment → Tab to Settings → Select Options → Enter
# Toggle 'show_timing' to enable
```

**Option 2: Edit config file**
```toml
# In ~/.claude/ccline/config.toml
[[segments]]
id = "cost"
[segments.options]
show_timing = true  # Shows timing breakdown (L=Load, P=Pricing, C=Calculate, A=Analyze, B=Block)
```

Output example: `$0.50 session · $2.30 today [182ms: L120|P2|C30|A20|B10]`

### Model Display

Shows simplified Claude model names:
- `claude-3-5-sonnet` → `Sonnet 3.5`
- `claude-4-sonnet` → `Sonnet 4`

### Directory Display

Shows current workspace directory with folder icon.

### Git Status Indicators

- Branch name with Nerd Font icon
- Status: `✓` Clean, `●` Dirty, `⚠` Conflicts  
- Remote tracking: `↑n` Ahead, `↓n` Behind

### Usage Display

Token usage percentage based on transcript analysis with context limit tracking.

### Cost Statistics and Burn Rate (Disabled by Default)

CCometixLine includes advanced cost tracking and burn rate monitoring features. These are **disabled by default** for optimal performance.

#### Enabling Cost Features

To enable cost tracking and burn rate monitoring:

1. **Using TUI Configuration** (Recommended):
   ```bash
   ccline --config
   # Navigate to Cost and BurnRate segments
   # Press Enter to enable each segment
   # Press 's' to save configuration
   ```

2. **Manual Configuration**:
   Edit `~/.claude/ccline/config.toml` and set:
   ```toml
   [[segments]]
   id = "cost"
   enabled = true
   
   [[segments]]
   id = "burn_rate"
   enabled = true
   ```

#### What These Features Provide

**Cost Statistics**:
- **Session cost**: Cost for current Claude Code session
- **Daily total**: Total cost for today across all sessions
- **Billing blocks**: 5-hour billing periods with remaining time
- Dynamic billing block algorithm with automatic activity detection
- Manual start time setting for multi-device synchronization

**Burn Rate Monitoring**:
- Real-time token consumption rate with visual indicators
- 🔥 High burn rate (>5000 tokens/min)
- ⚡ Medium burn rate (2000-5000 tokens/min)
- 📊 Normal burn rate (<2000 tokens/min)
- Shows cost per hour projection

#### Advanced Configuration

The Cost and BurnRate segments support additional options in `~/.claude/ccline/config.toml`:

```toml
[[segments]]
id = "cost"
enabled = true

[segments.options]
show_timing = false  # Show performance timing breakdown (default: false)
fast_loader = true   # Use optimized parallel file loader (default: true)

[[segments]]
id = "burn_rate"  
enabled = true

[segments.options]
fast_loader = true   # Use optimized parallel file loader (default: true)
```

**Performance Options**:
- `show_timing`: When enabled, displays timing breakdown for each processing step (L=Load, P=Pricing, C=Calculate, A=Analyze, B=Block)
- `fast_loader`: Uses parallel I/O and memory-mapped files for 4x faster loading (recommended for large usage histories)
- `thread_multiplier`: Adjusts the number of threads used for parallel file loading (default: auto-detect based on CPU cores)
  - Systems with hyperthreading: defaults to 1.5x physical cores
  - Systems without hyperthreading: defaults to 1.0x physical cores
  - Range: 0.5-4.0 (final thread count is clamped between 2-16)

## Configuration

Configuration is managed through `~/.claude/ccline/config.toml`. Use the TUI (`ccline --tui`) for visual configuration or edit the file directly.

## Performance

- **Startup time**: < 50ms (vs ~200ms for TypeScript equivalents)
- **Memory usage**: < 10MB (vs ~25MB for Node.js tools)
- **Binary size**: ~2MB optimized release build

## Requirements

- **Git**: Version 1.5+ (Git 2.22+ recommended for better branch detection)
- **Terminal**: Must support Nerd Fonts for proper icon display
  - Install a [Nerd Font](https://www.nerdfonts.com/) (e.g., FiraCode Nerd Font, JetBrains Mono Nerd Font)
  - Configure your terminal to use the Nerd Font
- **Claude Code**: For statusline integration

## Development

```bash
# Build development version
cargo build

# Run tests
cargo test

# Build optimized release
cargo build --release
```

## Roadmap

- [ ] TOML configuration file support
- [ ] TUI configuration interface
- [ ] Custom themes
- [ ] Plugin system
- [ ] Cross-platform binaries

## Acknowledgments

### ccusage Integration

Cost tracking features are built upon the statistical methods and pricing data from the [ccusage](https://github.com/ryoppippi/ccusage) project.

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

## License

This project is licensed under the [MIT License](LICENSE).

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=Haleclipse/CCometixLine&type=Date)](https://star-history.com/#Haleclipse/CCometixLine&Date)