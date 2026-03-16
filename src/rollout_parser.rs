use crate::types::*;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::Path;

/// Streaming JSONL parser that tracks byte offset for incremental reads
pub struct RolloutParser {
    pub state: RolloutState,
    seek_offset: u64,
}

impl RolloutParser {
    pub fn new() -> Self {
        Self {
            state: RolloutState::new(),
            seek_offset: 0,
        }
    }

    /// Parse only new lines since last read. Returns true if state changed.
    pub fn parse_new_lines(&mut self, path: &Path) -> bool {
        let file = match std::fs::File::open(path) {
            Ok(f) => f,
            Err(_) => return false,
        };

        let file_len = file.metadata().map(|m| m.len()).unwrap_or(0);
        if file_len <= self.seek_offset {
            return false;
        }

        let mut reader = BufReader::new(file);
        if self.seek_offset > 0 {
            if reader.seek(SeekFrom::Start(self.seek_offset)).is_err() {
                return false;
            }
        }

        let mut changed = false;
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(n) => {
                    self.seek_offset += n as u64;
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        if self.parse_line(trimmed) {
                            changed = true;
                        }
                    }
                }
                Err(_) => break,
            }
        }

        changed
    }

    /// Parse from the beginning (full re-parse)
    pub fn parse_full(&mut self, path: &Path) -> bool {
        self.seek_offset = 0;
        self.state = RolloutState::new();
        self.parse_new_lines(path)
    }

    fn parse_line(&mut self, line: &str) -> bool {
        let event: RolloutEvent = match serde_json::from_str(line) {
            Ok(e) => e,
            Err(_) => return false,
        };

        match event.event_type.as_str() {
            "session_meta" => self.handle_session_meta(&event.data),
            "turn_context" => self.handle_turn_context(&event.data),
            "token_count" => self.handle_token_count(&event.data),
            "function_call" => self.handle_function_call(&event.data),
            "function_call_output" => self.handle_function_call_output(&event.data),
            _ => false,
        }
    }

    fn handle_session_meta(&mut self, data: &serde_json::Value) -> bool {
        if let Ok(meta) = serde_json::from_value::<SessionMeta>(data.clone()) {
            self.state.session_meta = Some(meta);
            true
        } else {
            false
        }
    }

    fn handle_turn_context(&mut self, data: &serde_json::Value) -> bool {
        if let Ok(ctx) = serde_json::from_value::<TurnContext>(data.clone()) {
            if let Some(m) = ctx.model {
                self.state.model = Some(m);
            }
            if let Some(e) = ctx.reasoning_effort {
                self.state.reasoning_effort = Some(e);
            }
            true
        } else {
            false
        }
    }

    fn handle_token_count(&mut self, data: &serde_json::Value) -> bool {
        if let Ok(tc) = serde_json::from_value::<TokenCount>(data.clone()) {
            if let Some(t) = tc.total_token_usage {
                self.state.total_tokens = Some(t);
            }
            if let Some(w) = tc.model_context_window {
                self.state.context_window = Some(w);
            }
            if let Some(i) = tc.input_tokens {
                self.state.input_tokens = Some(i);
            }
            if let Some(o) = tc.output_tokens {
                self.state.output_tokens = Some(o);
            }

            if let Some(rl) = tc.rate_limits {
                if let Some(pt) = rl.plan_type {
                    self.state.plan_type = Some(pt);
                }
                if let Some(p) = rl.primary {
                    self.state.primary_usage = p.used_percent;
                    self.state.primary_resets_at = p.resets_at;
                    self.state.primary_window_secs = p.window_seconds;
                }
                if let Some(s) = rl.secondary {
                    self.state.secondary_usage = s.used_percent;
                    self.state.secondary_resets_at = s.resets_at;
                }
            }
            true
        } else {
            false
        }
    }

    fn handle_function_call(&mut self, data: &serde_json::Value) -> bool {
        if let Ok(fc) = serde_json::from_value::<FunctionCall>(data.clone()) {
            let call_id = match fc.call_id {
                Some(id) => id,
                None => return false,
            };
            let name = fc.name.unwrap_or_else(|| "unknown".to_string());

            // Extract a short summary from args
            let args_summary = extract_args_summary(&name, &fc.args);

            // Check if this is an update_plan call
            if name == "update_plan" {
                if let Some(args) = &fc.args {
                    self.parse_plan_update(args);
                }
            }

            self.state.tools.insert(
                call_id,
                ToolEntry {
                    name,
                    args_summary,
                    start_time: std::time::Instant::now(),
                    completed: false,
                },
            );
            true
        } else {
            false
        }
    }

    fn handle_function_call_output(&mut self, data: &serde_json::Value) -> bool {
        if let Ok(fco) = serde_json::from_value::<FunctionCallOutput>(data.clone()) {
            if let Some(call_id) = fco.call_id {
                if let Some(tool) = self.state.tools.get_mut(&call_id) {
                    tool.completed = true;
                    let count = self.state.tool_counts.entry(tool.name.clone()).or_insert(0);
                    *count += 1;
                }
            }
            true
        } else {
            false
        }
    }

    fn parse_plan_update(&mut self, args: &serde_json::Value) {
        // update_plan args: { "steps": [{ "step": "...", "status": "..." }] }
        if let Some(steps) = args.get("steps").and_then(|s| s.as_array()) {
            let parsed: Vec<PlanStep> = steps
                .iter()
                .filter_map(|s| serde_json::from_value(s.clone()).ok())
                .collect();
            if !parsed.is_empty() {
                self.state.plan_steps = parsed;
            }
        }
    }
}

