use crate::render::bars;
use crate::render::colors;
use crate::types::RolloutState;
use chrono::{DateTime, Utc};

/// Render: Usage ██░░░░░░░░ N% (Xh Ym / 5h)
pub fn render(state: &RolloutState, bar_width: usize) -> Option<String> {
    let usage_pct = state.primary_usage?;

    let bar = bars::labeled_bar("Usage", usage_pct, bar_width);

    let time_info = format_time_remaining(
        state.primary_resets_at.as_deref(),
        state.primary_window_secs,
    );

    Some(if time_info.is_empty() {
        bar
    } else {
        format!("{} {}", bar, colors::dim(&time_info))
    })
}

/// Render secondary (7-day) usage if present
pub fn render_secondary(state: &RolloutState, bar_width: usize) -> Option<String> {
    let usage_pct = state.secondary_usage?;
    let bar = bars::labeled_bar("7d", usage_pct, bar_width);
    Some(bar)
}

fn format_time_remaining(resets_at: Option<&str>, window_secs: Option<u64>) -> String {
    let (remaining, window) = match (resets_at, window_secs) {
        (Some(r), Some(w)) => {
            let reset_time = r.parse::<DateTime<Utc>>().ok();
            let remaining = reset_time.map(|rt| {
                let now = Utc::now();
                let diff = rt - now;
                diff.num_seconds().max(0) as u64
            });
            (remaining, w)
        }
        _ => return String::new(),
    };

    match remaining {
        Some(secs) => {
            let elapsed = window.saturating_sub(secs);
            let elapsed_str = format_duration(elapsed);
            let window_str = format_duration(window);
            format!("({} / {})", elapsed_str, window_str)
        }
        None => String::new(),
    }
}

fn format_duration(secs: u64) -> String {
    let hours = secs / 3600;
    let mins = (secs % 3600) / 60;
    if hours > 0 && mins > 0 {
        format!("{}h {}m", hours, mins)
    } else if hours > 0 {
        format!("{}h", hours)
    } else {
        format!("{}m", mins)
    }
}
