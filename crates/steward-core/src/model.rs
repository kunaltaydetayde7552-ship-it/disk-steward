//! Serde types of the UI contract (see docs/API.md). All JSON keys are camelCase.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskStatus {
    pub volume: String,
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub used_bytes: u64,
    /// RFC3339, UTC.
    pub measured_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Toolchain,
    PackageCache,
    Store,
    Build,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Risk {
    None,
    CloneShared,
    RewrittenOften,
    BlockedByRule,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Target {
    pub id: String,
    pub path: String,
    pub kind: Kind,
    pub enabled: bool,
    pub allowed: bool,
    pub logical_bytes: u64,
    pub on_disk_bytes: u64,
    pub private_bytes: Option<u64>,
    pub total_files: u64,
    pub compressed_files: u64,
    pub risk: Risk,
    pub risk_detail: Option<String>,
}

/// Result of `measure::measure` (also the size part of a `Target`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Measure {
    pub logical_bytes: u64,
    pub on_disk_bytes: u64,
    pub private_bytes: Option<u64>,
    pub total_files: u64,
    pub compressed_files: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Savings {
    pub files: u64,
    pub original_bytes: u64,
    pub on_disk_bytes: u64,
    pub saved_bytes: u64,
    pub ratio: f64,
    pub quarantined: u64,
    pub last_run_day: Option<String>,
    /// Engine `savedBytes` minus the sum of `logicalBytes - onDiskBytes` over enabled targets, never negative.
    /// Not measured (`0`) when sizes were skipped. See docs/API.md for why the two disagree.
    #[serde(default)]
    pub unattributed_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewRoot {
    pub path: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preview {
    pub files: u64,
    pub bytes: u64,
    pub skipped: u64,
    pub roots: Vec<PreviewRoot>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunResult {
    pub started_at: String,
    pub duration_ms: u64,
    /// Free-space delta of the data volume (after - before); negative if space shrank.
    pub freed_bytes: i64,
    pub preview: Preview,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleAgent {
    pub label: String,
    pub loaded: bool,
    pub last_exit_status: Option<i32>,
    pub log_path: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DockerUsage {
    pub available: bool,
    pub images_bytes: u64,
    pub build_cache_bytes: u64,
    pub volumes_bytes: u64,
    pub reclaimable_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeasurementState {
    Fresh,
    Stale,
    Measuring,
    Never,
}

/// How trustworthy the sizes in an `Overview` are. `core` only produces `fresh` (just measured) or
/// `never` (`with_sizes = false`); the app adds `stale` and `measuring`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Measurement {
    pub state: MeasurementState,
    /// RFC3339, UTC; `null` when never measured.
    pub measured_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Overview {
    pub disk: DiskStatus,
    pub savings: Savings,
    pub targets: Vec<Target>,
    pub schedule: Vec<ScheduleAgent>,
    pub docker: DockerUsage,
    pub measurement: Measurement,
}
