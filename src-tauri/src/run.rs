//! One engine run at a time: guard, progress events, cancellation, history.

use crate::{
    dto::{Phase, RunProgress},
    error::{AppError, AppResult},
    events::{self, Emit},
    progress::Tracker,
    state::AppState,
};
use std::{sync::PoisonError, time::Duration};
use steward_core::{model::RunResult, overview};

const PROGRESS_GAP: Duration = Duration::from_millis(100);
const CANCELLED: &str = "cancelled";

/// Releases the run slot when the run ends, however it ends.
struct SlotGuard<'a>(&'a AppState);

impl Drop for SlotGuard<'_> {
    fn drop(&mut self) {
        *self.0.run.lock().unwrap_or_else(PoisonError::into_inner) = Default::default();
    }
}

impl AppState {
    /// Blocks until the engine finishes. Emits `run://progress` while it runs and `run://finished` with the result.
    pub fn apply(&self, out: &dyn Emit) -> AppResult<RunResult> {
        {
            let mut slot = self.run_slot();
            if slot.active {
                return Err(AppError::new(
                    "run_in_progress",
                    "a run is already in progress",
                ));
            }
            *slot = crate::state::RunSlot {
                active: true,
                ..Default::default()
            };
        }
        let _guard = SlotGuard(self);
        let outcome = self.run_engine(out);
        let progress = |phase| {
            events::emit(
                out,
                events::RUN_PROGRESS,
                &RunProgress {
                    phase,
                    files: None,
                    total_files: None,
                    bytes: None,
                    current_path: None,
                },
            )
        };
        match outcome {
            Ok(run) => {
                let cancelled = run.errors.iter().any(|e| e == CANCELLED);
                if let Err(e) = self.store.push_run(run.clone()) {
                    eprintln!("run history not written: {e}");
                }
                events::emit(out, events::RUN_FINISHED, &run);
                if cancelled {
                    return Err(AppError::new("run_failed", CANCELLED));
                }
                Ok(run)
            }
            Err(e) => {
                progress(Phase::Failed);
                Err(e)
            }
        }
    }

    fn run_engine(&self, out: &dyn Emit) -> AppResult<RunResult> {
        // Same guard as `steward apply`: never run with an enabled target that is not allowed.
        let blocked = overview::blocked_enabled(&self.engine).map_err(AppError::engine)?;
        if let Some(t) = blocked.first() {
            let code = if t.risk == steward_core::model::Risk::CloneShared {
                "clone_shared"
            } else {
                "target_blocked"
            };
            return Err(AppError::new(
                code,
                format!(
                    "{} is enabled but blocked: {}",
                    t.id,
                    t.risk_detail.clone().unwrap_or_default()
                ),
            ));
        }
        events::emit(
            out,
            events::RUN_PROGRESS,
            &RunProgress {
                phase: Phase::Starting,
                files: None,
                total_files: None,
                bytes: None,
                current_path: None,
            },
        );
        let preview = self.engine.preview().map_err(AppError::engine)?;
        let mut tracker = Tracker::new(Some(preview.files), PROGRESS_GAP);
        let (mut run, success) = self
            .engine
            .apply_with(
                preview,
                |pid| self.register_pid(pid),
                |line| {
                    if let Some(p) = tracker.on_line(line) {
                        events::emit(out, events::RUN_PROGRESS, &p);
                    }
                },
            )
            .map_err(AppError::engine)?;
        let cancelled = self.run_slot().cancelled;
        if cancelled {
            run.errors.push(CANCELLED.into());
        }
        let phase = if success && !cancelled {
            Phase::Done
        } else {
            Phase::Failed
        };
        events::emit(out, events::RUN_PROGRESS, &tracker.snapshot(phase, None));
        Ok(run)
    }

    /// Remembers the child pid; a cancel that arrived before the child existed is delivered now.
    fn register_pid(&self, pid: u32) {
        let mut slot = self.run_slot();
        slot.pid = Some(pid);
        if slot.cancelled {
            interrupt(pid);
        }
    }

    /// SIGINT to the engine: its swaps are atomic, so stopping is safe.
    pub fn cancel_run(&self) -> AppResult<()> {
        let mut slot = self.run_slot();
        if !slot.active {
            return Err(AppError::not_found("no run in progress"));
        }
        slot.cancelled = true;
        if let Some(pid) = slot.pid {
            interrupt(pid);
        }
        Ok(())
    }
}

fn interrupt(pid: u32) {
    if let Ok(pid) = i32::try_from(pid) {
        // SAFETY: plain kill(2) on a child pid we spawned; failure (already exited) is harmless.
        unsafe { libc::kill(pid, libc::SIGINT) };
    }
}
