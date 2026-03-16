# codex-hud

Real-time HUD for [OpenAI Codex CLI](https://github.com/openai/codex) via tmux. Shows model info, context health, rate limit usage, tool activity, and plan progress — all updated live as you work.

```
┌──────────────────────────────────────────┐
│                                          │
│     Codex CLI (main pane)                │
│                                          │
├──────────────────────────────────────────┤
│ [gpt-5.4 high | Plus] │ my-proj git:main│
│ Context █████░░░░░ 47% │ Usage ██░░ 12% │
│ ◐ exec_command: cargo build | ✓ read ×3 │
│ ▸ Refactor auth layer (2/5)             │
└──────────────────────────────────────────┘
```

## Install

### From source

```bash
git clone https://github.com/CLRT19/codex-hud.git
cd codex-hud
cargo build --release
cp target/release/codex-hud ~/.local/bin/  # or anywhere on your PATH
```

### Requirements

- Rust 1.70+
- tmux (optional — falls back to standalone watch mode)
- [Codex CLI](https://github.com/openai/codex) installed

## Usage

### Default: tmux mode

Launch Codex inside a tmux session with the HUD in a bottom pane:

```bash
codex-hud
```

Pass arguments through to Codex:

```bash
codex-hud -- --model gpt-5.4 --approval-mode full-auto
```

### Standalone watch mode

Run the HUD in a separate terminal window while Codex runs elsewhere:

```bash
codex-hud --watch
```

### Attach to existing tmux session

Add a HUD pane to a tmux session you already have running:

```bash
codex-hud --attach my-session
```

### Options

| Flag | Description | Default |
|------|-------------|---------|
| `--watch` | Standalone mode (no tmux) | off |
| `--attach <SESSION>` | Add HUD to existing tmux session | — |
| `--height <N>` | Height of HUD pane in lines | 4 |

## What it shows

| Line | Content | Always shown |
|------|---------|:---:|
| **Project** | `[model effort \| plan] │ project git:(branch*)` | ✓ |
| **Context** | `Context █████░░░░░ N% │ Usage ██░░░░░░░░ N%` | ✓ |
| **Tools** | `◐ exec_command: cmd \| ✓ read_file ×3` | configurable |
| **Plan** | `▸ Step description (2/5)` | configurable |
| **Environment** | `3 MCP servers` | configurable |

### Context color thresholds

| Usage | Color | Meaning |
|-------|-------|---------|
| < 70% | 🟢 Green | Normal |
| 70–85% | 🟡 Yellow | Warning |
| > 85% | 🔴 Red | Critical (shows token breakdown) |

## Configuration

Create `~/.codex-hud/config.toml` to customize:

```toml
# Show tool activity line (default: true)
show_tools = true

# Show plan/step progress line (default: true)
show_plan = true

# Show environment info like MCP server count (default: false)
show_environment = false

# Width of progress bars in characters (default: 10)
bar_width = 10

# Compact single-line mode (default: false)
compact = false
```

## How it works

1. Codex CLI writes session events to `~/.codex/sessions/YYYY/MM/DD/rollout-*.jsonl`
2. `codex-hud` finds the most recent rollout file and watches it for changes using OS-native file events ([notify](https://docs.rs/notify) crate)
3. On each new event, it incrementally parses only the new JSONL lines (no re-reading the whole file)
4. Renders ANSI-colored output to the terminal

Data is read directly from the rollout JSONL — no API calls or OAuth needed. Rate limit info (5h and 7d windows) is embedded in `token_count` events by Codex itself.

### Supported rollout events

| Event | Data extracted |
|-------|---------------|
| `session_meta` | Project directory, git branch, CLI version |
| `turn_context` | Model name, reasoning effort |
| `token_count` | Token usage, context window, rate limits, plan type |
| `function_call` | Tool name, arguments, start time |
| `function_call_output` | Tool completion |
| `update_plan` | Plan steps and their status |

## Development

```bash
cargo build          # Dev build
cargo test           # Run tests (7 tests: parser, bars)
cargo build --release # Optimized build (~1.3MB binary)
```

## License

MIT
