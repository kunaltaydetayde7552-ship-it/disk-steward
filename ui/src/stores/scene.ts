// The light of the object, shared by the hero, the sidebar cube, the tray and the window floor.

import { computed } from "vue";
import type { Light } from "../lib/light";
import { useOverview } from "./overview";
import { useRun } from "./run";
import { useUi } from "./ui";

export function useGlobalLight() {
  const ov = useOverview();
  const run = useRun();
  return computed<Light>(() => {
    if (run.phase === "failed") return "red";
    if (run.phase === "running" || run.phase === "previewing") return "amber";
    if (ov.measuring || ov.engineMissing || !ov.enabled.length) return "off";
    return "green";
  });
}

/** The target the Targets screen shows: the route's id, else the first enabled, else the first. */
export function useSelectedTarget() {
  const ov = useOverview();
  const ui = useUi();
  return computed(() => {
    const all = ov.targets;
    return all.find((x) => x.id === ui.route.targetId) ?? all.find((x) => x.enabled) ?? all[0] ?? null;
  });
}
