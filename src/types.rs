use serde::Deserialize;
use std::collections::HashMap;

/// Top-level rollout JSONL event wrapper
#[derive(Debug, Deserialize)]
pub struct RolloutEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(flatten)]
    pub data: serde_json::Value,
}

/// session_meta event — first line of rollout
#[derive(Debug, Clone, Deserialize)]
pub struct SessionMeta {
    pub id: Option<String>,
    pub cwd: Option<String>,
    pub cli_version: Option<String>,
    pub git: Option<GitInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitInfo {
    pub branch: Option<String>,
}

/// turn_context event — emitted each turn
#[derive(Debug, Clone, Deserialize)]
pub struct TurnContext {
    pub model: Option<String>,
    pub reasoning_effort: Option<String>,
}

/// token_count event — token usage + rate limits
#[derive(Debug, Clone, Deserialize)]
pub struct TokenCount {
    pub total_token_usage: Option<u64>,
    pub model_context_window: Option<u64>,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub rate_limits: Option<RateLimits>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RateLimits {
    pub plan_type: Option<String>,
    pub primary: Option<RateWindow>,
    pub secondary: Option<RateWindow>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RateWindow {
    pub used_percent: Option<f64>,
    pub resets_at: Option<String>,
    pub window_seconds: Option<u64>,
}

/// function_call event — tool invocation
#[derive(Debug, Clone, Deserialize)]
pub struct FunctionCall {
    pub call_id: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "arguments")]
    pub args: Option<serde_json::Value>,
}

/// function_call_output event — tool result
#[derive(Debug, Clone, Deserialize)]
pub struct FunctionCallOutput {
    pub call_id: Option<String>,
}

/// Tracked tool state
#[derive(Debug, Clone)]
pub struct ToolEntry {
    pub name: String,
    pub args_summary: String,
    pub start_time: std::time::Instant,
    pub completed: bool,
}

/// Plan step from update_plan function call
#[derive(Debug, Clone, Deserialize)]
pub struct PlanStep {
    pub step: String,
    pub status: String,
}

/// Accumulated state from parsing rollout events
#[derive(Debug, Clone)]
pub struct RolloutState {
    pub session_meta: Option<SessionMeta>,
    pub model: Option<String>,
    pub reasoning_effort: Option<String>,
    pub plan_type: Option<String>,

    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
    pub context_window: Option<u64>,

    pub primary_usage: Option<f64>,
    pub primary_resets_at: Option<String>,
    pub primary_window_secs: Option<u64>,
    pub secondary_usage: Option<f64>,
    pub secondary_resets_at: Option<String>,

    /// call_id -> ToolEntry
    pub tools: HashMap<String, ToolEntry>,
    /// Completed tool name counts
    pub tool_counts: HashMap<String, u32>,

    pub plan_steps: Vec<PlanStep>,
}

impl RolloutState {
    pub fn new() -> Self {
        Self {
            session_meta: None,
            model: None,
            reasoning_effort: None,
            plan_type: None,
            input_tokens: None,
            output_tokens: None,
            total_tokens: None,
            context_window: None,
            primary_usage: None,
            primary_resets_at: None,
            primary_window_secs: None,
            secondary_usage: None,
            secondary_resets_at: None,
            tools: HashMap::new(),
            tool_counts: HashMap::new(),
            plan_steps: Vec::new(),
        }
    }

    pub fn context_percent(&self) -> Option<f64> {
        match (self.total_tokens, self.context_window) {
            (Some(used), Some(window)) if window > 0 => {
                Some((used as f64 / window as f64) * 100.0)
            }
            _ => None,
        }
    }

    pub fn running_tools(&self) -> Vec<&ToolEntry> {
        self.tools
            .values()
            .filter(|t| !t.completed)
            .collect()
    }

    pub fn plan_progress(&self) -> (usize, usize) {
        let done = self.plan_steps.iter().filter(|s| s.status == "completed").count();
        (done, self.plan_steps.len())
    }

    pub fn project_name(&self) -> Option<String> {
        self.session_meta.as_ref().and_then(|m| {
            m.cwd.as_ref().and_then(|cwd| {
                std::path::Path::new(cwd)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
            })
        })
    }

    pub fn git_branch(&self) -> Option<String> {
        self.session_meta
            .as_ref()
            .and_then(|m| m.git.as_ref())
            .and_then(|g| g.branch.clone())
    }
}

impl Default for RolloutState {
    fn default() -> Self {
        Self::new()
    }
}
