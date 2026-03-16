use crate::render::bars;
use crate::render::colors;
use crate::types::RolloutState;

/// Render: Context █████░░░░░ N%
pub fn render(state: &RolloutState, bar_width: usize) -> String {
    match state.context_percent() {
        Some(pct) => {
            let bar = bars::labeled_bar("Context", pct, bar_width);

            // At high usage, show token breakdown
            if pct > 85.0 {
                let total = state.total_tokens.unwrap_or(0);
                let window = state.context_window.unwrap_or(0);
                let detail = colors::dim(&format!(" ({}/{})", format_tokens(total), format_tokens(window)));
                format!("{}{}", bar, detail)
            } else {
                bar
            }
        }
        None => colors::dim("Context: waiting for data..."),
    }
}

fn format_tokens(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.0}k", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}
