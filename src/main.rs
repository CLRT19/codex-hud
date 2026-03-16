mod codex_config;
mod config;
mod git;
mod render;
mod rollout_parser;
mod session_finder;
mod tmux;
mod types;
mod watcher;

use clap::Parser;

/// Real-time HUD for OpenAI Codex CLI
#[derive(Parser, Debug)]
#[command(name = "codex-hud", version, about)]
struct Cli {
    /// Run in standalone watch mode (no tmux, just render HUD)
    #[arg(long)]
    watch: bool,

    /// Attach HUD pane to an existing tmux session
    #[arg(long)]
    attach: Option<String>,

    /// Height of the HUD pane in lines
    #[arg(long, default_value = "4")]
    height: u16,

    /// Arguments to pass to codex (after --)
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    codex_args: Vec<String>,
}

fn main() {
    let cli = Cli::parse();
    let config = config::HudConfig::load();

    if cli.watch {
        // Standalone watch mode
        watcher::run(&config);
        return;
    }

    if let Some(session) = &cli.attach {
        // Attach to existing tmux session
        match tmux::attach_hud(session, cli.height) {
            Ok(()) => {}
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    // Default: launch tmux with codex + HUD
    if !tmux::is_available() {
        eprintln!("tmux not found. Falling back to --watch mode.");
        eprintln!("Run Codex in another terminal, then use: codex-hud --watch");
        eprintln!();
        watcher::run(&config);
        return;
    }

    if tmux::is_inside_tmux() {
        // Already in tmux, just attach HUD pane
        let session = std::env::var("TMUX")
            .ok()
            .and_then(|t| t.split(',').last().map(|s| s.to_string()))
            .unwrap_or_else(|| "0".to_string());

        // Get session name from tmux
        let session_name = std::process::Command::new("tmux")
            .args(["display-message", "-p", "#{session_name}"])
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
                } else {
                    None
                }
            })
            .unwrap_or(session);

        match tmux::attach_hud(&session_name, cli.height) {
            Ok(()) => {}
            Err(e) => {
                eprintln!("Error attaching HUD: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    match tmux::launch(&cli.codex_args, cli.height) {
        Ok(()) => {}
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
