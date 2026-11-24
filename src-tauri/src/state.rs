//! Shared app state: the engine, the stores, and the "one at a time" guards for measuring and running.

use crate::{
    dto::Failure,
    error::{AppError, AppResult},
    events::{self, Emit},
    store::Store,
};
use std::{
    fs,
    path::Path,
    sync::{
        Arc, Mutex, PoisonError,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use steward_core::{
    disk,
    engine::Engine,
    model::{Measurement, MeasurementState, Overview, Risk, ScheduleAgent, Target},
    overview::{self, probe, resolve, unattributed_bytes},
    schedule,
    util::rfc3339_to_secs,
};

/// Bump whenever risk or measurement semantics change: it invalidates every cached overview.
const RULES_VERSION: u32 = 2;

/// Sizes older than this are reported as `stale`.
pub const STALE_AFTER: Duration = Duration::from_secs(6 * 3600);

#[derive(Debug, Default)]
pub struct RunSlot {
    pub active: bool,
    pub pid: Option<u32>,
    pub cancelled: bool,
}

pub struct AppState {
    pub engine: Engine,
    pub store: Store,
    measuring: AtomicBool,
    pub run: Mutex<RunSlot>,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

impl AppState {
    pub fn new(engine: Engine, store: Store) -> Self {
        Self {
            engine,
            store,
            measuring: AtomicBool::new(false),
            run: Mutex::default(),
        }
    }

    pub fn run_slot(&self) -> std::sync::MutexGuard<'_, RunSlot> {
        self.run.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The cached overview (free space and `measurement` refreshed), or sizeless targets when nothing is cached.
    pub fn get_overview(&self) -> AppResult<Overview> {
        let mut overview = match self.store.load_overview() {
            Some(cached) => cached,
            None => overview::overview(&self.engine, false).map_err(AppError::engine)?,
        };
        if let Ok(status) = disk::status() {
            overview.disk = status;
        }
        overview.measurement = self.measurement_of(&overview.measurement);
        Ok(overview)
    }

    /// What the cached sizes depend on: app version, rules version, and the mtimes of the engine config
    /// and state file (every apply, in-app, CLI or launchd, rewrites the state).
    pub fn fingerprint(&self) -> String {
        let mtime = |p: &Path| {
            fs::metadata(p)
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map_or(0, |d| d.as_nanos())
        };
        let state = self
            .engine
            .home
            .join("Library/Application Support/auto-compress-apfs/state.tsv");
        format!(
            "{}|{RULES_VERSION}|{}|{}",
            env!("CARGO_PKG_VERSION"),
            mtime(&self.engine.config),
            mtime(&state)
        )
    }

    /// True when a measured cache exists but was measured against something else than the current fingerprint.
    fn cache_outdated(&self) -> bool {
        self.store.load_fingerprint().as_deref() != Some(self.fingerprint().as_str())
    }

    /// Starts the background refresh when the cached sizes are outdated (and nothing is running).
    pub fn refresh_if_outdated(self: &Arc<Self>, out: Arc<dyn Emit>) {
        let measured = self
            .store
            .load_overview()
            .is_some_and(|o| o.measurement.measured_at.is_some());
        if measured && self.cache_outdated() && !self.run_slot().active {
            self.refresh_overview(out);
        }
    }

    fn measurement_of(&self, stored: &Measurement) -> Measurement {
        let state = match (&stored.measured_at, self.measuring.load(Ordering::SeqCst)) {
            (_, true) => MeasurementState::Measuring,
            (None, false) => MeasurementState::Never,
            (Some(_), false) if self.cache_outdated() => MeasurementState::Stale,
            (Some(at), false) => match rfc3339_to_secs(at) {
                Some(t) if now_secs().saturating_sub(t) <= STALE_AFTER.as_secs() => {
                    MeasurementState::Fresh
                }
                _ => MeasurementState::Stale,
            },
        };
        Measurement {
            state,
            measured_at: stored.measured_at.clone(),
        }
    }

    /// Starts a full measurement on a background thread; a no-op while one is already running.
    pub fn refresh_overview(self: &Arc<Self>, out: Arc<dyn Emit>) {
        if self.measuring.swap(true, Ordering::SeqCst) {
            return;
        }
        events::emit(
            out.as_ref(),
            events::OVERVIEW_MEASURING,
            &serde_json::json!({}),
        );
        let state = Arc::clone(self);
        thread::spawn(move || {
            let fingerprint = state.fingerprint(); // taken before measuring: a change meanwhile reads as outdated
            let result = overview::overview(&state.engine, true);
            let saved = result.as_ref().ok().map(|o| {
                state
                    .store
                    .save_overview(o)
                    .and_then(|()| state.store.save_fingerprint(&fingerprint))
            });
            state.measuring.store(false, Ordering::SeqCst);
            match result {
                Ok(o) => {
                    if let Some(Err(e)) = saved {
                        eprintln!("overview cache not written: {e}");
                    }
                    events::emit(out.as_ref(), events::OVERVIEW_UPDATED, &o);
                }
                Err(message) => {
                    let e = AppError::engine(message);
                    events::emit(
                        out.as_ref(),
                        events::OVERVIEW_FAILED,
                        &Failure {
                            code: e.code,
                            message: e.message,
                        },
                    );
                }
            }
        });
    }

    /// Adds or removes a target from the engine patterns. Only top-level `patterns` change.
    pub fn set_target_enabled(&self, id: &str, enabled: bool) -> AppResult<Target> {
        if self.run_slot().active {
            return Err(AppError::new("run_in_progress", "a run is in progress"));
        }
        let entry = resolve(&self.engine, id).map_err(AppError::not_found)?;
        let probe = probe(&self.engine, &entry);
        if enabled {
            if probe.risk == Risk::CloneShared {
                return Err(AppError::new(
                    "clone_shared",
                    probe.risk_detail.unwrap_or_default(),
                ));
            }
            if !probe.allowed {
                return Err(AppError::new(
                    "target_blocked",
                    probe.risk_detail.unwrap_or_default(),
                ));
            }
        }
        self.engine
            .set_target_enabled(&entry, enabled)
            .map_err(AppError::config)?;
        Ok(self.mark_cached(probe, enabled))
    }

    /// Wraps `install-launchd` with the real config path. Writes `~/Library/LaunchAgents`: tests use a stub engine.
    pub fn install_schedule(&self) -> AppResult<Vec<ScheduleAgent>> {
        self.engine
            .install_launchd()
            .map_err(|m| AppError::new("schedule_failed", m))?;
        Ok(schedule::list(&self.engine.home))
    }

    pub fn uninstall_schedule(&self) -> AppResult<Vec<ScheduleAgent>> {
        self.engine
            .uninstall_launchd()
            .map_err(|m| AppError::new("schedule_failed", m))?;
        Ok(schedule::list(&self.engine.home))
    }

    /// Mirrors the new `enabled` flag into the cached overview and returns the target with its cached sizes.
    fn mark_cached(&self, probe: Target, enabled: bool) -> Target {
        let Some(mut cached) = self.store.load_overview() else {
            return Target { enabled, ..probe };
        };
        let slot = cached.targets.iter_mut().find(|t| t.id == probe.id);
        let target = match slot {
            Some(t) => {
                t.enabled = enabled;
                t.clone()
            }
            None => Target { enabled, ..probe },
        };
        if cached.measurement.measured_at.is_some() {
            cached.savings.unattributed_bytes =
                unattributed_bytes(cached.savings.saved_bytes, &cached.targets);
        }
        if let Err(e) = self.store.save_overview(&cached) {
            eprintln!("overview cache not written: {e}");
        }
        target
    }
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;