fn extract_args_summary(name: &str, args: &Option<serde_json::Value>) -> String {
    let args = match args {
        Some(a) => a,
        None => return String::new(),
    };

    match name {
        "exec_command" | "shell" => {
            args.get("command")
                .or_else(|| args.get("cmd"))
                .and_then(|v| v.as_str())
                .map(|s| truncate(s, 40))
                .unwrap_or_default()
        }
        "write_file" | "create_file" => {
            args.get("path")
                .or_else(|| args.get("file_path"))
                .and_then(|v| v.as_str())
                .map(|s| short_path(s))
                .unwrap_or_default()
        }
        "read_file" => {
            args.get("path")
                .or_else(|| args.get("file_path"))
                .and_then(|v| v.as_str())
                .map(|s| short_path(s))
                .unwrap_or_default()
        }
        _ => String::new(),
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max - 1])
    }
}

fn short_path(s: &str) -> String {
    std::path::Path::new(s)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_parse_session_meta() {
        let mut parser = RolloutParser::new();
        let tmp = create_temp_rollout(&[
            r#"{"type":"session_meta","id":"sess-123","cwd":"/home/user/project","cli_version":"1.0.0","git":{"branch":"main"}}"#,
        ]);

        assert!(parser.parse_full(tmp.path()));
        assert!(parser.state.session_meta.is_some());
        let meta = parser.state.session_meta.as_ref().unwrap();
        assert_eq!(meta.id.as_deref(), Some("sess-123"));
        assert_eq!(meta.cwd.as_deref(), Some("/home/user/project"));
        assert_eq!(parser.state.git_branch().as_deref(), Some("main"));
    }

    #[test]
    fn test_parse_token_count() {
        let mut parser = RolloutParser::new();
        let tmp = create_temp_rollout(&[
            r#"{"type":"token_count","total_token_usage":50000,"model_context_window":200000,"input_tokens":30000,"output_tokens":20000,"rate_limits":{"plan_type":"plus","primary":{"used_percent":25.0,"resets_at":"2025-01-01T05:00:00Z","window_seconds":18000},"secondary":{"used_percent":5.0,"resets_at":"2025-01-07T00:00:00Z"}}}"#,
        ]);

        assert!(parser.parse_full(tmp.path()));
        assert_eq!(parser.state.total_tokens, Some(50000));
        assert_eq!(parser.state.context_window, Some(200000));
        assert_eq!(parser.state.plan_type.as_deref(), Some("plus"));
        assert_eq!(parser.state.primary_usage, Some(25.0));
        assert_eq!(parser.state.secondary_usage, Some(5.0));
        assert!((parser.state.context_percent().unwrap() - 25.0).abs() < 0.1);
    }

    #[test]
    fn test_parse_function_calls() {
        let mut parser = RolloutParser::new();
        let tmp = create_temp_rollout(&[
            r#"{"type":"function_call","call_id":"call-1","name":"exec_command","arguments":{"command":"ls -la"}}"#,
            r#"{"type":"function_call","call_id":"call-2","name":"read_file","arguments":{"path":"/foo/bar.rs"}}"#,
            r#"{"type":"function_call_output","call_id":"call-1"}"#,
        ]);

        assert!(parser.parse_full(tmp.path()));
        assert_eq!(parser.state.tools.len(), 2);
        assert_eq!(parser.state.running_tools().len(), 1); // call-2 still running
        assert_eq!(*parser.state.tool_counts.get("exec_command").unwrap(), 1);
    }

    #[test]
    fn test_incremental_parse() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("rollout.jsonl");

        // Write initial lines
        {
            let mut f = std::fs::File::create(&path).unwrap();
            writeln!(f, r#"{{"type":"session_meta","id":"s1","cwd":"/tmp"}}"#).unwrap();
        }

        let mut parser = RolloutParser::new();
        assert!(parser.parse_new_lines(&path));
        assert!(parser.state.session_meta.is_some());

        // Append more data
        {
            let mut f = std::fs::OpenOptions::new().append(true).open(&path).unwrap();
            writeln!(f, r#"{{"type":"turn_context","model":"gpt-5.4","reasoning_effort":"high"}}"#).unwrap();
        }

        assert!(parser.parse_new_lines(&path));
        assert_eq!(parser.state.model.as_deref(), Some("gpt-5.4"));
    }

    fn create_temp_rollout(lines: &[&str]) -> tempfile::NamedTempFile {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        for line in lines {
            writeln!(tmp, "{}", line).unwrap();
        }
        tmp.flush().unwrap();
        tmp
    }
}
