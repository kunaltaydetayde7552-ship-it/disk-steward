import { afterEach, beforeEach, describe, expect, spyOn, test } from "bun:test";
import { api, CANCELLED, emitInPage } from "../../api/client";
import { mockOverview, PREVIEW } from "../../api/mockData";
import { deferred, flush, freshPinia, stubBrowser } from "../../__tests__/helpers";
import type { RunResult } from "../../api/types";
import { useOverview } from "../overview";
import { useRun } from "../run";
import { useUi } from "../ui";

let restoreBrowser: () => void;

const result = (over: Partial<RunResult> = {}): RunResult => ({
  startedAt: "2026-11-24T10:00:00Z", durationMs: 48200, freedBytes: 198 * 1024 ** 2, preview: structuredClone(PREVIEW), errors: [], ...over,
});

function spies() {
  return {
    preview: spyOn(api, "preview").mockResolvedValue(structuredClone(PREVIEW)),
    apply: spyOn(api, "apply").mockResolvedValue(result()),
    cancelRun: spyOn(api, "cancelRun").mockResolvedValue(null),
    getOverview: spyOn(api, "getOverview").mockResolvedValue(mockOverview()),
    listRuns: spyOn(api, "listRuns").mockResolvedValue([]),
  };
}

function setup() {
  const s = spies();
  const ov = useOverview();
  ov.data = mockOverview();
  return { s, ov, run: useRun(), ui: useUi() };
}

beforeEach(() => {
  restoreBrowser = stubBrowser();
  freshPinia();
});
afterEach(() => {
  restoreBrowser();
  for (const name of ["preview", "apply", "cancelRun", "getOverview", "listRuns"] as const) (api[name] as { mockRestore?: () => void }).mockRestore?.();
});

describe("preview, confirm, apply, done", () => {
  test("preview is read only: it never applies and waits for confirmation", async () => {
    const { s, run, ui } = setup();
    await run.startPreview();
    expect([run.phase, run.preview?.files, ui.confirmOpen, s.apply.mock.calls.length]).toEqual(["preview", PREVIEW.files, false, 0]);
  });

  test("askConfirm opens the dialog and still compresses nothing", async () => {
    const { s, run, ui } = setup();
    await run.startPreview();
    run.askConfirm();
    expect([ui.confirmOpen, run.phase, s.apply.mock.calls.length]).toEqual([true, "preview", 0]);
  });

  test("apply finishes in done with the result, the history and a refreshed overview", async () => {
    const { s, run } = setup();
    await run.startPreview();
    await run.apply();
    expect([run.phase, run.freed, run.history.length, s.apply.mock.calls.length, s.getOverview.mock.calls.length]).toEqual(["done", 198 * 1024 ** 2, 1, 1, 1]);
  });

  test("apply records the free space from before the run", async () => {
    const { ov, run } = setup();
    await run.apply();
    expect(run.freeBefore).toBe(ov.disk!.freeBytes);
  });

  test("progress events fill the run state and never go backwards", async () => {
    const { run } = setup();
    await run.init();
    await run.startPreview();
    emitInPage("run://progress", { phase: "compressing", files: 600, totalFiles: 1312, bytes: 1, currentPath: `${PREVIEW.roots[0].path}/a` });
    emitInPage("run://progress", { phase: "compressing", files: 500, totalFiles: null, bytes: 1, currentPath: `${PREVIEW.roots[1].path}/b` });
    expect([run.phase, run.files, run.totalFiles, run.activeRoot]).toEqual(["running", 600, 1312, PREVIEW.roots[1].path]);
  });

  test("run://finished from another window completes the run once, even if repeated", async () => {
    const { run } = setup();
    await run.init();
    emitInPage("run://finished", result());
    emitInPage("run://finished", result());
    expect([run.phase, run.history.length]).toEqual(["done", 1]);
  });

  test("discard returns to idle after the result", async () => {
    const { run } = setup();
    await run.apply();
    run.discard();
    expect([run.phase, run.preview, run.result]).toEqual(["idle", null, null]);
  });
});

describe("preview edge cases", () => {
  test("without enabled targets there is no preview", async () => {
    const { s, ov, run } = setup();
    ov.data!.targets.forEach((t) => (t.enabled = false));
    await run.startPreview();
    expect([run.phase, s.preview.mock.calls.length]).toEqual(["idle", 0]);
  });

  test("a preview with 0 files does not open the confirmation", async () => {
    const { s, run, ui } = setup();
    s.preview.mockResolvedValue({ files: 0, bytes: 0, skipped: 0, roots: [] });
    await run.startPreview(true);
    run.askConfirm();
    expect([run.phase, ui.confirmOpen]).toEqual(["preview", false]);
  });

  test("a failing preview returns to idle with an error toast", async () => {
    const { s, run, ui } = setup();
    s.preview.mockRejectedValue({ code: "internal", message: "boom" });
    await run.startPreview();
    expect([run.phase, ui.toasts.at(-1)?.kind]).toEqual(["idle", "error"]);
  });

  test("a missing engine goes to idle without a toast and flags the overview", async () => {
    const { s, ov, run, ui } = setup();
    s.preview.mockRejectedValue({ code: "engine_missing", message: "x" });
    await run.startPreview();
    expect([run.phase, ui.toasts.length, ov.engineMissing]).toEqual(["idle", 0, true]);
  });
});

