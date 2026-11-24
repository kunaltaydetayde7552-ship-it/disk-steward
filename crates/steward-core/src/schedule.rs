//! LaunchAgents of interest: auto-compress-apfs' own daily agent and the docker GC job.

use crate::model::ScheduleAgent;
use std::{path::Path, process::Command};

/// `(label, default log file relative to $HOME)`.
pub const AGENTS: [(&str, &str); 2] = [
    (
        "local.auto-compress-apfs",
        "Library/Logs/auto-compress-apfs/launchd.log",
    ),
    ("com.lucasbrito.docker-gc", "Library/Logs/docker-gc.log"),
];

pub fn list(home: &Path) -> Vec<ScheduleAgent> {
    AGENTS
        .iter()
        .map(|(label, log)| agent(label, &home.join(log).display().to_string()))
        .collect()
}

fn agent(label: &str, default_log: &str) -> ScheduleAgent {
    let listed = Command::new("/bin/launchctl")
        .args(["list", label])
        .output();
    let text = match listed {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).into_owned(),
        _ => {
            return ScheduleAgent {
                label: label.into(),
                loaded: false,
                last_exit_status: None,
                log_path: Some(default_log.into()),
            };
        }
    };
    ScheduleAgent {
        label: label.into(),
        loaded: true,
        last_exit_status: key(&text, "LastExitStatus").and_then(|v| v.parse().ok()),
        log_path: key(&text, "StandardOutPath").or_else(|| Some(default_log.into())),
    }
}

/// Value of `"Key" = value;` in `launchctl list <label>` output (quotes stripped).
pub fn key(text: &str, name: &str) -> Option<String> {
    let rest = text
        .lines()
        .find_map(|l| l.trim().strip_prefix(&format!("\"{name}\" = ")))?;
    Some(rest.trim_end_matches(';').trim_matches('"').to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIST: &str = include_str!("../tests/fixtures/launchctl_list.txt");

    #[test]
    fn parses_launchctl_list() {
        assert_eq!(key(LIST, "LastExitStatus").as_deref(), Some("0"));
        assert_eq!(
            key(LIST, "StandardOutPath").as_deref(),
            Some("/Users/dev/Library/Logs/docker-gc.log")
        );
        assert_eq!(key(LIST, "Missing"), None);
    }

    #[test]
    fn unloaded_agent_reports_default_log() {
        let a = agent("local.steward-test-nonexistent", "/x/log");
        assert_eq!(
            (a.loaded, a.last_exit_status, a.log_path.as_deref()),
            (false, None, Some("/x/log"))
        );
    }
}
