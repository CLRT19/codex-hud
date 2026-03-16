use crate::render::colors;
use crate::types::RolloutState;

/// Render: ▸ Current step description (2/5)
pub fn render(state: &RolloutState) -> Option<String> {
    if state.plan_steps.is_empty() {
        return None;
    }

    let (done, total) = state.plan_progress();

    // Find the first in-progress step, or the first not-completed step
    let current = state
        .plan_steps
        .iter()
        .find(|s| s.status == "in_progress")
        .or_else(|| state.plan_steps.iter().find(|s| s.status != "completed"));

    let step_desc = current
        .map(|s| truncate(&s.step, 50))
        .unwrap_or_else(|| "All steps complete".to_string());

    let progress = colors::dim(&format!("({}/{})", done, total));

    let marker = if done == total {
        colors::colored("✓", colors::GREEN)
    } else {
        colors::colored("▸", colors::CYAN)
    };

    Some(format!("{} {} {}", marker, step_desc, progress))
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max - 1])
    }
}
