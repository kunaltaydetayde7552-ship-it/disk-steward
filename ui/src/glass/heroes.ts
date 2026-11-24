// What the object shows on each screen: one mapping from app state to block parameters.

import type { Target } from "../api/types";
import { sharedBytes, isPseudo } from "../lib/targets";
import type { Light } from "../lib/light";
import { UNIT as U } from "./scene";
import type { BlockParams } from "./geometry";

const WIRE: BlockParams = { mode: "wire", H: U * 0.5, G: U * 0.5, priv: 0, shared: 0, pb: U * 0.5 + 60, pool: 0.15, light: "off" };

/** The data volume: light is data in use, clear glass is free space. */
export function overviewHero(used: number, total: number, light: Light, measuring: boolean): BlockParams {
  if (measuring) return { mode: "scan", H: U, G: U, priv: 0, shared: 0, pb: null, pool: 0.5, light: "off" };
  const off = light === "off";
  return {
    mode: "solid", H: U, G: U, priv: total > 0 ? (used / total) * (U - 14) : 0, shared: 0, pb: null,
    pool: off ? 0.3 : 0.85, light: off ? "off" : "lit", pulse: light === "amber", seam: light === "red",
  };
}

/** One folder: glass height is bytes on disk, the ghost is the logical size, the core is private bytes. */
export function targetHero(t: Target, measuring: boolean): BlockParams {
  if (isPseudo(t)) return { mode: "wire", H: U * 0.62, G: U * 0.62, priv: 0, shared: 0, pb: U * 0.62 + 46, posts: true, lock: true, pool: 0.2, light: "off" };
  const blocked = !t.allowed;
  if (measuring || t.logicalBytes <= 0) {
    return { mode: "scan", H: U * 0.7, G: U * 0.7, priv: 0, shared: 0, pb: blocked ? U * 0.7 + 34 : null, posts: blocked, lock: blocked, pool: 0.3, light: "off" };
  }
  const k = (U - 12) / t.logicalBytes;
  const H = t.onDiskBytes * k + 8;
  const lit = t.enabled;
  return {
    mode: "solid", H, G: U, priv: (t.privateBytes ?? t.onDiskBytes) * k, shared: sharedBytes(t) * k,
    clones: t.risk === "clone_shared" ? 23 : t.risk === "blocked_by_rule" ? 14 : 0,
    pb: blocked ? U + 34 : lit ? null : U + 40, posts: blocked, lock: blocked, cold: blocked,
    pool: lit ? 0.85 : 0.28, light: lit ? "lit" : "off",
  };
}

export interface RunHeroInput {
  phase: "idle" | "previewing" | "preview" | "running" | "done" | "failed";
  /** share of the preview processed, 0..1 */
  fraction: number;
  /** freed / previewed bytes, the height the block ends at */
  ratio: number;
  stopped: boolean;
  /** enabled targets' on-disk / logical bytes, for the idle block */
  enabledRatio: number | null;
}

/** The run: the platen comes down as files are swapped. */
export function runHero(r: RunHeroInput): BlockParams {
  const end = U * Math.min(1, Math.max(0.35, 1 - r.ratio));
  const at = (k: number) => U - (U - end) * k;
  switch (r.phase) {
    case "previewing":
      return { mode: "scan", H: U, G: U, priv: 0, shared: 0, pb: U + 60, pool: 0.5, light: "off" };
    case "preview":
      return { mode: "solid", H: U, G: U, priv: U - 14, shared: 0, pb: U + 46, pool: 0.4, light: "off" };
    case "running": {
      const H = at(r.fraction);
      return { mode: "solid", H, G: U, priv: H - 14, shared: 0, pb: H, pool: 0.75, light: "lit", pulse: true };
    }
    case "done": {
      const H = at(r.stopped ? r.fraction : 1);
      return { mode: "solid", H, G: U, priv: H - 14, shared: 0, pb: H + 44, pool: 0.85, light: "lit" };
    }
    case "failed": {
      const H = at(r.fraction);
      return { mode: "solid", H, G: U, priv: H - 14, shared: 0, pb: H + 34, pool: 0.7, light: "lit", seam: true };
    }
    default: {
      if (r.enabledRatio == null) return WIRE;
      const H = Math.max(U * 0.3, r.enabledRatio * U);
      return { mode: "solid", H, G: U, priv: H - 14, shared: 0, pb: H + 56, pool: 0.7, light: "lit" };
    }
  }
}

/** The schedule: the press waits above the enabled targets, lit when the daily agent is loaded. */
export function scheduleHero(enabledRatio: number | null, installed: boolean): BlockParams {
  if (enabledRatio == null) return WIRE;
  const H = Math.max(U * 0.3, enabledRatio * U);
  return { mode: "solid", H, G: U, priv: H - 14, shared: 0, pb: H + 72, pool: installed ? 0.7 : 0.3, light: installed ? "lit" : "off" };
}

export function enabledRatio(targets: Target[]): number | null {
  const on = targets.filter((t) => t.enabled);
  const logical = on.reduce((s, t) => s + t.logicalBytes, 0);
  const disk = on.reduce((s, t) => s + t.onDiskBytes, 0);
  if (!on.length) return null;
  return logical > 0 ? disk / logical : 0.85;
}
