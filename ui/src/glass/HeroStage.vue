<script setup lang="ts">
// The signature: one glass block that travels between the [data-hero] slot of each screen, carrying its
// light, with a pool of that light on the window floor. The stage covers the scroll viewport only (the
// dock sits below it in flow), so the block can never slide under the dock.
import { animate } from "motion-v";
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { bytes } from "../lib/format";
import { LIGHT_VAR, type Light } from "../lib/light";
import { targetName } from "../lib/targets";
import { useOverview } from "../stores/overview";
import { useRun } from "../stores/run";
import { useGlobalLight, useSelectedTarget } from "../stores/scene";
import { useSchedule } from "../stores/schedule";
import { useUi } from "../stores/ui";
import type { BlockParams } from "./geometry";
import GlassBlock from "./GlassBlock.vue";
import { enabledRatio, overviewHero, runHero, scheduleHero, targetHero } from "./heroes";

const EASE = [0.5, 0, 0.1, 1] as const;
const reduce = typeof matchMedia !== "undefined" && matchMedia("(prefers-reduced-motion: reduce)").matches;

const { t } = useI18n();
const ui = useUi();
const ov = useOverview();
const run = useRun();
const sched = useSchedule();
const globalLight = useGlobalLight();
const selected = useSelectedTarget();

const root = ref<HTMLElement | null>(null);
const heroEl = ref<HTMLElement | null>(null);

const scene = computed<{ params: BlockParams; light: Light; label: string }>(() => {
  const ratio = enabledRatio(ov.targets);
  switch (ui.screen) {
    case "targets": {
      const x = selected.value;
      if (!x) return { params: overviewHero(0, 1, "off", false), light: "off", label: "" };
      const failed = run.failedPath && run.failedPath.startsWith(x.path);
      const light: Light = x.enabled && x.allowed ? (failed ? "red" : "green") : "off";
      const label = x.logicalBytes > 0
        ? t("targets.heroAria", { name: targetName(x), disk: bytes(x.onDiskBytes), logical: bytes(x.logicalBytes) })
        : t("targets.heroAriaNone", { name: targetName(x) });
      return { params: targetHero(x, ov.measuring), light, label };
    }
    case "runs": {
      const pb = run.preview?.bytes ?? 0;
      const r = run.result && pb > 0 ? Math.max(0, run.freed) / pb : 0.45;
      const params = runHero({ phase: run.phase, fraction: run.fraction, ratio: r, stopped: run.stopped, enabledRatio: ratio });
      const light: Light = { previewing: "amber", preview: "off", running: "amber", done: "green", failed: "red", idle: ratio == null ? "off" : "green" }[run.phase] as Light;
      return { params, light, label: t("runs.heroAria") };
    }
    case "schedule": {
      const on = sched.installed && ratio != null;
      return { params: scheduleHero(ratio, sched.installed), light: on ? "green" : "off", label: t("schedule.heroAria", { state: t(on ? "schedule.heroOn" : "schedule.heroOff") }) };
    }
    default: {
      const d = ov.disk;
      const params = overviewHero(d?.usedBytes ?? 0, d?.totalBytes ?? 1, globalLight.value, ov.measuring);
      const light: Light = ov.measuring ? "amber" : globalLight.value;
      return { params, light, label: d ? t("overview.heroAria", { used: bytes(d.usedBytes), total: bytes(d.totalBytes) }) : "" };
    }
  }
});

let placed = false;
let watched: Element | null = null;
const ro = typeof ResizeObserver === "undefined" ? null : new ResizeObserver(() => place(true));

function slot(): HTMLElement | null {
  const el = document.querySelector<HTMLElement>("#screen [data-hero]");
  return el && el.getBoundingClientRect().width > 0 ? el : null;
}

function place(instant = false) {
  const host = root.value;
  const hero = heroEl.value;
  if (!host || !hero) return;
  // the floor that carries the light: the whole main column, so the pool runs under the dock too
  const floor = host.closest<HTMLElement>("[data-floor]") ?? host.parentElement;
  floor?.style.setProperty("--hl", LIGHT_VAR[scene.value.light]);
  const s = slot();
  if (s !== watched) {
    if (watched) ro?.unobserve(watched);
    if (s) ro?.observe(s);
    watched = s;
  }
  if (!s) {
    animate(hero, { opacity: 0 }, { duration: instant || reduce ? 0 : 0.2 });
    return;
  }
  const h = host.getBoundingClientRect();
  const r = s.getBoundingClientRect();
  const fast = instant || reduce || !placed;
  animate(hero, { x: r.left - h.left, y: r.top - h.top, width: `${r.width}px`, height: `${r.height}px`, opacity: 1 }, { duration: fast ? 0 : 0.75, ease: EASE });
  floor?.style.setProperty("--ax", `${(r.left - h.left + r.width / 2).toFixed(0)}px`);
  floor?.style.setProperty("--ay", `${(r.top - h.top + r.height * 0.7).toFixed(0)}px`);
  placed = true;
}

const schedule = (instant = false) => nextTick(() => requestAnimationFrame(() => place(instant)));
const layoutKey = computed(() => [ui.route.name, ui.route.targetId, ov.ready, ov.measuring, run.phase, ov.enabled.length, sched.installed].join("|"));
watch(layoutKey, () => schedule());
watch(() => scene.value.light, () => schedule(true));

const onScroll = () => place(true);
const onResize = () => place(true);
onMounted(() => {
  schedule(true);
  if (root.value) ro?.observe(root.value);
  document.addEventListener("scroll", onScroll, { capture: true, passive: true });
  window.addEventListener("resize", onResize);
  void document.fonts?.ready.then(() => place(true));
});
onBeforeUnmount(() => {
  ro?.disconnect();
  document.removeEventListener("scroll", onScroll, { capture: true });
  window.removeEventListener("resize", onResize);
});
</script>

<template>
  <div ref="root" class="pointer-events-none absolute inset-0 z-[5] overflow-hidden" aria-hidden="false">
    <div ref="heroEl" class="hero-light absolute left-0 top-0 opacity-0" :style="{ '--hl': LIGHT_VAR[scene.light] }">
      <GlassBlock :params="scene.params" :label="scene.label" />
    </div>
  </div>
</template>
