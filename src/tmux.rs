use std::process::Command;

const SESSION_NAME: &str = "codex";

/// Check if tmux is available
pub fn is_available() -> bool {
    Command::new("tmux")
        .arg("-V")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if we're already inside a tmux session
pub fn is_inside_tmux() -> bool {
    std::env::var("TMUX").is_ok()
}

/// Launch a tmux session with Codex in the main pane and HUD in the bottom pane
pub fn launch(codex_args: &[String], hud_height: u16) -> Result<(), String> {
    let hud_binary = std::env::current_exe()
        .map_err(|e| format!("Cannot find self: {}", e))?
        .to_string_lossy()
        .to_string();

    let codex_cmd = build_codex_command(codex_args);
    let hud_cmd = format!("{} --watch", hud_binary);

    // Kill any existing session with the same name
    let _ = Command::new("tmux")
        .args(["kill-session", "-t", SESSION_NAME])
        .output();

    // Create new session running codex
    let status = Command::new("tmux")
        .args([
            "new-session",
            "-d",
            "-s",
            SESSION_NAME,
            "-x",
            "200",
            "-y",
            "50",
            &codex_cmd,
        ])
        .status()
        .map_err(|e| format!("Failed to create tmux session: {}", e))?;

    if !status.success() {
        return Err("tmux new-session failed".to_string());
    }

    // Split a bottom pane for the HUD
    let status = Command::new("tmux")
        .args([
            "split-window",
            "-t",
            SESSION_NAME,
            "-v",
            "-l",
            &hud_height.to_string(),
            &hud_cmd,
        ])
        .status()
        .map_err(|e| format!("Failed to split tmux pane: {}", e))?;

    if !status.success() {
        return Err("tmux split-window failed".to_string());
    }

    // Focus the main (codex) pane
    let _ = Command::new("tmux")
        .args(["select-pane", "-t", &format!("{}:0.0", SESSION_NAME)])
        .status();

    // Attach to the session
    let status = Command::new("tmux")
        .args(["attach-session", "-t", SESSION_NAME])
        .status()
        .map_err(|e| format!("Failed to attach to tmux session: {}", e))?;

    if !status.success() {
        return Err("tmux attach failed".to_string());
    }

    Ok(())
}

/// Attach HUD pane to an existing tmux session
pub fn attach_hud(target_session: &str, hud_height: u16) -> Result<(), String> {
    let hud_binary = std::env::current_exe()
        .map_err(|e| format!("Cannot find self: {}", e))?
        .to_string_lossy()
        .to_string();

    let hud_cmd = format!("{} --watch", hud_binary);

    let status = Command::new("tmux")
        .args([
            "split-window",
            "-t",
            target_session,
            "-v",
            "-l",
            &hud_height.to_string(),
            &hud_cmd,
        ])
        .status()
        .map_err(|e| format!("Failed to split pane: {}", e))?;

    if !status.success() {
        return Err("tmux split-window failed".to_string());
    }

    // Focus back to the main pane
    let _ = Command::new("tmux")
        .args(["select-pane", "-t", &format!("{}:0.0", target_session)])
        .status();

    Ok(())
}

fn build_codex_command(args: &[String]) -> String {
    let mut parts = vec!["codex".to_string()];
    parts.extend(args.iter().cloned());
    parts.join(" ")
}
