//! Event names and the emitter abstraction that keeps the app logic testable without a Tauri runtime.

use serde::Serialize;
use tauri::{AppHandle, Emitter};

pub const OVERVIEW_MEASURING: &str = "overview://measuring";
pub const OVERVIEW_UPDATED: &str = "overview://updated";
pub const OVERVIEW_FAILED: &str = "overview://failed";
pub const RUN_PROGRESS: &str = "run://progress";
pub const RUN_FINISHED: &str = "run://finished";
/// Sent by the tray menu item "Comprimir agora": the UI shows its confirmation, nothing is compressed by the click.
pub const TRAY_COMPRESS_NOW: &str = "tray://compress-now";

pub trait Emit: Send + Sync {
    fn emit_json(&self, event: &str, payload: serde_json::Value);
}

pub fn emit(out: &dyn Emit, event: &str, payload: &impl Serialize) {
    if let Ok(v) = serde_json::to_value(payload) {
        out.emit_json(event, v);
    }
}

/// Emits to every window, then refreshes the tray title when the free space may have changed.
pub struct AppEmitter(pub AppHandle);

impl Emit for AppEmitter {
    fn emit_json(&self, event: &str, payload: serde_json::Value) {
        // a failed emit only means no window is listening
        let _ = self.0.emit(event, payload);
        if event == OVERVIEW_UPDATED || event == RUN_FINISHED {
            crate::tray::refresh(&self.0);
        }
    }
}
