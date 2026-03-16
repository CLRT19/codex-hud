use crate::codex_config::CodexConfig;
use crate::config::HudConfig;
use crate::render;
use crate::rollout_parser::RolloutParser;
use crate::session_finder;
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

/// Main watch loop: find session → watch file → parse → render
pub fn run(config: &HudConfig) {
    let codex_config = CodexConfig::load();

    loop {
        // Find the latest rollout file
        let rollout_path = match find_rollout_with_retry() {
            Some(p) => p,
            None => {
                render_waiting();
                std::thread::sleep(Duration::from_secs(2));
                continue;
            }
        };

        eprintln!("Watching: {}", rollout_path.display());

        // Parse existing content
        let mut parser = RolloutParser::new();
        parser.parse_full(&rollout_path);
        render::render(&parser.state, config, &codex_config);

        // Watch for changes
        let (tx, rx) = mpsc::channel();

        let mut watcher = match notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        }) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("Failed to create file watcher: {}", e);
                // Fall back to polling
                poll_loop(&rollout_path, &mut parser, config, &codex_config);
                continue;
            }
        };

        // Watch the parent directory to also catch new rollout files
        let watch_dir = rollout_path.parent().unwrap_or(&rollout_path);
        if let Err(e) = watcher.watch(watch_dir, RecursiveMode::NonRecursive) {
            eprintln!("Failed to watch directory: {}", e);
            poll_loop(&rollout_path, &mut parser, config, &codex_config);
            continue;
        }

        // Event loop
        let tick = Duration::from_secs(1);
        loop {
            match rx.recv_timeout(tick) {
                Ok(event) => {
                    if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                        // Check if a newer rollout appeared
                        if let Some(newest) = session_finder::find_latest_rollout() {
                            if newest != rollout_path {
                                // New session started, break to outer loop
                                break;
                            }
                        }

                        if parser.parse_new_lines(&rollout_path) {
                            render::render(&parser.state, config, &codex_config);
                        }
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // Periodic re-render for time-based updates (durations, etc.)
                    render::render(&parser.state, config, &codex_config);
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    break;
                }
            }
        }
    }
}

/// Polling fallback when notify doesn't work
fn poll_loop(
    path: &PathBuf,
    parser: &mut RolloutParser,
    config: &HudConfig,
    codex_config: &CodexConfig,
) {
    loop {
        std::thread::sleep(Duration::from_millis(500));
        if parser.parse_new_lines(path) {
            render::render(&parser.state, config, codex_config);
        }

        // Check for newer session
        if let Some(newest) = session_finder::find_latest_rollout() {
            if &newest != path {
                return; // Break to outer loop for new session
            }
        }
    }
}

fn find_rollout_with_retry() -> Option<PathBuf> {
    // Try a few times with short delay
    for _ in 0..3 {
        if let Some(p) = session_finder::find_latest_rollout() {
            return Some(p);
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    None
}

fn render_waiting() {
    use crossterm::{cursor, terminal, ExecutableCommand};
    use std::io::Write;

    let mut stdout = std::io::stdout();
    let _ = stdout.execute(cursor::MoveTo(0, 0));
    let _ = stdout.execute(terminal::Clear(terminal::ClearType::All));
    let _ = writeln!(
        stdout,
        "{}",
        render::colors::dim("Waiting for Codex session...")
    );
    let _ = stdout.flush();
}
