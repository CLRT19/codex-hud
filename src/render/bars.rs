use crossterm::style::Color;
use super::colors;

const FULL_BLOCK: char = '█';
const EMPTY_BLOCK: char = '░';

/// Render a progress bar: █████░░░░░
pub fn progress_bar(percent: f64, width: usize, color: Color) -> String {
    let clamped = percent.clamp(0.0, 100.0);
    let filled = ((clamped / 100.0) * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);

    let filled_str: String = std::iter::repeat(FULL_BLOCK).take(filled).collect();
    let empty_str: String = std::iter::repeat(EMPTY_BLOCK).take(empty).collect();

    format!(
        "{}{}",
        colors::colored(&filled_str, color),
        colors::dim(&empty_str),
    )
}

/// Render a labeled progress bar: Label █████░░░░░ N%
pub fn labeled_bar(label: &str, percent: f64, width: usize) -> String {
    let color = colors::threshold_color(percent);
    let bar = progress_bar(percent, width, color);
    let pct = colors::colored(&format!("{:.0}%", percent), color);
    format!("{} {} {}", colors::bold(label), bar, pct)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar_0() {
        // At 0%, should have all empty blocks (ignoring ANSI codes)
        let bar = progress_bar(0.0, 10, colors::GREEN);
        let stripped = strip_ansi(&bar);
        assert_eq!(stripped.chars().count(), 10);
        assert!(stripped.chars().all(|c| c == '░'));
    }

    #[test]
    fn test_progress_bar_100() {
        let bar = progress_bar(100.0, 10, colors::GREEN);
        let stripped = strip_ansi(&bar);
        assert_eq!(stripped.chars().count(), 10);
        assert!(stripped.chars().all(|c| c == '█'));
    }

    #[test]
    fn test_progress_bar_50() {
        let bar = progress_bar(50.0, 10, colors::GREEN);
        let stripped = strip_ansi(&bar);
        assert_eq!(stripped.chars().count(), 10);
        assert_eq!(stripped.chars().filter(|&c| c == '█').count(), 5);
        assert_eq!(stripped.chars().filter(|&c| c == '░').count(), 5);
    }

    fn strip_ansi(s: &str) -> String {
        let mut result = String::new();
        let mut in_escape = false;
        for c in s.chars() {
            if c == '\x1b' {
                in_escape = true;
            } else if in_escape {
                if c == 'm' {
                    in_escape = false;
                }
            } else {
                result.push(c);
            }
        }
        result
    }
}
