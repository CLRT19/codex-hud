use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

/// Parsed ~/.codex/config.toml for MCP server info and model defaults
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct CodexConfig {
    pub model: Option<String>,
    #[serde(rename = "mcp_servers")]
    pub mcp_servers: Option<HashMap<String, serde_json::Value>>,
}

impl CodexConfig {
    pub fn load() -> Self {
        let path = codex_config_path();
        match std::fs::read_to_string(&path) {
            Ok(content) => toml::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn mcp_count(&self) -> usize {
        self.mcp_servers.as_ref().map_or(0, |m| m.len())
    }
}

fn codex_config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".codex").join("config.toml")
}
