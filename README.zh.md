# CCometixLine

[English](README.md) | [中文](README.zh.md)

基于 Rust 的高性能 Claude Code 状态栏工具，集成 Git 信息和实时使用量跟踪。

![Language:Rust](https://img.shields.io/static/v1?label=Language&message=Rust&color=orange&style=flat-square)
![License:MIT](https://img.shields.io/static/v1?label=License&message=MIT&color=blue&style=flat-square)

## 截图

![CCometixLine](assets/img1.png)

状态栏显示：模型 | 目录 | Git 分支状态 | 使用量 | 成本统计 | 燃烧率

## 特性

- **高性能** Rust 原生速度
- **Git 集成** 显示分支、状态和跟踪信息
- **模型显示** 简化的 Claude 模型名称
- **使用量跟踪** 基于转录文件分析
- **成本追踪** 显示会话、日常和计费块统计信息（默认禁用）
- **燃烧率监控** 实时消耗模式监控（默认禁用）
- **目录显示** 显示当前工作空间
- **简洁设计** 使用 Nerd Font 图标
- **简单配置** 通过命令行选项配置
- **环境变量控制** 功能自定义选项

## 安装

### 快速安装（推荐）

通过 npm 安装（适用于所有平台）：

```bash
# 全局安装
npm install -g @cometix/ccline

# 或使用 yarn
yarn global add @cometix/ccline

# 或使用 pnpm
pnpm add -g @cometix/ccline
```

使用镜像源加速下载：
```bash
npm install -g @cometix/ccline --registry https://registry.npmmirror.com
```

安装后：
- ✅ 全局命令 `ccline` 可在任何地方使用
- ✅ 自动配置 Claude Code 到 `~/.claude/ccline/ccline`
- ✅ 立即可用！

### 更新

```bash
npm update -g @cometix/ccline
```

### 手动安装

或者从 [Releases](https://github.com/Haleclipse/CCometixLine/releases) 手动下载：

#### Linux

#### 选项 1: 动态链接版本（推荐）
```bash
mkdir -p ~/.claude/ccline
wget https://github.com/Haleclipse/CCometixLine/releases/latest/download/ccline-linux-x64.tar.gz
tar -xzf ccline-linux-x64.tar.gz
cp ccline ~/.claude/ccline/
chmod +x ~/.claude/ccline/ccline
```
*系统要求: Ubuntu 22.04+, CentOS 9+, Debian 11+, RHEL 9+ (glibc 2.35+)*

#### 选项 2: 静态链接版本（通用兼容）
```bash
mkdir -p ~/.claude/ccline
wget https://github.com/Haleclipse/CCometixLine/releases/latest/download/ccline-linux-x64-static.tar.gz
tar -xzf ccline-linux-x64-static.tar.gz
cp ccline ~/.claude/ccline/
chmod +x ~/.claude/ccline/ccline
```
*适用于任何 Linux 发行版（静态链接，无依赖）*

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
# 创建目录并下载
New-Item -ItemType Directory -Force -Path "$env:USERPROFILE\.claude\ccline"
Invoke-WebRequest -Uri "https://github.com/Haleclipse/CCometixLine/releases/latest/download/ccline-windows-x64.zip" -OutFile "ccline-windows-x64.zip"
Expand-Archive -Path "ccline-windows-x64.zip" -DestinationPath "."
Move-Item "ccline.exe" "$env:USERPROFILE\.claude\ccline\"
```

### 从源码构建

```bash
git clone https://github.com/Haleclipse/CCometixLine.git
cd CCometixLine
cargo build --release
cp target/release/ccometixline ~/.claude/ccline/ccline
```

### Claude Code 配置

添加到 Claude Code `settings.json`：

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

## 使用

```bash
# 基础使用 (显示所有启用的段落)
ccline

# 显示帮助
ccline --help

# 打印默认配置
ccline --print-config

# TUI 配置模式 (计划中)
ccline --configure

# 计费块管理
ccline --set-block-start <时间>    # 设置当天计费块开始时间
ccline --clear-block-start          # 清除计费块开始时间设置
ccline --show-block-status          # 显示当前计费块状态
```

### 计费块同步功能

解决同一账号在多设备间切换时计费块不同步的问题：

```bash
# 在设备A上设置块开始时间为上午10点
ccline --set-block-start 10

# 支持的时间格式：
ccline --set-block-start 10        # 10:00 (24小时制)
ccline --set-block-start 10:30     # 10:30
ccline --set-block-start "10:30"   # 带引号也可以

# 查看当前设置
ccline --show-block-status

# 清除设置，恢复自动计算
ccline --clear-block-start
```

## 默认段落

显示：`模型 | 目录 | Git 分支状态 | 使用量`

注意：成本统计和燃烧率段落可用但默认禁用以获得最佳性能。

### 性能调试

启用成本段落的耗时统计以分析性能：

**选项 1：使用 TUI**（推荐）
```bash
ccline --config
# 导航到 Cost 段 → Tab 切换到设置 → 选择 Options → 回车
# 切换 'show_timing' 来启用
```

**选项 2：编辑配置文件**
```toml
# 在 ~/.claude/ccline/config.toml
[[segments]]
id = "cost"
[segments.options]
show_timing = true  # 显示耗时细分 (L=加载, P=定价, C=计算, A=分析, B=块)
```

输出示例：`$0.50 session · $2.30 today [182ms: L120|P2|C30|A20|B10]`

### 模型显示

显示简化的 Claude 模型名称：
- `claude-3-5-sonnet` → `Sonnet 3.5`
- `claude-4-sonnet` → `Sonnet 4`

### 目录显示

显示当前工作空间目录和文件夹图标。

### Git 状态指示器

- 带 Nerd Font 图标的分支名
- 状态：`✓` 清洁，`●` 有更改，`⚠` 冲突
- 远程跟踪：`↑n` 领先，`↓n` 落后

### 使用量显示

基于转录文件分析的令牌使用百分比，包含上下文限制跟踪。

### 成本统计和燃烧率监控（默认禁用）

CCometixLine 包含高级成本追踪和燃烧率监控功能。为了获得最佳性能，这些功能**默认禁用**。

#### 启用成本功能

要启用成本追踪和燃烧率监控：

1. **使用 TUI 配置**（推荐）：
   ```bash
   ccline --config
   # 导航到 Cost 和 BurnRate 段
   # 按 Enter 启用各个段
   # 按 's' 保存配置
   ```

2. **手动配置**：
   编辑 `~/.claude/ccline/config.toml` 并设置：
   ```toml
   [[segments]]
   id = "cost"
   enabled = true
   
   [[segments]]
   id = "burn_rate"
   enabled = true
   ```

#### 这些功能提供什么

**成本统计**：
- **会话成本**：当前 Claude Code 会话的成本
- **日常总计**：今日所有会话的总成本
- **计费块**：5小时计费周期及剩余时间
- 动态计费块算法，自动活动检测
- 手动设置开始时间以在多设备间同步

**燃烧率监控**：
- 实时令牌消耗率监控和视觉指示器
- 🔥 高燃烧率 (>5000 tokens/分钟)
- ⚡ 中等燃烧率 (2000-5000 tokens/分钟)
- 📊 正常燃烧率 (<2000 tokens/分钟)
- 显示每小时成本预测

#### 高级配置

Cost 和 BurnRate 段在 `~/.claude/ccline/config.toml` 中支持额外选项：

```toml
[[segments]]
id = "cost"
enabled = true

[segments.options]
show_timing = false  # 显示性能计时细分（默认：false）
fast_loader = true   # 使用优化的并行文件加载器（默认：true）

[[segments]]
id = "burn_rate"  
enabled = true

[segments.options]
fast_loader = true   # 使用优化的并行文件加载器（默认：true）
```

**性能选项**：
- `show_timing`：启用时，显示每个处理步骤的计时细分（L=加载，P=定价，C=计算，A=分析，B=块）
- `fast_loader`：使用并行 I/O 和内存映射文件，实现 4 倍速度提升（推荐用于大量使用历史记录）
- `thread_multiplier`：调整并行文件加载使用的线程数（默认：根据 CPU 核心自动检测）
  - 支持超线程的系统：默认为物理核心数的 1.5 倍
  - 不支持超线程的系统：默认为物理核心数的 1.0 倍
  - 范围：0.5-4.0（最终线程数限制在 2-16 之间）

## 配置

配置通过 `~/.claude/ccline/config.toml` 管理。使用 TUI（`ccline --tui`）进行可视化配置或直接编辑文件。

## 性能

- **启动时间**：< 50ms（TypeScript 版本约 200ms）
- **内存使用**：< 10MB（Node.js 工具约 25MB）
- **二进制大小**：约 2MB 优化版本

## 系统要求

- **Git**: 版本 1.5+ (推荐 Git 2.22+ 以获得更好的分支检测)
- **终端**: 必须支持 Nerd Font 图标正常显示
  - 安装 [Nerd Font](https://www.nerdfonts.com/) 字体
  - 中文用户推荐: [Maple Font](https://github.com/subframe7536/maple-font) (支持中文的 Nerd Font)
  - 在终端中配置使用该字体
- **Claude Code**: 用于状态栏集成

## 开发

```bash
# 构建开发版本
cargo build

# 运行测试
cargo test

# 构建优化版本
cargo build --release
```

## 路线图

- [ ] TOML 配置文件支持
- [ ] TUI 配置界面
- [ ] 自定义主题
- [ ] 插件系统
- [ ] 跨平台二进制文件

## 致谢

### ccusage 集成

成本追踪功能基于 [ccusage](https://github.com/ryoppippi/ccusage) 项目的统计方法和定价数据实现。

## 贡献

欢迎贡献！请随时提交 issue 或 pull request。

## 许可证

本项目采用 [MIT 许可证](LICENSE)。

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=Haleclipse/CCometixLine&type=Date)](https://star-history.com/#Haleclipse/CCometixLine&Date)