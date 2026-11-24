//! Serializable error returned by every command: `{ code, message }`. The UI translates codes.

use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AppError {
    /// `engine_missing | target_blocked | clone_shared | config_io | run_in_progress | run_failed |
    /// schedule_failed | not_found | internal`
    pub code: &'static str,
    pub message: String,
}

pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn internal(message: impl ToString) -> Self {
        Self::new("internal", message.to_string())
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new("not_found", message)
    }

    /// An error string from `steward_core::engine`: the binary could not be started, or it failed.
    pub fn engine(message: String) -> Self {
        let code = if message.starts_with("cannot run ") {
            "engine_missing"
        } else {
            "run_failed"
        };
        Self::new(code, message)
    }

    /// An error string from reading or editing the engine config.
    pub fn config(message: impl Into<String>) -> Self {
        Self::new("config_io", message)
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        Self::internal(e)
    }
}
