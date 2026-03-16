use std::process::Command;

#[derive(Debug, Clone, Default)]
pub struct GitStatus {
    pub branch: Option<String>,
    pub dirty: bool,
    pub ahead: u32,
    pub behind: u32,
}

impl GitStatus {
    /// Get git status for the given working directory
    pub fn for_dir(cwd: &str) -> Self {
        let branch = get_branch(cwd);
        let dirty = is_dirty(cwd);
        let (ahead, behind) = get_ahead_behind(cwd);
        Self {
            branch,
            dirty,
            ahead,
            behind,
        }
    }

    /// Format as git:(branch*) ↑1↓2
    pub fn display(&self) -> Option<String> {
        let branch = self.branch.as_ref()?;
        let mut s = format!("git:({}{})", branch, if self.dirty { "*" } else { "" });
        if self.ahead > 0 {
            s.push_str(&format!(" ↑{}", self.ahead));
        }
        if self.behind > 0 {
            s.push_str(&format!(" ↓{}", self.behind));
        }
        Some(s)
    }
}

fn get_branch(cwd: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(cwd)
        .output()
        .ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

fn is_dirty(cwd: &str) -> bool {
    Command::new("git")
        .args(["diff", "--quiet", "HEAD"])
        .current_dir(cwd)
        .status()
        .map(|s| !s.success())
        .unwrap_or(false)
}

fn get_ahead_behind(cwd: &str) -> (u32, u32) {
    let output = Command::new("git")
        .args(["rev-list", "--left-right", "--count", "HEAD...@{upstream}"])
        .current_dir(cwd)
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let text = String::from_utf8_lossy(&o.stdout);
            let parts: Vec<&str> = text.trim().split('\t').collect();
            if parts.len() == 2 {
                let ahead = parts[0].parse().unwrap_or(0);
                let behind = parts[1].parse().unwrap_or(0);
                (ahead, behind)
            } else {
                (0, 0)
            }
        }
        _ => (0, 0),
    }
}
