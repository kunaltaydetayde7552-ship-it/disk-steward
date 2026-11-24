//! Turns the engine's `-v` log (stderr) into `run://progress` payloads.
//!
//! Lines used: `... INFO start ...`, `... INFO compressed <path> <orig> -> <disk>`, anything mentioning
//! "verif" (best effort: the engine's verify step is not logged at INFO in every version).

use crate::dto::{Phase, RunProgress};
use std::time::{Duration, Instant};

pub struct Tracker {
    total_files: Option<u64>,
    files: u64,
    bytes: u64,
    phase: Phase,
    min_gap: Duration,
    last_emit: Option<Instant>,
}

impl Tracker {
    pub fn new(total_files: Option<u64>, min_gap: Duration) -> Self {
        Self {
            total_files,
            files: 0,
            bytes: 0,
            phase: Phase::Starting,
            min_gap,
            last_emit: None,
        }
    }

    /// The payload to emit for `line`, or `None` when the line is irrelevant or throttled.
    pub fn on_line(&mut self, line: &str) -> Option<RunProgress> {
        let (phase, path) = self.classify(line)?;
        let changed = phase != self.phase;
        self.phase = phase;
        let quiet = self.last_emit.is_some_and(|t| t.elapsed() < self.min_gap);
        if quiet && !changed {
            return None;
        }
        self.last_emit = Some(Instant::now());
        Some(self.snapshot(phase, path))
    }

    pub fn snapshot(&self, phase: Phase, current_path: Option<String>) -> RunProgress {
        let counted = self.files > 0;
        RunProgress {
            phase,
            files: counted.then_some(self.files),
            total_files: self.total_files,
            bytes: counted.then_some(self.bytes),
            current_path,
        }
    }

    fn classify(&mut self, line: &str) -> Option<(Phase, Option<String>)> {
        if let Some((path, original)) = parse_compressed(line) {
            self.files += 1;
            self.bytes += original;
            return Some((Phase::Compressing, Some(path)));
        }
        if line.contains(" INFO start ") {
            return Some((Phase::Starting, None));
        }
        line.to_ascii_lowercase()
            .contains("verif")
            .then_some((Phase::Verifying, None))
    }
}

/// `... INFO compressed /a b/c.txt 400000 -> 4096` -> `("/a b/c.txt", 400000)`.
fn parse_compressed(line: &str) -> Option<(String, u64)> {
    let rest = line.split_once(" INFO compressed ")?.1;
    let (head, _) = rest.rsplit_once(" -> ")?;
    let (path, original) = head.rsplit_once(' ')?;
    Some((path.to_owned(), original.parse().ok()?))
}

#[cfg(test)]
#[path = "progress_tests.rs"]
mod tests;
