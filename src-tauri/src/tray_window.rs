//! The borderless popup shown under the menu bar icon (same frontend, route `#/tray`).

use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};
use tauri::{
    AppHandle, Manager, PhysicalPosition, Rect, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
    WindowEvent,
};

pub const LABEL: &str = "tray";
pub const WIDTH: f64 = 360.0;
pub const HEIGHT: f64 = 480.0;
const MARGIN: i32 = 8;
/// A click on the icon while the popup is open first blurs it (hiding it); that click must not reopen it.
const REOPEN_GUARD: Duration = Duration::from_millis(300);

static LAST_BLUR_HIDE: Mutex<Option<Instant>> = Mutex::new(None);

/// Top-left of the popup (physical px): centered under the icon, kept inside the monitor `[left, left + width)`.
pub fn position(
    icon: (f64, f64, f64, f64),
    window_w: i32,
    monitor: Option<(i32, i32)>,
) -> (i32, i32) {
    let (ix, iy, iw, ih) = icon;
    let x = (ix + iw / 2.0).round() as i32 - window_w / 2;
    let x = match monitor {
        Some((left, width)) => x.clamp(
            left + MARGIN,
            (left + width - window_w - MARGIN).max(left + MARGIN),
        ),
        None => x,
    };
    (x, (iy + ih).round() as i32 + MARGIN / 2)
}

pub fn toggle(app: &AppHandle, icon: Rect) {
    let Ok(window) = get_or_create(app) else {
        return;
    };
    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
        return;
    }
    let recently_hidden = LAST_BLUR_HIDE
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .is_some_and(|t| t.elapsed() < REOPEN_GUARD);
    if recently_hidden {
        return;
    }
    let scale = window.scale_factor().unwrap_or(1.0);
    let (p, s) = (
        icon.position.to_physical::<f64>(scale),
        icon.size.to_physical::<f64>(scale),
    );
    let monitor = app
        .monitor_from_point(p.x + s.width / 2.0, p.y)
        .ok()
        .flatten()
        .map(|m| {
            (
                m.position().x,
                i32::try_from(m.size().width).unwrap_or(i32::MAX),
            )
        });
    let (x, y) = position(
        (p.x, p.y, s.width, s.height),
        (WIDTH * scale) as i32,
        monitor,
    );
    let _ = window.set_position(PhysicalPosition::new(x, y));
    let _ = window.show();
    let _ = window.set_focus();
}

/// Hides the popup when it loses focus. Call from the global window-event handler.
pub fn on_event(window: &tauri::Window, event: &WindowEvent) {
    if window.label() == LABEL && matches!(event, WindowEvent::Focused(false)) {
        *LAST_BLUR_HIDE
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = Some(Instant::now());
        let _ = window.hide();
    }
}

fn get_or_create(app: &AppHandle) -> tauri::Result<WebviewWindow> {
    if let Some(w) = app.get_webview_window(LABEL) {
        return Ok(w);
    }
    let window = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("index.html#/tray".into()))
        .inner_size(WIDTH, HEIGHT)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)
        .build()?;
    #[cfg(target_os = "macos")]
    {
        use window_vibrancy::{NSVisualEffectMaterial, apply_vibrancy};
        if let Err(e) = apply_vibrancy(&window, NSVisualEffectMaterial::HudWindow, None, Some(12.0))
        {
            eprintln!("vibrancy unavailable: {e}");
        }
    }
    Ok(window)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centers_under_the_icon_and_clamps_to_the_monitor() {
        // icon at x=1000..1040, y=0..24 on a 1440 wide monitor
        assert_eq!(
            position((1000.0, 0.0, 40.0, 24.0), 720, Some((0, 1440))),
            (660, 28)
        );
        // icon near the right edge: popup stays inside
        assert_eq!(
            position((1420.0, 0.0, 20.0, 24.0), 720, Some((0, 1440))),
            (1440 - 720 - MARGIN, 28)
        );
        // icon near the left edge of a monitor at x=1440
        assert_eq!(
            position((1445.0, 0.0, 20.0, 24.0), 720, Some((1440, 1440))),
            (1440 + MARGIN, 28)
        );
        // unknown monitor: no clamping
        assert_eq!(position((100.0, 0.0, 20.0, 24.0), 720, None), (-250, 28));
    }
}
