// One compression, start to proof: preview (read only) -> confirmation -> apply -> df result.
// The phase follows the backend events, so a run started in one window shows in the other.

import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { api, CANCELLED, on, type ApiError, type Preview, type RunProgress, type RunResult } from "../api/client";
import { errorCode, errorMessage, errorText } from "../lib/errors";
import { useOverview } from "./overview";
import { useUi } from "./ui";

export type Phase = "idle" | "previewing" | "preview" | "running" | "done" | "failed";

const isError = (line: string) => /^\s*ERROR\b/.test(line);

export const useRun = defineStore("run", () => {
  const ui = useUi();
  const ov = useOverview();

  const phase = ref<Phase>("idle");
  const preview = ref<Preview | null>(null);
  const progress = ref<RunProgress | null>(null);
  const result = ref<RunResult | null>(null);
  /** apply rejected before or without a result (engine missing, blocked target, ...) */
  const failure = ref<ApiError | null>(null);
  const stopped = ref(false);
  const stopRequested = ref(false);
  const history = ref<RunResult[]>([]);
  /** free space when the run started: "Livre antes" */
  const freeBefore = ref<number | null>(null);
  /** preview roots in the order the engine reached them */
  const reached = ref<string[]>([]);
  /** highest file count reported */
  const files = ref(0);
  /** open the confirmation as soon as the preview arrives (retry, tray "Comprimir agora") */
  const confirmWhenReady = ref(false);
  let lastPhase: RunProgress["phase"] | null = null;
  let finishedAt: string | null = null;

  const totalFiles = computed(() => progress.value?.totalFiles ?? preview.value?.files ?? 0);
  const fraction = computed(() => (totalFiles.value > 0 ? Math.min(1, files.value / totalFiles.value) : 0));
  const busy = computed(() => phase.value === "previewing" || phase.value === "running");
  const lastRun = computed(() => history.value[0] ?? null);
  /** the root being compressed now */
  const activeRoot = computed(() => reached.value[reached.value.length - 1] ?? null);
  /** the path the failure points at (for the red cube of its target) */
  const failedPath = computed(() => {
    if (phase.value !== "failed") return null;
    const line = result.value?.errors.find(isError) ?? "";
    return line.match(/(\/[^\s…]+)/)?.[1] ?? null;
  });
  const freed = computed(() => result.value?.freedBytes ?? 0);
  const filesDone = computed(() => (phase.value === "done" && !stopped.value ? (result.value?.preview.files ?? files.value) : files.value));

  function resetRun() {
    progress.value = null;
    result.value = null;
    failure.value = null;
    stopped.value = false;
    stopRequested.value = false;
    reached.value = [];
    files.value = 0;
    lastPhase = null;
    finishedAt = null;
  }

  async function startPreview(openConfirm = false) {
    if (busy.value || !ov.enabled.length) return;
    resetRun();
    confirmWhenReady.value = openConfirm;
    phase.value = "previewing";
    preview.value = null;
    try {
      const p = await api.preview();
      if (phase.value !== "previewing") return;
      preview.value = p;
      phase.value = "preview";
      if (confirmWhenReady.value && p.files > 0) ui.confirmOpen = true;
    } catch (e) {
      if (phase.value !== "previewing") return;
      phase.value = "idle";
      ov.noteError(e);
      if (errorCode(e) !== "engine_missing") ui.toast("error", errorText(e));
    } finally {
      confirmWhenReady.value = false;
    }
  }

  function discard() {
    if (phase.value === "preview" || phase.value === "done" || phase.value === "failed") {
      phase.value = "idle";
      preview.value = null;
      resetRun();
    }
  }

  /** Opens the confirmation; nothing is compressed by this. */
  function askConfirm() {
    if (phase.value === "preview" && preview.value && preview.value.files > 0) ui.confirmOpen = true;
  }

  /** Only called from the confirmation dialog. */
  async function apply() {
    if (phase.value === "running") return;
    resetRun();
    freeBefore.value = ov.disk?.freeBytes ?? null;
    phase.value = "running";
    try {
      finish(await api.apply());
    } catch (e) {
      const code = errorCode(e);
      if (code === "run_failed" && errorMessage(e) === CANCELLED) {
        // run://finished already carried the result; make sure the stop reads as a stop
        stopped.value = true;
        if (phase.value === "running") phase.value = "done";
        return;
      }
      if (code === "run_in_progress") return; // another window started one: its events drive this phase
      ov.noteError(e);
      if (code === "engine_missing") {
        phase.value = "idle";
        return;
      }
      failure.value = { code, message: errorMessage(e) };
      phase.value = "failed";
    }
  }

  async function cancel() {
    if (phase.value !== "running" || stopRequested.value) return;
    stopRequested.value = true;
    try {
      await api.cancelRun();
    } catch (e) {
      stopRequested.value = false;
      if (errorCode(e) !== "not_found") ui.toast("error", errorText(e));
    }
  }

  /** "Tentar de novo": a fresh preview, then the same confirmation as always. */
  const retry = () => startPreview(true);

  function onProgress(p: RunProgress) {
    if (p.phase === "starting" || p.phase === "compressing" || p.phase === "verifying") {
      if (phase.value !== "running") {
        resetRun();
        phase.value = "running";
        freeBefore.value = ov.disk?.freeBytes ?? null;
      }
    }
    lastPhase = p.phase;
    progress.value = { ...p, totalFiles: p.totalFiles ?? progress.value?.totalFiles ?? null };
    if (p.files != null) files.value = Math.max(files.value, p.files);
    const roots = preview.value?.roots ?? [];
    const root = p.currentPath ? roots.find((r) => p.currentPath!.startsWith(r.path)) : null;
    if (root && !reached.value.includes(root.path)) reached.value = [...reached.value, root.path];
  }

  function finish(r: RunResult) {
    if (finishedAt === r.startedAt) return;
    finishedAt = r.startedAt;
    result.value = r;
    if (!preview.value) preview.value = r.preview;
    stopped.value = r.errors.includes(CANCELLED);
    const failedRun = !stopped.value && (lastPhase === "failed" || r.errors.some(isError));
    phase.value = failedRun ? "failed" : "done";
    if (!history.value.some((h) => h.startedAt === r.startedAt)) history.value = [r, ...history.value];
    void ov.reload();
  }

  async function loadHistory() {
    try {
      history.value = await api.listRuns();
    } catch {
      // history is decoration on the idle screen; the run itself does not depend on it
    }
  }

  let started = false;
  async function init() {
    if (started) return;
    started = true;
    await Promise.all([on("run://progress", onProgress), on("run://finished", finish)]);
    await loadHistory();
  }

  return {
    phase, preview, progress, result, failure, stopped, stopRequested, history, freeBefore, reached, files,
    totalFiles, fraction, busy, lastRun, activeRoot, failedPath, freed, filesDone,
    startPreview, discard, askConfirm, apply, cancel, retry, init,
  };
});
