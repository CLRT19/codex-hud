use std::path::PathBuf;

/// Find the most recent active rollout JSONL file in ~/.codex/sessions/
pub fn find_latest_rollout() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    let sessions_dir = PathBuf::from(&home).join(".codex").join("sessions");

    if !sessions_dir.exists() {
        return None;
    }

    find_newest_rollout_in(&sessions_dir)
}

fn find_newest_rollout_in(sessions_dir: &PathBuf) -> Option<PathBuf> {
    // Walk YYYY/MM/DD directories in reverse chronological order
    let mut year_dirs: Vec<_> = std::fs::read_dir(sessions_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .collect();
    year_dirs.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

    for year in &year_dirs {
        let mut month_dirs: Vec<_> = std::fs::read_dir(year.path())
            .ok()?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .collect();
        month_dirs.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

        for month in &month_dirs {
            let mut day_dirs: Vec<_> = std::fs::read_dir(month.path())
                .ok()?
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                .collect();
            day_dirs.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

            for day in &day_dirs {
                if let Some(rollout) = find_newest_rollout_file(&day.path()) {
                    return Some(rollout);
                }
            }
        }
    }

    None
}

fn find_newest_rollout_file(day_dir: &PathBuf) -> Option<PathBuf> {
    let mut rollout_files: Vec<_> = std::fs::read_dir(day_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("rollout-")
                && e.file_name().to_string_lossy().ends_with(".jsonl")
        })
        .collect();

    // Sort by modification time, newest first
    rollout_files.sort_by(|a, b| {
        let a_time = a.metadata().and_then(|m| m.modified()).ok();
        let b_time = b.metadata().and_then(|m| m.modified()).ok();
        b_time.cmp(&a_time)
    });

    rollout_files.first().map(|e| e.path())
}

/// Watch for new rollout files appearing (for session transitions)
pub fn find_rollout_for_date(year: &str, month: &str, day: &str) -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    let day_dir = PathBuf::from(&home)
        .join(".codex")
        .join("sessions")
        .join(year)
        .join(month)
        .join(day);

    find_newest_rollout_file(&day_dir)
}
