//! JSON files in the app data dir: the overview cache and the run history.

use serde::{Serialize, de::DeserializeOwned};
use std::{fs, io, path::PathBuf};
use steward_core::model::{Overview, RunResult};

pub const MAX_RUNS: usize = 50;

#[derive(Debug, Clone)]
pub struct Store {
    dir: PathBuf,
}

impl Store {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    pub fn load_overview(&self) -> Option<Overview> {
        self.read("overview.json")
    }

    pub fn save_overview(&self, overview: &Overview) -> io::Result<()> {
        self.write("overview.json", overview)
    }

    /// What the cached overview was measured against (see `AppState::fingerprint`); `None` for old caches.
    pub fn load_fingerprint(&self) -> Option<String> {
        fs::read_to_string(self.dir.join("overview.fingerprint")).ok()
    }

    pub fn save_fingerprint(&self, fingerprint: &str) -> io::Result<()> {
        fs::create_dir_all(&self.dir)?;
        fs::write(self.dir.join("overview.fingerprint"), fingerprint)
    }

    /// Newest first.
    pub fn load_runs(&self) -> Vec<RunResult> {
        self.read("runs.json").unwrap_or_default()
    }

    /// Prepends `run` and keeps the last `MAX_RUNS`.
    pub fn push_run(&self, run: RunResult) -> io::Result<()> {
        let mut runs = self.load_runs();
        runs.insert(0, run);
        runs.truncate(MAX_RUNS);
        self.write("runs.json", &runs)
    }

    // A corrupt or missing file is a cache miss, never an error: the next write replaces it.
    fn read<T: DeserializeOwned>(&self, name: &str) -> Option<T> {
        serde_json::from_slice(&fs::read(self.dir.join(name)).ok()?).ok()
    }

    fn write(&self, name: &str, value: &impl Serialize) -> io::Result<()> {
        fs::create_dir_all(&self.dir)?;
        let path = self.dir.join(name);
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, serde_json::to_vec_pretty(value)?)?;
        fs::rename(&tmp, &path)
    }
}

#[cfg(test)]
#[path = "store_tests.rs"]
mod tests;
