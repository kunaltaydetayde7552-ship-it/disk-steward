//! Payloads that only exist in the shell (the rest are `steward_core::model` types). camelCase on the wire.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Starting,
    Compressing,
    Verifying,
    Done,
    Failed,
}

/// Payload of `run://progress`. Unknown values are `null`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunProgress {
    pub phase: Phase,
    pub files: Option<u64>,
    pub total_files: Option<u64>,
    pub bytes: Option<u64>,
    pub current_path: Option<String>,
}

/// Payload of `overview://failed`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Failure {
    pub code: &'static str,
    pub message: String,
}
