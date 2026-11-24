//! Menu bar item: runtime-drawn glyph, free-space title, popup on left click, native menu on right click.

use crate::{
    events, tray_glyph,
    tray_i18n::{Key, Locale, fill, tr},
    tray_window,
};
use std::{sync::Mutex, time::Duration};
use steward_core::disk;
use tauri::{
    AppHandle, Emitter, Manager,
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

const TRAY_ID: &str = "free-space";
const REFRESH: Duration = Duration::from_secs(60);
const GIB: f64 = 1_073_741_824.0;

/// Locale chosen at startup; the strings are static.
static LOCALE: Mutex<Option<Locale>> = Mutex::new(None);

/// `"75 GB"`: free space in GiB rounded to a whole number (the unit label follows what Finder calls it).
pub fn free_label(free_bytes: u64) -> String {
    format!("{} GB", (free_bytes as f64 / GIB).round() as u64)
}

fn locale() -> Locale {
    *LOCALE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get_or_insert_with(Locale::system)
}

/// Re-reads the free space (instant statfs) and updates the title and tooltip. Safe from any thread.
pub fn refresh(app: &AppHandle) {
    let (Some(tray), Ok(status)) = (app.tray_by_id(TRAY_ID), disk::status()) else {
        return;
    };
    let label = free_label(status.free_bytes);
    // tray calls fail only while the tray is being torn down
    let _ = tray.set_tooltip(Some(fill(tr(locale(), Key::Tooltip), &[("free", &label)])));
    #[cfg(target_os = "macos")]
    let _ = tray.set_title(Some(label));
}

pub fn show_main_window(app: &AppHandle) {
    #[cfg(target_os = "macos")]
    let _ = app.set_dock_visibility(true);
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

/// Builds the tray icon and starts the periodic title refresh.
pub fn init(app: &AppHandle) -> tauri::Result<()> {
    let loc = locale();
    let open = MenuItem::with_id(app, "open", tr(loc, Key::OpenApp), true, None::<&str>)?;
    let compress = MenuItem::with_id(
        app,
        "compress",
        tr(loc, Key::CompressNow),
        true,
        None::<&str>,
    )?;
    let quit = MenuItem::with_id(app, "quit", tr(loc, Key::Quit), true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &compress, &quit])?;
    let glyph = tray_glyph::render(cfg!(target_os = "macos"));
    let builder = TrayIconBuilder::with_id(TRAY_ID)
        .icon(Image::new_owned(glyph.rgba, glyph.w, glyph.h))
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, e| match e.id().as_ref() {
            "open" => show_main_window(app),
            "compress" => {
                show_main_window(app);
                // the UI asks for confirmation; the click itself compresses nothing
                let _ = app.emit(events::TRAY_COMPRESS_NOW, serde_json::json!({}));
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, e| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                rect,
                ..
            } = e
            {
                tray_window::toggle(tray.app_handle(), rect);
            }
        });
    #[cfg(target_os = "macos")]
    let builder = builder.icon_as_template(true);
    builder.build(app)?;
    refresh(app);
    let handle = app.clone();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(REFRESH);
            refresh(&handle);
        }
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn free_label_rounds_gib() {
        assert_eq!(free_label(80_530_636_800), "75 GB");
        assert_eq!(free_label(0), "0 GB");
        assert_eq!(free_label(1_610_612_736), "2 GB"); // 1.5 GiB
    }
}
