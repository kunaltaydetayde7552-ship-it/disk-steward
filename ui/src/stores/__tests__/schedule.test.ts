import { afterEach, beforeEach, describe, expect, spyOn, test } from "bun:test";
import { api } from "../../api/client";
import { mockOverview, mockSchedule } from "../../api/mockData";
import { deferred, freshPinia, stubBrowser } from "../../__tests__/helpers";
import type { ScheduleAgent } from "../../api/types";
import { COMPRESS_AGENT, useSchedule } from "../schedule";
import { useOverview } from "../overview";
import { useUi } from "../ui";

let restoreBrowser: () => void;

const withCompress = (loaded: boolean): ScheduleAgent[] => mockSchedule().map((a) => (a.label === COMPRESS_AGENT ? { ...a, loaded } : a));

function setup() {
  const ov = useOverview();
  ov.data = mockOverview();
  return { ov, schedule: useSchedule(), ui: useUi() };
}

beforeEach(() => {
  restoreBrowser = stubBrowser();
  freshPinia();
});
afterEach(() => {
  restoreBrowser();
  for (const name of ["installSchedule", "uninstallSchedule"] as const) (api[name] as { mockRestore?: () => void }).mockRestore?.();
});

describe("schedule store", () => {
  test("starts from the overview list: not installed", () => {
    expect(setup().schedule.installed).toBe(false);
  });

  test("install succeeds: installed, toast ok, no error, not busy", async () => {
    const { schedule, ui } = setup();
    spyOn(api, "installSchedule").mockResolvedValue(withCompress(true));
    await schedule.install();
    expect([schedule.installed, schedule.error, schedule.busy, ui.toasts.at(-1)?.kind]).toEqual([true, null, null, "ok"]);
  });

  test("uninstall succeeds: no longer installed", async () => {
    const { schedule, ui } = setup();
    spyOn(api, "installSchedule").mockResolvedValue(withCompress(true));
    spyOn(api, "uninstallSchedule").mockResolvedValue(withCompress(false));
    await schedule.install();
    await schedule.uninstall();
    expect([schedule.installed, schedule.busy, ui.toasts.at(-1)?.kind]).toEqual([false, null, "ok"]);
  });

  test("install failure keeps the state, exposes the error and does not toast success", async () => {
    const { schedule, ui } = setup();
    spyOn(api, "installSchedule").mockRejectedValue({ code: "schedule_failed", message: "launchctl bootstrap failed" });
    await schedule.install();
    expect(schedule.installed).toBe(false);
    expect(schedule.error).toEqual({ code: "schedule_failed", message: "launchctl bootstrap failed" });
    expect([schedule.busy, ui.toasts.length]).toEqual([null, 0]);
  });

  test("uninstall failure leaves the agent installed", async () => {
    const { schedule } = setup();
    spyOn(api, "installSchedule").mockResolvedValue(withCompress(true));
    spyOn(api, "uninstallSchedule").mockRejectedValue({ code: "schedule_failed", message: "x" });
    await schedule.install();
    await schedule.uninstall();
    expect([schedule.installed, schedule.error?.code]).toEqual([true, "schedule_failed"]);
  });

  test("an engine_missing failure also flags the overview store", async () => {
    const { schedule, ov } = setup();
    spyOn(api, "installSchedule").mockRejectedValue({ code: "engine_missing", message: "x" });
    await schedule.install();
    expect(ov.engineMissing).toBe(true);
  });

  test("a new attempt clears the previous error", async () => {
    const { schedule } = setup();
    const spy = spyOn(api, "installSchedule").mockRejectedValueOnce({ code: "schedule_failed", message: "x" });
    await schedule.install();
    spy.mockResolvedValue(withCompress(true));
    await schedule.install();
    expect([schedule.error, schedule.installed]).toEqual([null, true]);
  });

  test("a second install while one is running is ignored", async () => {
    const { schedule } = setup();
    const gate = deferred<ScheduleAgent[]>();
    const spy = spyOn(api, "installSchedule").mockReturnValue(gate.promise);
    const first = schedule.install();
    await schedule.install();
    expect([spy.mock.calls.length, schedule.busy]).toEqual([1, "install"]);
    gate.resolve(withCompress(true));
    await first;
  });
});
