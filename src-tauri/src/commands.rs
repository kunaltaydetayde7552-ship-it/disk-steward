//! Tauri commands: thin wrappers over `AppState`. Blocking work never runs on the main thread.

use crate::{
    error::{AppError, AppResult},
    events::{AppEmitter, Emit},
    state::AppState,
};
use std::{path::Path, sync::Arc};
use steward_core::{
    model::{Overview, Preview, RunResult, ScheduleAgent, Target},
    schedule,
};
use tauri::{AppHandle, State};

type Shared<'a> = State<'a, Arc<AppState>>;

async fn blocking<T: Send + 'static>(
    f: impl FnOnce() -> AppResult<T> + Send + 'static,
) -> AppResult<T> {
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(AppError::internal)?
}

fn emitter(app: AppHandle) -> Arc<dyn Emit> {
    Arc::new(AppEmitter(app))
}

#[tauri::command]
pub async fn get_overview(app: AppHandle, state: Shared<'_>) -> AppResult<Overview> {
    let state = Arc::clone(&state);
    state.refresh_if_outdated(emitter(app));
    blocking(move || state.get_overview()).await
}

#[tauri::command]
pub async fn refresh_overview(app: AppHandle, state: Shared<'_>) -> AppResult<()> {
    state.refresh_overview(emitter(app));
    Ok(())
}

#[tauri::command]
pub async fn set_target_enabled(
    app: AppHandle,
    state: Shared<'_>,
    id: String,
    enabled: bool,
) -> AppResult<Target> {
    let state = Arc::clone(&state);
    let target = blocking({
        let state = Arc::clone(&state);
        move || state.set_target_enabled(&id, enabled)
    })
    .await?;
    state.refresh_if_outdated(emitter(app)); // the config changed, so the risks may have too
    Ok(target)
}

#[tauri::command]
pub async fn preview(state: Shared<'_>) -> AppResult<Preview> {
    let state = Arc::clone(&state);
    blocking(move || state.engine.preview().map_err(AppError::engine)).await
}

#[tauri::command]
pub async fn apply(app: AppHandle, state: Shared<'_>) -> AppResult<RunResult> {
    let state = Arc::clone(&state);
    let out = emitter(app);
    let run = blocking({
        let (state, out) = (Arc::clone(&state), Arc::clone(&out));
        move || state.apply(out.as_ref())
    })
    .await;
    state.refresh_if_outdated(out); // the engine state changed, even for a cancelled or failed run
    run
}

#[tauri::command]
pub async fn cancel_run(state: Shared<'_>) -> AppResult<()> {
    state.cancel_run()
}

#[tauri::command]
pub async fn list_runs(state: Shared<'_>) -> AppResult<Vec<RunResult>> {
    Ok(state.store.load_runs())
}

#[tauri::command]
pub async fn get_schedule(state: Shared<'_>) -> AppResult<Vec<ScheduleAgent>> {
    let home = state.engine.home.clone();
    blocking(move || Ok(schedule::list(&home))).await
}

#[tauri::command]
pub async fn install_schedule(state: Shared<'_>) -> AppResult<Vec<ScheduleAgent>> {
    let state = Arc::clone(&state);
    blocking(move || state.install_schedule()).await
}

#[tauri::command]
pub async fn uninstall_schedule(state: Shared<'_>) -> AppResult<Vec<ScheduleAgent>> {
    let state = Arc::clone(&state);
    blocking(move || state.uninstall_schedule()).await
}

#[tauri::command]
pub async fn reveal_in_finder(path: String) -> AppResult<()> {
    if !Path::new(&path).exists() {
        return Err(AppError::not_found(format!("{path} does not exist")));
    }
    tauri_plugin_opener::reveal_item_in_dir(&path).map_err(AppError::internal)
}
