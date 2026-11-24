// Window plumbing between the tray popup (#/tray) and the main window. Every call is best effort:
// in a plain browser the tray view and the main window are the same page, so it navigates in place.
//
// Capabilities the backend must grant (src-tauri/capabilities/default.json):
//   core:window:allow-show, core:window:allow-set-focus, core:window:allow-unminimize, core:window:allow-hide,
//   core:window:allow-start-dragging (title bar drag regions), core:event:allow-emit-to, process:allow-exit.

import { emitInPage, isTauri } from "./client";
import type { EventMap } from "./types";

type Navigate = EventMap["ui://navigate"];

/** Shows and focuses the main window, then asks it to go to `path` (and optionally start a preview). */
export async function openMainWindow(path: string, action: Navigate["action"] = null): Promise<void> {
  if (!isTauri()) {
    emitInPage("ui://navigate", { path, action });
    return;
  }
  try {
    const { WebviewWindow, getCurrentWebviewWindow } = await import("@tauri-apps/api/webviewWindow");
    const { emitTo } = await import("@tauri-apps/api/event");
    const main = await WebviewWindow.getByLabel("main");
    if (main) {
      await main.show();
      await main.unminimize();
      await main.setFocus();
      await emitTo("main", "ui://navigate", { path, action } satisfies Navigate);
    }
    await getCurrentWebviewWindow().hide();
  } catch (e) {
    console.error("open main window", e);
  }
}

/** Hides the tray popup (Esc). */
export async function hideCurrentWindow(): Promise<void> {
  if (!isTauri()) return;
  try {
    const { getCurrentWebviewWindow } = await import("@tauri-apps/api/webviewWindow");
    await getCurrentWebviewWindow().hide();
  } catch (e) {
    console.error("hide window", e);
  }
}

export async function quitApp(): Promise<void> {
  if (!isTauri()) return;
  try {
    const { exit } = await import("@tauri-apps/plugin-process");
    await exit(0);
  } catch (e) {
    console.error("quit", e);
  }
}
