// Wire types of docs/API.md (camelCase keys, snake_case enum values) plus the shell payloads of src-tauri.

export interface DiskStatus {
  volume: string;
  totalBytes: number;
  freeBytes: number;
  usedBytes: number;
  measuredAt: string;
}

export type TargetKind = "toolchain" | "package_cache" | "store" | "build" | "custom";
export type Risk = "none" | "clone_shared" | "rewritten_often" | "blocked_by_rule";

export interface Target {
  id: string;
  path: string;
  kind: TargetKind;
  enabled: boolean;
  allowed: boolean;
  logicalBytes: number;
  onDiskBytes: number;
  /** null when the filesystem refuses the attribute, when not measured, and for pseudo targets */
  privateBytes: number | null;
  totalFiles: number;
  compressedFiles: number;
  risk: Risk;
  /** one-line English tooltip from the backend; never translated */
  riskDetail: string | null;
}

export interface Savings {
  files: number;
  originalBytes: number;
  onDiskBytes: number;
  savedBytes: number;
  ratio: number;
  quarantined: number;
  lastRunDay: string | null;
  /** engine savings not explained by the enabled targets (deleted, rewritten or no longer targeted files) */
  unattributedBytes: number;
}

export interface PreviewRoot {
  path: string;
  bytes: number;
}

export interface Preview {
  files: number;
  bytes: number;
  skipped: number;
  roots: PreviewRoot[];
}

export interface RunResult {
  startedAt: string;
  durationMs: number;
  /** free-space delta of the data volume: can be negative or noisy */
  freedBytes: number;
  preview: Preview;
  /** engine ERROR/WARN lines; shown raw in the run log only */
  errors: string[];
}

export interface ScheduleAgent {
  label: string;
  loaded: boolean;
  lastExitStatus: number | null;
  logPath: string | null;
}

export interface DockerUsage {
  available: boolean;
  imagesBytes: number;
  buildCacheBytes: number;
  volumesBytes: number;
  reclaimableBytes: number;
}

export type MeasurementState = "fresh" | "stale" | "measuring" | "never";

export interface Measurement {
  state: MeasurementState;
  measuredAt: string | null;
}

export interface Overview {
  disk: DiskStatus;
  savings: Savings;
  targets: Target[];
  schedule: ScheduleAgent[];
  docker: DockerUsage;
  measurement: Measurement;
}

export type RunPhase = "starting" | "compressing" | "verifying" | "done" | "failed";

export interface RunProgress {
  phase: RunPhase;
  files: number | null;
  totalFiles: number | null;
  bytes: number | null;
  currentPath: string | null;
}

export type ErrorCode =
  | "engine_missing"
  | "target_blocked"
  | "clone_shared"
  | "config_io"
  | "run_in_progress"
  | "run_failed"
  | "schedule_failed"
  | "not_found"
  | "internal";

/** Every command rejects with this shape; `message` is English for logs. */
export interface ApiError {
  code: ErrorCode;
  message: string;
}

/** The backend marks a run stopped by `cancel_run` with this line in `errors`. */
export const CANCELLED = "cancelled";

export interface CommandMap {
  get_overview: { args: Record<string, never>; result: Overview };
  refresh_overview: { args: Record<string, never>; result: null };
  set_target_enabled: { args: { id: string; enabled: boolean }; result: Target };
  preview: { args: Record<string, never>; result: Preview };
  apply: { args: Record<string, never>; result: RunResult };
  cancel_run: { args: Record<string, never>; result: null };
  list_runs: { args: Record<string, never>; result: RunResult[] };
  get_schedule: { args: Record<string, never>; result: ScheduleAgent[] };
  install_schedule: { args: Record<string, never>; result: ScheduleAgent[] };
  uninstall_schedule: { args: Record<string, never>; result: ScheduleAgent[] };
  reveal_in_finder: { args: { path: string }; result: null };
}
export type CommandName = keyof CommandMap;

export interface EventMap {
  "overview://measuring": Record<string, never>;
  "overview://updated": Overview;
  "overview://failed": ApiError;
  "run://progress": RunProgress;
  "run://finished": RunResult;
  /** the tray menu item "Comprimir agora": the UI previews and asks; the click compresses nothing */
  "tray://compress-now": Record<string, never>;
  /** sent by the tray popup to the main window (src/api/shell.ts) */
  "ui://navigate": { path: string; action: "preview" | null };
}
export type EventName = keyof EventMap;