describe("failure", () => {
  test("apply rejected before a result goes to failed with the error", async () => {
    const { s, run } = setup();
    s.apply.mockRejectedValue({ code: "target_blocked", message: "uv-cache is blocked" });
    await run.apply();
    expect([run.phase, run.failure]).toEqual(["failed", { code: "target_blocked", message: "uv-cache is blocked" }]);
  });

  test("a result with an ERROR line is a failed run and points at the file", async () => {
    const { s, run } = setup();
    const path = "/Users/dev/.npm/_cacache/content-v2/sha512/9c";
    s.apply.mockResolvedValue(result({ errors: [`ERROR verify: ${path}… differs from the original`] }));
    await run.apply();
    expect([run.phase, run.failedPath, run.stopped]).toEqual(["failed", path, false]);
  });

  test("WARN-only lines do not fail the run", async () => {
    const { s, run } = setup();
    s.apply.mockResolvedValue(result({ errors: ["WARN skipped a busy file"] }));
    await run.apply();
    expect(run.phase).toBe("done");
  });

  test("a missing engine returns to idle instead of failed", async () => {
    const { s, run } = setup();
    s.apply.mockRejectedValue({ code: "engine_missing", message: "x" });
    await run.apply();
    expect(run.phase).toBe("idle");
  });

  test("retry runs a fresh preview and asks for confirmation again, never applying", async () => {
    const { s, run, ui } = setup();
    s.apply.mockResolvedValue(result({ errors: ["ERROR verify: /x differs"] }));
    await run.apply();
    await run.retry();
    expect([run.phase, ui.confirmOpen, s.apply.mock.calls.length]).toEqual(["preview", true, 1]);
  });
});

describe("cancel", () => {
  test("cancel while running asks the backend once", async () => {
    const { s, run } = setup();
    const gate = deferred<RunResult>();
    s.apply.mockReturnValue(gate.promise);
    const running = run.apply();
    await Promise.all([run.cancel(), run.cancel()]);
    expect([s.cancelRun.mock.calls.length, run.stopRequested]).toEqual([1, true]);
    gate.resolve(result());
    await running;
  });

  test("cancel outside a run does nothing", async () => {
    const { s, run } = setup();
    await run.cancel();
    expect(s.cancelRun.mock.calls.length).toBe(0);
  });

  test("a cancelled run ends as a stop, not as a failure", async () => {
    const { s, run } = setup();
    await run.init();
    const gate = deferred<RunResult>();
    s.apply.mockReturnValue(gate.promise);
    const running = run.apply();
    emitInPage("run://finished", result({ errors: [CANCELLED] }));
    gate.reject({ code: "run_failed", message: CANCELLED });
    await running;
    expect([run.phase, run.stopped, run.failure]).toEqual(["done", true, null]);
  });

  test("a rejected cancel (not_found) allows trying again without a toast", async () => {
    const { s, run, ui } = setup();
    const gate = deferred<RunResult>();
    s.apply.mockReturnValue(gate.promise);
    const running = run.apply();
    s.cancelRun.mockRejectedValue({ code: "not_found", message: "no run" });
    await run.cancel();
    expect([run.stopRequested, ui.toasts.length]).toEqual([false, 0]);
    gate.resolve(result());
    await running;
  });
});

describe("second apply", () => {
  test("apply while running does not reach the backend again", async () => {
    const { s, run } = setup();
    const gate = deferred<RunResult>();
    s.apply.mockReturnValue(gate.promise);
    const first = run.apply();
    await run.apply();
    expect([s.apply.mock.calls.length, run.phase]).toEqual([1, "running"]);
    gate.resolve(result());
    await first;
  });

  test("a run_in_progress rejection (another window) keeps the run phase", async () => {
    const { s, run } = setup();
    s.apply.mockRejectedValue({ code: "run_in_progress", message: "busy" });
    await run.apply();
    expect([run.phase, run.failure]).toEqual(["running", null]);
  });

  test("preview is refused while a run is in progress", async () => {
    const { s, run } = setup();
    const gate = deferred<RunResult>();
    s.apply.mockReturnValue(gate.promise);
    const running = run.apply();
    await run.startPreview();
    expect([s.preview.mock.calls.length, run.phase]).toEqual([0, "running"]);
    gate.resolve(result());
    await running;
  });
});

describe("tray://compress-now contract", () => {
  // The listener lives in App.vue (a component, not loadable by bun). These tests cover the store calls it makes.
  test("starting a preview with confirmation opens the dialog and never applies", async () => {
    const { s, run, ui } = setup();
    await run.startPreview(true);
    await flush();
    expect([run.phase, ui.confirmOpen, s.apply.mock.calls.length]).toEqual(["preview", true, 0]);
  });

  test("a preview that is already ready only asks for confirmation", async () => {
    const { s, run, ui } = setup();
    await run.startPreview();
    run.askConfirm();
    expect([ui.confirmOpen, s.preview.mock.calls.length, s.apply.mock.calls.length]).toEqual([true, 1, 0]);
  });

  test("the tray event itself does not start any compression in the stores", async () => {
    const { s, run } = setup();
    await run.init();
    emitInPage("tray://compress-now", {});
    await flush();
    expect([run.phase, s.apply.mock.calls.length]).toEqual(["idle", 0]);
  });
});
