// In-memory backend for a plain browser (no Tauri). Same commands, events and error shapes as src-tauri.
// Scenarios come from the query string, comma separated: ?mock=fail,docker-off
//   never           sizes never measured (measurement.state "never")
//   stale           sizes measured three days ago
//   measuring       a measurement is running when the app opens (lasts 20 s)
//   engine-missing  the engine binary is not found until "Procurar de novo" (refresh_overview) finds it
//   none-enabled    no target enabled
//   docker-off      Docker not running
//   fail            the next run stops at a verification failure (only once)
//   schedule-fail   installing the daily agent fails; docker-gc last exit 1
//   installed       the daily agent is already installed
//   no-history      no run has happened yet

import {
  FAILED_AT_FILES, FAILED_ERRORS, FAILED_FREED, MOCK_HOME, PREVIEW, ROOT_FILES, RUN_DURATION_MS, RUN_FREED, STALE_AT,
  mockDocker, mockHistory, mockOverview,
} from "./mockData";
import {
  CANCELLED, type ApiError, type CommandMap, type CommandName, type ErrorCode, type EventMap, type EventName,
  type Overview, type RunProgress, type RunResult, type ScheduleAgent, type Target,
} from "./types";

type Handler = (payload: never) => void;

const RUN_SECONDS = 9;
const TICK_MS = 120;

const clone = <T>(v: T): T => structuredClone(v);
const sleep = (ms: number) => new Promise<void>((r) => setTimeout(r, ms));
const fail = (code: ErrorCode, message: string): ApiError => ({ code, message });

function scenarios(): Set<string> {
  const raw = typeof location === "undefined" ? "" : new URLSearchParams(location.search).get("mock") ?? "";
  return new Set(raw.split(",").map((s) => s.trim()).filter(Boolean));
}

export class MockBackend {
  private flags = scenarios();
  private overview: Overview = mockOverview();
  private history: RunResult[] = this.flags.has("no-history") ? [] : mockHistory();
  private listeners = new Map<EventName, Set<Handler>>();
  private engineMissing = this.flags.has("engine-missing");
  private failNext = this.flags.has("fail");
  private measuring = false;
  private run: { cancel: boolean } | null = null;

  constructor() {
    const o = this.overview;
    if (this.flags.has("none-enabled")) o.targets.forEach((t) => (t.enabled = false));
    if (this.flags.has("docker-off")) o.docker = mockDocker(false);
    if (this.flags.has("installed")) o.schedule[0] = { ...o.schedule[0], loaded: true, lastExitStatus: 0 };
    if (this.flags.has("schedule-fail")) o.schedule[1] = { ...o.schedule[1], lastExitStatus: 1 };
    if (this.flags.has("stale")) o.measurement = { state: "stale", measuredAt: STALE_AT };
    if (this.flags.has("never")) {
      o.measurement = { state: "never", measuredAt: null };
      o.savings.unattributedBytes = 0;
      // docker system df only runs with a measured overview
      o.docker = mockDocker(false);
      o.targets.forEach((t) => Object.assign(t, { logicalBytes: 0, onDiskBytes: 0, privateBytes: null, totalFiles: 0, compressedFiles: 0 }));
    }
    if (this.flags.has("measuring")) this.startMeasuring(20_000);
  }

  listen<E extends EventName>(event: E, handler: (payload: EventMap[E]) => void): () => void {
    const set = this.listeners.get(event) ?? new Set<Handler>();
    set.add(handler as Handler);
    this.listeners.set(event, set);
    return () => set.delete(handler as Handler);
  }

  /** Also used by the screenshot driver through window.__stewardMock in dev builds. */
  emit<E extends EventName>(event: E, payload: EventMap[E]): void {
    this.listeners.get(event)?.forEach((h) => (h as (p: EventMap[E]) => void)(clone(payload)));
  }

  async invoke<K extends CommandName>(cmd: K, args: CommandMap[K]["args"]): Promise<CommandMap[K]["result"]> {
    const result = await this.handle(cmd, args as Record<string, unknown>);
    return clone(result) as CommandMap[K]["result"];
  }

  private async handle(cmd: CommandName, args: Record<string, unknown>): Promise<unknown> {
    switch (cmd) {
      case "get_overview":
        await sleep(160);
        return { ...this.overview, measurement: this.measuring ? { ...this.overview.measurement, state: "measuring" } : this.overview.measurement };
      case "refresh_overview":
        this.startMeasuring(3200);
        return null;
      case "set_target_enabled":
        await sleep(260);
        return this.setEnabled(String(args.id), Boolean(args.enabled));
      case "preview":
        await sleep(1400);
        if (this.engineMissing) throw fail("engine_missing", "cannot run auto-compress-apfs: No such file or directory (os error 2)");
        if (!this.overview.targets.some((t) => t.enabled)) return { files: 0, bytes: 0, skipped: 0, roots: [] };
        return PREVIEW;
      case "apply":
        return this.apply();
      case "cancel_run":
        if (!this.run) throw fail("not_found", "no run in progress");
        this.run.cancel = true;
        return null;
      case "list_runs":
        await sleep(120);
        return this.history;
      case "get_schedule":
        await sleep(120);
        return this.overview.schedule;
      case "install_schedule":
        return this.setSchedule(true);
      case "uninstall_schedule":
        return this.setSchedule(false);
      case "reveal_in_finder":
        await sleep(80);
        if (!String(args.path).startsWith("/")) throw fail("not_found", `${String(args.path)} does not exist`);
        return null;
    }
  }

