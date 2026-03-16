use crossterm::style::{Color, Stylize};

pub const GREEN: Color = Color::Rgb { r: 76, g: 191, b: 115 };
pub const YELLOW: Color = Color::Rgb { r: 230, g: 190, b: 60 };
pub const RED: Color = Color::Rgb { r: 220, g: 60, b: 60 };
pub const BLUE: Color = Color::Rgb { r: 100, g: 149, b: 237 };
pub const DIM: Color = Color::Rgb { r: 100, g: 100, b: 100 };
pub const WHITE: Color = Color::Rgb { r: 220, g: 220, b: 220 };
pub const CYAN: Color = Color::Rgb { r: 80, g: 200, b: 200 };
pub const MAGENTA: Color = Color::Rgb { r: 180, g: 100, b: 220 };

/// Get color based on context/usage percentage thresholds
pub fn threshold_color(percent: f64) -> Color {
    if percent < 70.0 {
        GREEN
    } else if percent < 85.0 {
        YELLOW
    } else {
        RED
    }
}

/// Color a string with the given color
pub fn colored(text: &str, color: Color) -> String {
    text.with(color).to_string()
}

/// Color a string as dim/muted
pub fn dim(text: &str) -> String {
    text.with(DIM).to_string()
}

/// Bold a string
pub fn bold(text: &str) -> String {
    text.bold().to_string()
}

/// Bold + colored
pub fn bold_colored(text: &str, color: Color) -> String {
    text.bold().with(color).to_string()
}
