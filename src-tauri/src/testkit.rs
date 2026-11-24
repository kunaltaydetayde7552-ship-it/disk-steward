//! Test fixtures: a stub engine in a temp dir (never the real engine, never the real HOME) and an event recorder.

use crate::{events::Emit, state::AppState, store::Store};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    sync::{Arc, Condvar, Mutex},
    time::{Duration, Instant},
};
use steward_core::engine::Engine;

const STUB: &str = r#"#!/bin/sh
dir=$(dirname "$0")
echo "$@" >> "$dir/calls.log"
case "$1" in
  stat) cat "$dir/stat.txt" ;;
  compress)
    case "$*" in
      *--apply*)
        trap 'kill $! 2>/dev/null; exit 130' INT
        echo "2026-11-24T16:54:46.056-03:00 INFO start stub" >&2
        echo "2026-11-24T16:54:46.148-03:00 INFO compressed /tmp/a b.txt 400000 -> 4096" >&2
        echo "2026-11-24T16:54:46.155-03:00 INFO compressed /tmp/c.txt 1000 -> 4096" >&2
        if [ -f "$dir/slow" ]; then touch "$dir/started"; sleep 10 & wait $!; fi
        echo "2026-11-24T16:54:46.160-03:00 WARN something odd" >&2
        echo "2026-11-24T16:54:46.172-03:00 INFO summary found=2" >&2 ;;
      *) cat "$dir/preview.txt" ;;
    esac ;;
  install-launchd|uninstall-launchd) echo ok ;;
  *) echo "unknown $1" >&2; exit 2 ;;
esac
"#;

pub struct Fixture {
    pub dir: tempfile::TempDir,
    pub state: Arc<AppState>,
}

impl Fixture {
    pub fn new() -> Self {
        Self::with_config("patterns = [\"~/.npm\"]\n\n[trash]\npatterns = []\n")
    }

    pub fn with_config(config: &str) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let (bin, cfg, home) = (
            root.join("engine.sh"),
            root.join("config.toml"),
            root.join("home"),
        );
        fs::create_dir(&home).unwrap();
        fs::write(&bin, STUB).unwrap();
        fs::set_permissions(&bin, fs::Permissions::from_mode(0o755)).unwrap();
        fs::write(&cfg, config).unwrap();
        fs::write(
            root.join("stat.txt"),
            include_str!("../../crates/steward-core/tests/fixtures/stat.txt"),
        )
        .unwrap();
        fs::write(
            root.join("preview.txt"),
            include_str!("../../crates/steward-core/tests/fixtures/preview.txt"),
        )
        .unwrap();
        let engine = Engine {
            bin,
            config: cfg,
            home,
        };
        assert!(
            engine.config.starts_with(root)
                && engine.home.starts_with(root)
                && engine.bin.starts_with(root)
        );
        let state = Arc::new(AppState::new(engine, Store::new(root.join("appdata"))));
        Self { dir, state }
    }

    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    /// Every engine invocation so far, one line each.
    pub fn calls(&self) -> Vec<String> {
        fs::read_to_string(self.path().join("calls.log"))
            .unwrap_or_default()
            .lines()
            .map(str::to_owned)
            .collect()
    }

    pub fn config_text(&self) -> String {
        fs::read_to_string(&self.state.engine.config).unwrap()
    }

    pub fn file(&self, name: &str) -> PathBuf {
        self.path().join(name)
    }
}

#[derive(Default)]
pub struct Recorder {
    events: Mutex<Vec<(String, serde_json::Value)>>,
    changed: Condvar,
}

impl Emit for Recorder {
    fn emit_json(&self, event: &str, payload: serde_json::Value) {
        self.events
            .lock()
            .unwrap()
            .push((event.to_owned(), payload));
        self.changed.notify_all();
    }
}

impl Recorder {
    pub fn all(&self) -> Vec<(String, serde_json::Value)> {
        self.events.lock().unwrap().clone()
    }

    pub fn payloads(&self, event: &str) -> Vec<serde_json::Value> {
        self.all()
            .into_iter()
            .filter(|(e, _)| e == event)
            .map(|(_, p)| p)
            .collect()
    }

    /// Blocks until `event` has been emitted (panics after 30 s).
    pub fn wait_for(&self, event: &str) {
        let deadline = Instant::now() + Duration::from_secs(30);
        let mut guard = self.events.lock().unwrap();
        while !guard.iter().any(|(e, _)| e == event) {
            let left = deadline
                .checked_duration_since(Instant::now())
                .expect("timed out waiting for event");
            guard = self.changed.wait_timeout(guard, left).unwrap().0;
        }
    }
}

pub fn wait_until(what: &str, cond: impl Fn() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(30);
    while !cond() {
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(20));
    }
}
