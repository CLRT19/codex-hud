pub mod bars;
pub mod colors;
pub mod lines;

use crate::codex_config::CodexConfig;
use crate::config::HudConfig;
use crate::types::RolloutState;
use crossterm::{cursor, terminal, ExecutableCommand};
use std::io::{self, Write};

/// Render the full HUD display to stdout
pub fn render(state: &RolloutState, config: &HudConfig, codex_config: &CodexConfig) {
    let term_width = terminal::size().map(|(w, _)| w).unwrap_or(80);

    let mut output = Vec::new();

    // Line 1: Project line (always shown)
    output.push(lines::project::render(state, term_width));

    // Line 2: Context + Usage (always shown)
    let context = lines::context::render(state, config.bar_width);
    let usage = lines::usage::render(state, config.bar_width);

    match usage {
        Some(u) => output.push(format!(
            "{} {} {}",
            context,
            colors::dim("│"),
            u
        )),
        None => output.push(context),
    }

    // Optional: Secondary usage (7-day)
    if let Some(secondary) = lines::usage::render_secondary(state, config.bar_width) {
        // Only show if it's getting notable
        if state.secondary_usage.unwrap_or(0.0) > 5.0 {
            output.push(secondary);
        }
    }

    // Optional: Tools line
    if config.show_tools {
        if let Some(tools_line) = lines::tools::render(state, term_width) {
            output.push(tools_line);
        }
    }

    // Optional: Plan line
    if config.show_plan {
        if let Some(plan_line) = lines::plan::render(state) {
            output.push(plan_line);
        }
    }

    // Optional: Environment line
    if config.show_environment {
        if let Some(env_line) = lines::environment::render(codex_config) {
            output.push(env_line);
        }
    }

    // Write output: cursor home, clear screen, draw lines
    let mut stdout = io::stdout();
    let _ = stdout.execute(cursor::MoveTo(0, 0));
    let _ = stdout.execute(terminal::Clear(terminal::ClearType::All));

    for line in &output {
        let _ = writeln!(stdout, "{}", line);
    }
    let _ = stdout.flush();
}
