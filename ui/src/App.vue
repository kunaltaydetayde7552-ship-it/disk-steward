<script setup lang="ts">
// Two scenes in one bundle: the main window and the menu bar popup (#/tray, its own webview in Tauri).
import { onBeforeUnmount, onMounted } from "vue";
import { on } from "./api/client";
import MainShell from "./components/shell/MainShell.vue";
import TrayView from "./components/tray/TrayView.vue";
import { useOverview } from "./stores/overview";
import { useRun } from "./stores/run";
import { useUi } from "./stores/ui";

const ui = useUi();
const ov = useOverview();
const run = useRun();
const offs: (() => void)[] = [];

onMounted(async () => {
  await Promise.all([ov.init(), run.init()]);
  offs.push(
    // the tray popup asks the main window to show a screen (and maybe start a preview); in a plain browser
    // the tray scene and the main window are the same page, so this also leaves #/tray
    await on("ui://navigate", ({ path, action }) => {
      ui.go(path);
      if (action === "preview") void run.startPreview();
    }),
    // the tray menu's "Comprimir agora": preview, then the same confirmation as always; never apply() directly
    await on("tray://compress-now", () => {
      if (ui.route.name === "tray") return;
      if (run.phase === "preview") return run.askConfirm();
      if (run.busy) return ui.goScreen("runs");
      ui.goScreen(ov.enabled.length ? "runs" : "targets");
      void run.startPreview(true);
    }),
  );
});
onBeforeUnmount(() => offs.forEach((off) => off()));
</script>

<template>
  <TrayView v-if="ui.route.name === 'tray'" />
  <MainShell v-else />
</template>
