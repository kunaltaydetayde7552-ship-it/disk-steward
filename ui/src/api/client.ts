// Thin typed wrapper over @tauri-apps/api. Outside Tauri (plain browser) it talks to the in-memory mock.

import { MockBackend } from "./mock";
import type { CommandMap, CommandName, EventMap, EventName, Overview, Preview, RunResult, ScheduleAgent, Target } from "./types";

export * from "./types";

export const isTauri = (): boolean => typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

let mockInstance: MockBackend | null = null;
function mock(): MockBackend {
  if (!mockInstance) {
    mockInstance = new MockBackend();
    if (import.meta.env.DEV) (window as unknown as { __stewardMock: MockBackend }).__stewardMock = mockInstance;
  }
  return mockInstance;
}

async function call<K extends CommandName>(cmd: K, args: CommandMap[K]["args"] = {} as CommandMap[K]["args"]): Promise<CommandMap[K]["result"]> {
  if (!isTauri()) return mock().invoke(cmd, args);
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<CommandMap[K]["result"]>(cmd, args as Record<string, unknown>);
}

/** Subscribe to a backend event. Returns an unsubscribe function. */
export async function on<E extends EventName>(event: E, handler: (payload: EventMap[E]) => void): Promise<() => void> {
  if (!isTauri()) return mock().listen(event, handler);
  const { listen } = await import("@tauri-apps/api/event");
  return listen<EventMap[E]>(event, (e) => handler(e.payload));
}

/** Plain browser only: delivers an event inside this page (the tray view and the main window share it). */
export function emitInPage<E extends EventName>(event: E, payload: EventMap[E]): void {
  if (!isTauri()) mock().emit(event, payload);
}

export const api = {
  /** Cached overview: instant. Free space is always current; sizes carry `measurement`. */
  getOverview: (): Promise<Overview> => call("get_overview"),
  /** Starts a full measurement; answers right away, then `overview://measuring` and `overview://updated` (or `overview://failed`). */
  refreshOverview: (): Promise<null> => call("refresh_overview"),
  setTargetEnabled: (id: string, enabled: boolean): Promise<Target> => call("set_target_enabled", { id, enabled }),
  /** Read-only: lists what would be compressed now. */
  preview: (): Promise<Preview> => call("preview"),
  /** Compresses for real. Only after the person confirmed the preview. */
  apply: (): Promise<RunResult> => call("apply"),
  cancelRun: (): Promise<null> => call("cancel_run"),
  listRuns: (): Promise<RunResult[]> => call("list_runs"),
  getSchedule: (): Promise<ScheduleAgent[]> => call("get_schedule"),
  installSchedule: (): Promise<ScheduleAgent[]> => call("install_schedule"),
  uninstallSchedule: (): Promise<ScheduleAgent[]> => call("uninstall_schedule"),
  revealInFinder: (path: string): Promise<null> => call("reveal_in_finder", { path }),
};
