use crate::render::colors;
use crate::types::RolloutState;

/// Render: ◐ exec_command: ls -la | ✓ exec_command ×12
pub fn render(state: &RolloutState, _term_width: u16) -> Option<String> {
    let running = state.running_tools();
    let counts = &state.tool_counts;

    if running.is_empty() && counts.is_empty() {
        return None;
    }

    let mut parts: Vec<String> = Vec::new();

    // Show running tools (max 2)
    for tool in running.iter().take(2) {
        let elapsed = tool.start_time.elapsed().as_secs();
        let spinner = if elapsed % 4 == 0 { "◐" } else if elapsed % 4 == 1 { "◓" } else if elapsed % 4 == 2 { "◑" } else { "◒" };

        let desc = if tool.args_summary.is_empty() {
            colors::colored(&format!("{} {}", spinner, tool.name), colors::YELLOW)
        } else {
            format!(
                "{} {}",
                colors::colored(&format!("{} {}", spinner, tool.name), colors::YELLOW),
                colors::dim(&format!(": {}", tool.args_summary)),
            )
        };
        parts.push(desc);
    }

    if running.len() > 2 {
        parts.push(colors::dim(&format!("+{} more", running.len() - 2)));
    }

    // Show completed tool counts (top 3 by count)
    let mut sorted_counts: Vec<_> = counts.iter().collect();
    sorted_counts.sort_by(|a, b| b.1.cmp(a.1));

    for (name, count) in sorted_counts.iter().take(3) {
        parts.push(format!(
            "{} {}",
            colors::colored("✓", colors::GREEN),
            if **count > 1 {
                format!("{} ×{}", name, count)
            } else {
                name.to_string()
            }
        ));
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join(&format!(" {} ", colors::dim("|"))))
    }
}
