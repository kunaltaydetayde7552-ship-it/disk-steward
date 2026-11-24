//! Tauri shell around `steward-core`.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod dto;
mod error;
mod events;
mod progress;
mod run;
mod state;
mod store;
#[cfg(test)]
mod testkit;
mod tray;
mod tray_glyph;
mod tray_i18n;
mod tray_window;

use std::sync::Arc;
use steward_core::engine::Engine;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .on_window_event(|window, event| {
            tray_window::on_event(window, event);
            // closing the main window sends the app to the menu bar (and off the Dock) so the tray keeps
            // updating; quit via the tray menu or Cmd+Q
            if let (true, tauri::WindowEvent::CloseRequested { api, .. }) =
                (window.label() == "main", event)
            {
                api.prevent_close();
                let _ = window.hide();
                #[cfg(target_os = "macos")]
                let _ = window.app_handle().set_dock_visibility(false);
            }
        })
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            app.manage(Arc::new(state::AppState::new(
                Engine::new(),
                store::Store::new(data_dir),
            )));
            tray::init(app.handle())?;
            #[cfg(target_os = "macos")]
            if let Some(window) = app.get_webview_window("main") {
                use window_vibrancy::{NSVisualEffectMaterial, apply_vibrancy};
                if let Err(e) = apply_vibrancy(
                    &window,
                    NSVisualEffectMaterial::UnderWindowBackground,
                    None,
                    None,
                ) {
                    eprintln!("vibrancy unavailable: {e}");
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_overview,
            commands::refresh_overview,
            commands::set_target_enabled,
            commands::preview,
            commands::apply,
            commands::cancel_run,
            commands::list_runs,
            commands::get_schedule,
            commands::install_schedule,
            commands::uninstall_schedule,
            commands::reveal_in_finder,
        ])
        .build(tauri::generate_context!())
        .expect("error while building the application")
        .run(|_app, _event| {
            // the close button only hides the window, so opening the app again has to bring it back
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Reopen { .. } = _event {
                tray::show_main_window(_app);
            }
        });
}
