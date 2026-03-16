use crate::codex_config::CodexConfig;
use crate::render::colors;

/// Render: 3 MCP servers | 5 rules
pub fn render(codex_config: &CodexConfig) -> Option<String> {
    let mcp = codex_config.mcp_count();

    if mcp == 0 {
        return None;
    }

    let mut parts: Vec<String> = Vec::new();

    if mcp > 0 {
        parts.push(format!("{} MCP server{}", mcp, if mcp == 1 { "" } else { "s" }));
    }

    if parts.is_empty() {
        None
    } else {
        Some(colors::dim(&parts.join(" | ")))
    }
}