  private startMeasuring(ms: number) {
    if (this.measuring) return;
    this.measuring = true;
    this.emit("overview://measuring", {});
    setTimeout(() => {
      if (this.engineMissing && ms < 20_000) {
        // shortcut: the mock "finds" the engine on the first retry so the recovery path can be shown
        this.engineMissing = false;
      }
      this.measuring = false;
      const fresh = mockOverview();
      const o = this.overview;
      o.targets = o.targets.map((t) => {
        const m = fresh.targets.find((f) => f.id === t.id);
        return m ? { ...m, enabled: t.enabled } : t;
      });
      o.savings.unattributedBytes = fresh.savings.unattributedBytes;
      if (!this.flags.has("docker-off")) o.docker = fresh.docker;
      o.measurement = { state: "fresh", measuredAt: new Date().toISOString().replace(/\.\d+Z$/, "Z") };
      o.disk = { ...o.disk, measuredAt: o.measurement.measuredAt ?? o.disk.measuredAt };
      this.emit("overview://updated", o);
    }, ms);
  }

  private setEnabled(id: string, enabled: boolean): Target {
    if (this.run) throw fail("run_in_progress", "a run is in progress");
    const t = this.overview.targets.find((x) => x.id === id);
    if (!t) throw fail("not_found", `unknown target \`${id}\``);
    if (enabled && t.risk === "clone_shared") throw fail("clone_shared", t.riskDetail ?? "");
    if (enabled && !t.allowed) throw fail("target_blocked", t.riskDetail ?? "");
    t.enabled = enabled;
    return t;
  }

  private async setSchedule(install: boolean): Promise<ScheduleAgent[]> {
    await sleep(1100);
    if (install && this.engineMissing) throw fail("engine_missing", "cannot run auto-compress-apfs: No such file or directory (os error 2)");
    if (install && this.flags.has("schedule-fail")) throw fail("schedule_failed", "launchctl bootstrap gui/501 failed: Input/output error (5)");
    const s = this.overview.schedule;
    s[0] = { ...s[0], loaded: install, lastExitStatus: null };
    return s;
  }

  private progress(p: Partial<RunProgress> & Pick<RunProgress, "phase">) {
    this.emit("run://progress", { files: null, totalFiles: PREVIEW.files, bytes: null, currentPath: null, ...p });
  }

  private async apply(): Promise<RunResult> {
    if (this.run) throw fail("run_in_progress", "a run is already in progress");
    if (this.engineMissing) throw fail("engine_missing", "cannot run auto-compress-apfs: No such file or directory (os error 2)");
    const run = { cancel: false };
    this.run = run;
    const startedAt = new Date().toISOString().replace(/\.\d+Z$/, "Z");
    const t0 = performance.now();
    const failing = this.failNext;
    this.failNext = false;
    const total = PREVIEW.files;
    const stopAt = failing ? FAILED_AT_FILES : total;
    this.progress({ phase: "starting" });
    await sleep(500);

    let files = 0;
    let bytes = 0;
    const perTick = total / ((RUN_SECONDS * 1000) / TICK_MS);
    while (files < stopAt && !run.cancel) {
      await sleep(TICK_MS);
      files = Math.min(stopAt, Math.round(files + perTick));
      bytes = Math.round((files / total) * PREVIEW.bytes);
      this.progress({ phase: "compressing", files, bytes, currentPath: this.pathAt(files) });
    }

    const errors = failing ? [...FAILED_ERRORS] : run.cancel ? [CANCELLED] : [];
    if (!failing && !run.cancel) {
      this.progress({ phase: "verifying", files, bytes });
      await sleep(600);
    }
    const share = files / total;
    const freedBytes = failing ? FAILED_FREED : Math.round(RUN_FREED * share);
    const result: RunResult = {
      startedAt, durationMs: failing ? 21900 : run.cancel ? Math.round(performance.now() - t0) : RUN_DURATION_MS,
      freedBytes, preview: clone(PREVIEW), errors,
    };
    this.overview.disk = { ...this.overview.disk, freeBytes: this.overview.disk.freeBytes + freedBytes, usedBytes: this.overview.disk.usedBytes - freedBytes };
    this.overview.savings = { ...this.overview.savings, savedBytes: this.overview.savings.savedBytes + freedBytes, files: this.overview.savings.files + files };
    this.history.unshift(result);
    this.run = null;
    this.progress({ phase: failing || run.cancel ? "failed" : "done", files, bytes });
    this.emit("run://finished", result);
    if (run.cancel) throw fail("run_failed", CANCELLED);
    return result;
  }

  /** The file being compressed after `files` files, inside the root that holds it. */
  private pathAt(files: number): string {
    let acc = 0;
    for (let i = 0; i < PREVIEW.roots.length; i++) {
      acc += ROOT_FILES[i];
      if (files <= acc) return `${PREVIEW.roots[i].path}/${MOCK_SAMPLE[i]}/${files}`;
    }
    return `${MOCK_HOME}/.rustup`;
  }
}

const MOCK_SAMPLE = ["index.crates.io-6f17d22bba15001f/serde-1.0.215/src", "_cacache/content-v2/sha512", "toolchains/stable-aarch64-apple-darwin/lib"];
