use serde::Deserialize;
use std::path::PathBuf;

/// HUD configuration loaded from ~/.codex-hud/config.toml
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct HudConfig {
    /// Show tools activity line
    pub show_tools: bool,
    /// Show plan/todos line
    pub show_plan: bool,
    /// Show environment info line (MCP, rules counts)
    pub show_environment: bool,
    /// Bar width in characters
    pub bar_width: usize,
    /// Compact single-line mode
    pub compact: bool,
}

impl Default for HudConfig {
    fn default() -> Self {
        Self {
            show_tools: true,
            show_plan: true,
            show_environment: false,
            bar_width: 10,
            compact: false,
        }
    }
}

impl HudConfig {
    pub fn load() -> Self {
        let path = config_path();
        match std::fs::read_to_string(&path) {
            Ok(content) => toml::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }
}

fn config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".codex-hud").join("config.toml")
}
