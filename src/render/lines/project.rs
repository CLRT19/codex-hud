use crate::render::colors;
use crate::types::RolloutState;

/// Render: [model effort | plan_type] │ project git:(branch*)
pub fn render(state: &RolloutState, term_width: u16) -> String {
    let _ = term_width; // reserved for truncation logic

    let model = state.model.as_deref().unwrap_or("unknown");
    let effort = state.reasoning_effort.as_deref().unwrap_or("");
    let plan_type = state
        .plan_type
        .as_deref()
        .map(capitalize)
        .unwrap_or_default();

    // Build bracket: [model effort | plan]
    let model_part = if effort.is_empty() {
        model.to_string()
    } else {
        format!("{} {}", model, effort)
    };

    let bracket = if plan_type.is_empty() {
        format!("[{}]", model_part)
    } else {
        format!("[{} | {}]", model_part, plan_type)
    };

    let colored_bracket = colors::bold_colored(&bracket, colors::CYAN);

    // Build project + git part
    let project = state.project_name().unwrap_or_default();
    let git = state
        .git_branch()
        .map(|b| {
            let dirty = ""; // We'll get dirty state from git.rs if needed
            format!("git:({}{})", b, dirty)
        })
        .unwrap_or_default();

    let right = if git.is_empty() {
        colors::bold(&project)
    } else {
        format!("{} {}", colors::bold(&project), colors::colored(&git, colors::MAGENTA))
    };

    if right.is_empty() {
        colored_bracket
    } else {
        format!("{} {} {}", colored_bracket, colors::dim("│"), right)
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}
