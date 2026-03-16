# CLAUDE.md

## Project Overview

Codex HUD is a real-time statusline for OpenAI's Codex CLI, built in Rust. It displays model info, context health, rate limit usage, tool activity, and plan progress via a tmux split pane or standalone terminal watcher.

## Build Commands

```bash
cargo build              # Dev build
cargo build --release    # Optimized release build (1.3MB binary)
cargo test               # Run all tests
```

## Architecture

### Data Flow

```
~/.codex/sessions/YYYY/MM/DD/rollout-*.jsonl
  → notify file watcher → streaming JSONL parser → render lines → terminal output
```

### How It Works

1. `codex-hud` creates a tmux session with Codex in the main pane + HUD in a bottom pane
2. The HUD pane watches the latest rollout JSONL file using `notify` crate
3. On each file change, it parses only new lines (streaming/incremental)
4. Re-renders ANSI-colored output to fill the pane

Also supports `codex-hud --watch` standalone mode (separate terminal, no tmux).

### Data Sources

| Data | Source |
|------|--------|
| Model, effort | `turn_context` event in rollout JSONL |
| Tokens, context % | `token_count` event |
| Rate limits (5h/7d) | `token_count.rate_limits` |
| Plan type | `rate_limits.plan_type` |
| Tools | `function_call` / `function_call_output` events |
| Plan steps | `update_plan` function call args |
| Git branch | `session_meta.git.branch` |
| MCP servers | `~/.codex/config.toml` |

### File Structure

```
src/
├── main.rs               # CLI: arg parsing, tmux or watch mode
├── tmux.rs               # tmux session/pane management
├── watcher.rs            # Main loop: find session → watch → parse → render
├── session_finder.rs     # Find active rollout in ~/.codex/sessions/
├── rollout_parser.rs     # Streaming JSONL parser
├── types.rs              # Data structs: RolloutState, ToolEntry, PlanStep
├── config.rs             # HUD config (~/.codex-hud/config.toml)
├── codex_config.rs       # Read ~/.codex/config.toml for MCP info
├── git.rs                # Git branch/dirty/ahead-behind
└── render/
    ├── mod.rs            # Render coordinator
    ├── colors.rs         # ANSI color helpers via crossterm
    ├── bars.rs           # Progress bar rendering (█░)
    └── lines/
        ├── mod.rs        # Barrel exports
        ├── project.rs    # [model effort | plan] │ project git:(branch*)
        ├── context.rs    # Context bar + token counts
        ├── usage.rs      # Rate limit bars (5h + 7d)
        ├── tools.rs      # Running/completed function calls
        ├── plan.rs       # Plan step progress
        └── environment.rs # MCP servers count
```

### Output Format

```
[gpt-5.4 high | Plus] │ my-project git:(main*)
Context █████░░░░░ 47% │ Usage ██░░░░░░░░ 2% (4h 58m / 5h)
◐ exec_command: rg --files | ✓ exec_command ×12
▸ Refactor authentication layer (2/5)
```

### Context Thresholds

| Threshold | Color |
|-----------|-------|
| <70% | Green |
| 70-85% | Yellow |
| >85% | Red |

## Dependencies

- `crossterm` — terminal colors, cursor control
- `serde` + `serde_json` — JSONL parsing
- `toml` — config file parsing
- `notify` — file system watcher
- `clap` — CLI arguments
- `chrono` — time formatting
