// ApiError -> interface copy. The backend `message` is English for logs: people read these instead
// (the run log is the one place raw engine lines are shown).

import type { ApiError, ErrorCode } from "../api/types";
import { t } from "../i18n";

const CODES: ErrorCode[] = ["engine_missing", "target_blocked", "clone_shared", "config_io", "run_in_progress", "run_failed", "schedule_failed", "not_found", "internal"];

export function errorCode(e: unknown): ErrorCode {
  const code = (e as Partial<ApiError> | null)?.code;
  return CODES.includes(code as ErrorCode) ? (code as ErrorCode) : "internal";
}

export function errorMessage(e: unknown): string {
  const m = (e as Partial<ApiError> | null)?.message;
  return typeof m === "string" ? m : e instanceof Error ? e.message : String(e);
}

export const errorText = (e: unknown): string => t(`errors.${errorCode(e)}`);
