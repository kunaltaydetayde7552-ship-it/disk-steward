import { afterEach, beforeEach, describe, expect, spyOn, test } from "bun:test";
import { api, emitInPage } from "../../api/client";
import { mockDocker, mockOverview } from "../../api/mockData";
import { MockBackend } from "../../api/mock";
import { errorText } from "../../lib/errors";
import { flush, freshPinia, stubBrowser } from "../../__tests__/helpers";
import type { Overview } from "../../api/types";
import { useOverview } from "../overview";
import { useUi } from "../ui";

let restoreBrowser: () => void;
let overview: Overview;

async function startStore() {
  const spy = spyOn(api, "getOverview").mockImplementation(async () => structuredClone(overview));
  const ov = useOverview();
  await ov.init();
  return { ov, spy };
}

beforeEach(() => {
  restoreBrowser = stubBrowser();
  freshPinia();
  overview = mockOverview();
});
afterEach(() => {
  restoreBrowser();
  (api.getOverview as { mockRestore?: () => void }).mockRestore?.();
});

describe("overview://measuring and overview://failed", () => {
  test("measuring event enters the measuring state", async () => {
    const { ov } = await startStore();
    emitInPage("overview://measuring", {});
    expect(ov.measurement).toBe("measuring");
  });

  test("failed event leaves the measuring state", async () => {
    const { ov } = await startStore();
    emitInPage("overview://measuring", {});
    emitInPage("overview://failed", { code: "internal", message: "walk failed" });
    expect([ov.measuring, ov.measurement]).toEqual([false, "fresh"]);
  });

  test("failed event exposes the translated error as an error toast", async () => {
    const { ov } = await startStore();
    emitInPage("overview://measuring", {});
    const error = { code: "config_io", message: "permission denied" } as const;
    emitInPage("overview://failed", error);
    expect(useUi().toasts.map((x) => [x.kind, x.text])).toContainEqual(["error", errorText(error)]);
    expect(ov.measuring).toBe(false);
  });

  test("failed with engine_missing flags the missing engine so the UI can offer a retry", async () => {
    const { ov } = await startStore();
    emitInPage("overview://failed", { code: "engine_missing", message: "not found" });
    expect(ov.engineMissing).toBe(true);
  });

  test("failed releases the busy flag of the retry button", async () => {
    const { ov } = await startStore();
    ov.engineMissing = true;
    ov.engineBusy = true;
    emitInPage("overview://failed", { code: "engine_missing", message: "not found" });
    expect(ov.engineBusy).toBe(false);
  });

  test("a later updated event recovers from a failure", async () => {
    const { ov } = await startStore();
    emitInPage("overview://failed", { code: "engine_missing", message: "x" });
    emitInPage("overview://updated", mockOverview());
    expect([ov.engineMissing, ov.measurement]).toEqual([false, "fresh"]);
  });

  test("refresh rejected by the backend stops measuring and toasts", async () => {
    const { ov } = await startStore();
    spyOn(api, "refreshOverview").mockRejectedValue({ code: "internal", message: "boom" });
    await ov.refresh();
    expect(ov.measuring).toBe(false);
    expect(useUi().toasts.at(-1)?.kind).toBe("error");
    (api.refreshOverview as unknown as { mockRestore: () => void }).mockRestore();
  });

  test("refresh while measuring does not call the backend again", async () => {
    const { ov } = await startStore();
    const spy = spyOn(api, "refreshOverview").mockResolvedValue(null);
    emitInPage("overview://measuring", {});
    await ov.refresh();
    expect(spy).not.toHaveBeenCalled();
    spy.mockRestore();
  });
});

describe("measurement state never", () => {
  test("never measured is reported as never, not as measuring", async () => {
    overview.measurement = { state: "never", measuredAt: null };
    const { ov } = await startStore();
    expect([ov.measurement, ov.neverMeasured, ov.measuredAt]).toEqual(["never", true, null]);
  });

  test("with state never, Docker is not available and the store says the sizes were never measured", async () => {
    overview.measurement = { state: "never", measuredAt: null };
    overview.docker = mockDocker(false);
    const { ov } = await startStore();
    expect([ov.docker?.available, ov.neverMeasured]).toEqual([false, true]);
  });

  test("with a fresh measurement and Docker unavailable, the state is not never (Docker is stopped)", async () => {
    overview.docker = mockDocker(false);
    const { ov } = await startStore();
    expect([ov.docker?.available, ov.neverMeasured]).toEqual([false, false]);
  });

  test("the mock backend scenario ?mock=never reports state never with Docker unavailable", async () => {
    const restore = stubBrowser("?mock=never");
    try {
      const o = await new MockBackend().invoke("get_overview", {});
      expect([o.measurement.state, o.docker.available]).toEqual(["never", false]);
    } finally {
      restore();
    }
  });

  test("a cached overview already measuring starts in the measuring state", async () => {
    overview.measurement = { state: "measuring", measuredAt: overview.measurement.measuredAt };
    const { ov } = await startStore();
    expect(ov.measuring).toBe(true);
  });

  test("updated event never leaves a stale measuring state behind", async () => {
    const { ov } = await startStore();
    emitInPage("overview://measuring", {});
    emitInPage("overview://updated", { ...mockOverview(), measurement: { state: "measuring", measuredAt: null } });
    expect([ov.measuring, ov.measurement]).toEqual([false, "fresh"]);
  });
});

describe("load", () => {
  test("a failing get_overview keeps the error for the screen", async () => {
    const ov = useOverview();
    spyOn(api, "getOverview").mockRejectedValue({ code: "internal", message: "db" });
    await ov.load();
    expect([ov.ready, ov.loadError?.code]).toEqual([false, "internal"]);
  });

  test("a successful load exposes targets split by state", async () => {
    const { ov } = await startStore();
    await flush();
    expect([ov.enabled.length, ov.blocked.length, ov.available.length]).toEqual([3, 3, 2]);
  });
});
