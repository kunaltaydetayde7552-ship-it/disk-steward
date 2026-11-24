// The dock: what is pending right now and the one action that moves it forward. One model for the dock,
// the Cmd+Return shortcut and the screenshots, so the three never disagree.

import { computed, nextTick } from "vue";
import { useI18n } from "vue-i18n";
import type { IconName } from "../components/ui/icons";
import { bytes, day, dayFromYmd, int, tilde } from "../lib/format";
import { targetShort } from "../lib/targets";
import { useOverview } from "../stores/overview";
import { useRun } from "../stores/run";
import { useUi } from "../stores/ui";

export type DockTone = "busy" | "count" | "run" | "ok" | "red" | "idle";

export interface DockAction {
  id: string;
  label: string;
  kind: "primary" | "secondary" | "ghost";
  icon?: IconName;
  busy?: boolean;
  disabled?: boolean;
  /** Cmd+Return presses this one */
  primary?: boolean;
  run: () => void;
}

export interface DockModel {
  /** changes when the dock shows a different situation (drives the swap animation) */
  key: string;
  tone: DockTone;
  icon?: IconName;
  count?: string;
  title: string;
  sub: string;
  bar?: boolean;
  actions: DockAction[];
}

/** Focus the failed run's log on the Runs screen. */
export async function focusRunLog(goRuns: () => void) {
  goRuns();
  await nextTick();
  requestAnimationFrame(() => {
    const log = document.getElementById("run-log");
    log?.scrollIntoView({ block: "center", behavior: matchMedia("(prefers-reduced-motion: reduce)").matches ? "auto" : "smooth" });
    log?.focus({ preventScroll: true });
  });
}

export function useDock() {
  const { t } = useI18n();
  const ui = useUi();
  const ov = useOverview();
  const run = useRun();

  return computed<DockModel>(() => {
    const preview = run.preview;
    switch (run.phase) {
      case "previewing":
        return {
          key: "previewing", tone: "busy",
          title: t("dock.previewing.title"),
          sub: t("dock.previewing.sub", { n: ov.enabled.length }, ov.enabled.length),
          actions: [{ id: "previewing", label: t("dock.previewing.action"), kind: "primary", busy: true, disabled: true, run: () => {} }],
        };
      case "preview":
        if (!preview || preview.files === 0) {
          return {
            key: "preview-empty", tone: "ok", icon: "check",
            title: t("dock.previewEmpty.title"), sub: t("dock.previewEmpty.sub"),
            actions: [{ id: "close", label: t("dock.previewEmpty.action"), kind: "secondary", primary: true, run: run.discard }],
          };
        }
        return {
          key: "preview", tone: "count", count: int(preview.files),
          title: t("dock.preview.title", { files: int(preview.files), bytes: bytes(preview.bytes) }),
          sub: t("dock.preview.sub"),
          actions: [
            { id: "discard", label: t("dock.preview.discard"), kind: "ghost", run: run.discard },
            { id: "confirm", label: t("dock.preview.action", { bytes: bytes(preview.bytes) }), kind: "primary", icon: "compress", primary: true, run: run.askConfirm },
          ],
        };
      case "running": {
        const p = run.progress?.phase;
        const title = p === "verifying" ? t("dock.running.verifying") : run.activeRoot ? t("dock.running.title", { path: tilde(run.activeRoot) }) : t("dock.running.starting");
        return {
          key: "running", tone: "run", bar: true, title,
          sub: t("dock.running.sub", { files: int(run.files), total: int(run.totalFiles) }),
          actions: [{
            id: "stop", label: run.stopRequested ? t("dock.running.stopping") : t("dock.running.stop"), kind: "secondary", icon: "stop",
            busy: run.stopRequested, disabled: run.stopRequested, run: run.cancel,
          }],
        };
      }
      case "done": {
        const again: DockAction = { id: "again", label: t("dock.done.again"), kind: "secondary", icon: "eye", primary: true, run: () => void run.startPreview() };
        const view: DockAction[] = ui.screen !== "runs" ? [{ id: "view", label: t("dock.done.view"), kind: "ghost", run: () => ui.goScreen("runs") }] : [];
        const warned = (run.result?.errors.length ?? 0) > 0 && !run.stopped;
        if (run.stopped) return { key: "stopped", tone: "ok", icon: "stop", title: t("dock.done.stoppedTitle"), sub: t("dock.done.stoppedSub", { files: int(run.filesDone) }), actions: [...view, again] };
        if (run.freed <= 0) return { key: "no-gain", tone: "ok", icon: "check", title: t("dock.done.noGainTitle"), sub: t("dock.done.noGainSub"), actions: [...view, again] };
        return {
          key: "done", tone: "ok", icon: "check",
          title: t("dock.done.title", { bytes: bytes(run.freed) }),
          sub: t(warned ? "dock.done.subWarnings" : "dock.done.sub", { files: int(run.filesDone) }),
          actions: [...view, again],
        };
      }
      case "failed": {
        const atFile = run.failedPath != null;
        return {
          key: "failed", tone: "red", icon: "alert",
          title: t(atFile ? "dock.failed.title" : "dock.failed.titleOther"),
          sub: atFile ? t("dock.failed.sub", { files: int(run.files), bytes: bytes(Math.max(0, run.freed)) }) : t("dock.failed.subOther"),
          actions: [
            { id: "log", label: t("dock.failed.log"), kind: "ghost", icon: "log", run: () => void focusRunLog(() => ui.goScreen("runs")) },
            { id: "retry", label: t("dock.failed.retry"), kind: "primary", icon: "refresh", primary: true, run: () => void run.retry() },
          ],
        };
      }
    }
    if (ov.engineMissing) {
      return {
        key: "engine-missing", tone: "red", icon: "alert",
        title: t("dock.engineMissing.title"), sub: t("dock.engineMissing.sub"),
        actions: [{
          id: "find", label: ov.engineBusy ? t("dock.engineMissing.busy") : t("dock.engineMissing.action"), kind: "primary", icon: "refresh",
          busy: ov.engineBusy, disabled: ov.engineBusy, primary: true, run: () => void ov.refresh(),
        }],
      };
    }
    if (ov.measuring) {
      return { key: "measuring", tone: "busy", title: t("dock.measuring.title", { n: ov.targets.length }), sub: t("dock.measuring.sub"), actions: [] };
    }
    if (ov.neverMeasured) {
      return {
        key: "never", tone: "idle", icon: "refresh", title: t("dock.never.title"), sub: t("dock.never.sub"),
        actions: [{ id: "measure", label: t("dock.never.action"), kind: "primary", icon: "refresh", primary: true, run: () => void ov.refresh() }],
      };
    }
    if (!ov.enabled.length) {
      return {
        key: "no-targets", tone: "idle", icon: "targets", title: t("dock.noTargets.title"), sub: t("dock.noTargets.sub"),
        actions: [{ id: "targets", label: t("dock.noTargets.action"), kind: "secondary", icon: "targets", primary: true, run: () => ui.goScreen("targets") }],
      };
    }
    const lastDay = ov.savings?.lastRunDay ? dayFromYmd(ov.savings.lastRunDay) : run.lastRun ? day(run.lastRun.startedAt) : null;
    return {
      key: "idle", tone: "idle", icon: "eye",
      title: lastDay ? t("dock.idle.title", { day: lastDay }) : t("dock.idle.titleFirst"),
      sub: t("dock.idle.sub", { names: ov.enabled.map(targetShort).join(", ") }),
      actions: [{ id: "preview", label: t("dock.idle.action"), kind: "primary", icon: "eye", primary: true, run: () => void run.startPreview() }],
    };
  });
}
