<script setup lang="ts">
// The menu bar popup (#/tray). In Tauri it fills its own 360 x 480 window over the HUD vibrancy; in a plain
// browser it floats over a desk under a quiet menu bar, so the scene can be judged. Same glass, same light.
import { computed, onBeforeUnmount, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { isTauri } from "../../api/client";
import { hideCurrentWindow, openMainWindow, quitApp } from "../../api/shell";
import GlassBlock from "../../glass/GlassBlock.vue";
import { enabledRatio, overviewHero, runHero } from "../../glass/heroes";
import { bytes, bytesParts, dayFromYmd, day, int, isToday } from "../../lib/format";
import { LIGHT_VAR } from "../../lib/light";
import { btnPrimary, btnSecondary, focusRing } from "../../lib/styles";
import { useOverview } from "../../stores/overview";
import { useRun } from "../../stores/run";
import { useGlobalLight } from "../../stores/scene";
import AppIcon from "../ui/AppIcon.vue";
import MiniCube from "../ui/MiniCube.vue";
import Numeral from "../ui/Numeral.vue";
import SegmentMeter from "../ui/SegmentMeter.vue";
import Spinner from "../ui/Spinner.vue";

const { t } = useI18n();
const ov = useOverview();
const run = useRun();
const light = useGlobalLight();
const native = isTauri();

const disk = computed(() => ov.disk);
const free = computed(() => bytesParts(disk.value?.freeBytes ?? 0));
const used = computed(() => (disk.value && disk.value.totalBytes > 0 ? disk.value.usedBytes / disk.value.totalBytes : 0));
const hl = computed(() => LIGHT_VAR[ov.measuring && run.phase === "idle" ? "amber" : light.value]);

const params = computed(() => {
  if (run.phase === "running" || run.phase === "failed") {
    const pb = run.preview?.bytes ?? 0;
    return runHero({ phase: run.phase, fraction: run.fraction, ratio: run.result && pb > 0 ? Math.max(0, run.freed) / pb : 0.45, stopped: run.stopped, enabledRatio: enabledRatio(ov.targets) });
  }
  return { ...overviewHero(disk.value?.usedBytes ?? 0, disk.value?.totalBytes ?? 1, light.value, ov.measuring), pool: 0.9 };
});

const savedLine = computed(() => {
  const s = ov.savings;
  if (!s || ov.engineMissing) return "";
  return s.unattributedBytes > 0
    ? t("tray.savedOutside", { saved: bytes(s.savedBytes), outside: bytes(s.unattributedBytes) })
    : t("tray.saved", { saved: bytes(s.savedBytes), files: int(s.files) });
});

const last = computed(() => {
  const r = run.lastRun;
  if (r) return { day: isToday(r.startedAt) ? t("tray.today") : day(r.startedAt), freed: bytes(Math.max(0, r.freedBytes)) };
  const d = ov.savings?.lastRunDay;
  return d ? { day: dayFromYmd(d), freed: null } : null;
});

const preview = () => (ov.enabled.length ? openMainWindow("/execucoes", "preview") : openMainWindow("/alvos", null));
const items = computed(() => [
  { id: "open", label: t("tray.open"), keys: "Cmd O", run: () => openMainWindow("/", null) },
  { id: "measure", label: t("tray.remeasure"), keys: "Cmd R", disabled: ov.measuring || run.busy, run: () => ov.refresh() },
]);

function onKey(e: KeyboardEvent) {
  const mod = e.metaKey || e.ctrlKey;
  if (e.key === "Escape") return void hideCurrentWindow();
  if (mod && e.key.toLowerCase() === "o") return e.preventDefault(), void openMainWindow("/", null);
  if (mod && e.key.toLowerCase() === "r") return e.preventDefault(), void ov.refresh();
  if (mod && e.key.toLowerCase() === "q") return e.preventDefault(), void quitApp();
  if (e.key !== "ArrowDown" && e.key !== "ArrowUp") return;
  // the popup reads as a menu: arrows walk its actions
  const all = [...document.querySelectorAll<HTMLElement>("[data-tray-item]:not([disabled])")];
  const at = all.indexOf(document.activeElement as HTMLElement);
  e.preventDefault();
  all[(at + (e.key === "ArrowDown" ? 1 : -1) + all.length) % all.length]?.focus();
}
onMounted(() => window.addEventListener("keydown", onKey));
onBeforeUnmount(() => window.removeEventListener("keydown", onKey));

const item = `flex h-8 w-full items-center justify-between rounded-[6px] px-2 text-left transition-colors duration-100 hover:bg-anodized/8 active:bg-anodized/12 disabled:opacity-45 ${focusRing}`;
</script>

<template>
  <div :class="['relative h-full overflow-hidden', native ? '' : 'desk-surface']" :style="{ '--hl': hl }">
    <!-- browser only: the menu bar the popup hangs from -->
    <div v-if="!native" class="relative flex h-7 items-center gap-5 bg-frost/70 px-4 text-[13px] backdrop-blur-xl max-md:hidden" aria-hidden="true">
      <span class="font-semibold">Finder</span>
      <span class="ml-auto grid h-5 place-items-center rounded-[5px] bg-anodized/12 px-1"><MiniCube :light="light === 'off' ? 'unlit' : light" :size="16" /></span>
    </div>

    <section
      :aria-label="t('tray.label')"
      :class="[
        'tray-material flex flex-col p-3 backdrop-blur-2xl',
        native ? 'h-full rounded-[12px]' : 'absolute right-2 top-9 w-[340px] rounded-[14px] shadow-float max-md:inset-x-2 max-md:top-2 max-md:w-auto',
      ]"
    >
      <div class="flex items-center gap-3 px-1">
        <div class="hero-light aspect-[520/600] w-[76px] shrink-0"><GlassBlock :params="params" :label="t('overview.heroAria', { used: bytes(disk?.usedBytes ?? 0), total: bytes(disk?.totalBytes ?? 0) })" /></div>
        <div class="min-w-0">
          <p v-if="disk"><Numeral :value="free[0]" :unit="t('tray.freeUnit', { unit: free[1] })" size="heading" /></p>
          <span v-else class="skel inline-block h-8 w-28" />
          <p class="mt-1 text-micro text-anodized-muted">{{ disk ? t("tray.of", { total: bytes(disk.totalBytes) }) : "" }}</p>
        </div>
      </div>
      <div class="mt-3 px-1">
        <SegmentMeter :value="used" :label="t('sidebar.diskInUse')" height="h-1" :min-cell="8" />
        <p class="mt-2 min-h-5 text-micro text-pretty text-anodized-muted">{{ savedLine }}</p>
      </div>

      <div class="my-3 h-px bg-aluminum-deep" />

      <div class="px-1" aria-live="polite">
        <template v-if="run.phase === 'running'">
          <div class="flex items-center gap-3">
            <span class="grid size-8 shrink-0 place-items-center rounded-full bg-magsafe-amber/15 text-magsafe-amber-deep"><Spinner /></span>
            <span class="min-w-0 flex-1"><span class="block font-medium">{{ t("tray.running") }}</span><span class="block text-micro tabular-nums text-anodized-muted">{{ t("tray.runningSub", { files: int(run.files), total: int(run.totalFiles) }) }}</span></span>
          </div>
          <div class="mt-2"><SegmentMeter continuous :value="run.fraction" :label="t('dock.progress')" tone="bg-magsafe-amber" height="h-1" :min-cell="8" /></div>
        </template>
        <div v-else-if="run.phase === 'failed'" class="flex items-center gap-3">
          <span class="grid size-8 shrink-0 place-items-center rounded-full bg-signal-red text-frost [&_svg]:size-4"><AppIcon name="alert" /></span>
          <span class="min-w-0"><span class="block font-medium text-signal-red-deep">{{ t("tray.failed") }}</span><span class="block text-micro text-anodized-muted">{{ t("tray.failedSub") }}</span></span>
        </div>
        <div v-else-if="ov.measuring" class="flex items-center gap-3">
          <span class="grid size-8 shrink-0 place-items-center rounded-full bg-magsafe-amber/15 text-magsafe-amber-deep"><Spinner /></span>
          <span class="block font-medium">{{ t("tray.measuring") }}</span>
        </div>
        <div v-else class="flex items-center gap-3">
          <span :class="['grid size-8 shrink-0 place-items-center rounded-full bg-aluminum [&_svg]:size-4', last ? 'text-magsafe-green-deep' : 'text-anodized-muted']"><AppIcon :name="last ? 'check' : 'eye'" /></span>
          <span class="min-w-0">
            <span class="block font-medium">{{ last ? t("tray.last", { day: last.day }) : t("tray.none") }}</span>
            <span class="block text-micro text-anodized-muted">{{ last?.freed ? t("tray.lastSub", { bytes: last.freed }) : t("tray.noneSub") }}</span>
          </span>
        </div>
      </div>

      <div class="mt-3 px-1">
        <button v-if="run.phase === 'running'" type="button" data-tray-item :disabled="run.stopRequested" :class="[btnSecondary, 'w-full [&_svg]:size-4']" @click="run.cancel()">
          <Spinner v-if="run.stopRequested" /><AppIcon v-else name="stop" />{{ run.stopRequested ? t("dock.running.stopping") : t("tray.stop") }}
        </button>
        <button v-else-if="run.phase === 'failed'" type="button" data-tray-item :class="[btnPrimary, 'w-full [&_svg]:size-4']" @click="openMainWindow('/execucoes', null)">
          <AppIcon name="log" />{{ t("tray.seeWhat") }}
        </button>
        <button v-else type="button" data-tray-item :disabled="run.phase === 'previewing'" :class="[btnPrimary, 'w-full [&_svg]:size-4']" @click="preview">
          <AppIcon :name="ov.enabled.length ? 'eye' : 'targets'" />{{ ov.enabled.length ? t("tray.preview") : t("runs.empty.open") }}
        </button>
      </div>

      <div class="my-2 h-px bg-aluminum-deep" />
      <button v-for="i in items" :key="i.id" type="button" data-tray-item :disabled="i.disabled" :class="item" @click="i.run()">
        <span>{{ i.label }}</span><span class="text-micro text-anodized-muted">{{ i.keys }}</span>
      </button>
      <div class="my-1 h-px bg-aluminum-deep" />
      <button type="button" data-tray-item :class="item" @click="quitApp()">
        <span>{{ t("tray.quit") }}</span><span class="text-micro text-anodized-muted">Cmd Q</span>
      </button>
    </section>
  </div>
</template>
