// Target naming and the derived numbers every screen reads the same way.

import type { Target } from "../api/types";
import { t, i18n } from "../i18n";
import { basename, tilde } from "./format";

const known = (key: string) => i18n.global.te(key, "pt-BR");

/** "Toolchains do Rust"; custom targets show their folder name. */
export const targetName = (x: Pick<Target, "id" | "path">): string =>
  known(`targets.names.${x.id}`) ? t(`targets.names.${x.id}`) : basename(x.path);

/** "rustup", the tool people type. */
export const targetShort = (x: Pick<Target, "id" | "path">): string =>
  known(`targets.short.${x.id}`) ? t(`targets.short.${x.id}`) : basename(x.path);

export const kindLabel = (x: Pick<Target, "kind">): string => t(`targets.kinds.${x.kind}`);
export const targetPath = (x: Pick<Target, "path">): string => tilde(x.path);

/** Bytes this folder shares with clones elsewhere (0 when the private size is unknown). */
export const sharedBytes = (x: Target): number => (x.privateBytes == null ? 0 : Math.max(0, x.onDiskBytes - x.privateBytes));

/** What compression already returned on this folder. */
export const savedBytes = (x: Target): number => Math.max(0, x.logicalBytes - x.onDiskBytes);
export const savedRatio = (x: Target): number => (x.logicalBytes > 0 ? savedBytes(x) / x.logicalBytes : 0);

/** The venv count the backend writes in its English riskDetail ("23 venvs hold ..."); null when absent. */
export function cloneCount(x: Target): number | null {
  const m = x.riskDetail?.match(/(\d+)\s+venvs?/i);
  return m ? Number(m[1]) : null;
}

export const isPseudo = (x: Target): boolean => x.risk === "rewritten_often";
export const isMeasured = (x: Target): boolean => isPseudo(x) || x.logicalBytes > 0 || x.totalFiles > 0 || x.privateBytes != null;

/** The light of a target's cube: green compressing, unlit off, cold blocked, wire pseudo, red when the run failed there. */
export type CubeLight = "green" | "amber" | "red" | "unlit" | "cold" | "wire";
export function targetLight(x: Target, failedPath: string | null): CubeLight {
  if (isPseudo(x)) return "wire";
  if (!x.allowed) return "cold";
  if (!x.enabled) return "unlit";
  return failedPath && failedPath.startsWith(x.path) ? "red" : "green";
}
