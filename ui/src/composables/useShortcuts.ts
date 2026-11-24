// Window shortcuts: Cmd+1..4 sections, Cmd+Return the dock's primary action, Cmd+R measure again.
// Esc is the native dialog's own cancel. Cmd+R is always swallowed so the webview never reloads.

import { onBeforeUnmount, onMounted, type Ref } from "vue";
import { useOverview } from "../stores/overview";
import { useRun } from "../stores/run";
import { SCREENS, useUi } from "../stores/ui";
import type { DockModel } from "./useDock";

export function useShortcuts(dock: Ref<DockModel>) {
  const ui = useUi();
  const ov = useOverview();
  const run = useRun();

  function onKey(e: KeyboardEvent) {
    const mod = e.metaKey || e.ctrlKey;
    if (!mod || e.altKey) return;
    const key = e.key.toLowerCase();
    if (key === "r") {
      e.preventDefault();
      if (!ui.confirmOpen && !run.busy) void ov.refresh();
      return;
    }
    if (ui.confirmOpen) return; // the dialog owns the keyboard while open
    if (e.key === "Enter") {
      const a = dock.value.actions.find((x) => x.primary && !x.disabled);
      if (!a) return;
      e.preventDefault();
      a.run();
      return;
    }
    const n = Number(e.key);
    if (n >= 1 && n <= SCREENS.length && !e.shiftKey) {
      e.preventDefault();
      ui.goScreen(SCREENS[n - 1].id);
    }
  }

  onMounted(() => window.addEventListener("keydown", onKey));
  onBeforeUnmount(() => window.removeEventListener("keydown", onKey));
}
