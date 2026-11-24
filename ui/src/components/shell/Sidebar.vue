<script setup lang="ts">
// Translucent source list over the window vibrancy: brand cube with the object's light, four sections with
// live badges, and the volume card. Below md it turns into a tab strip (only reachable in a plain browser).
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { isTauri } from "../../api/client";
import { bytes, day, isToday, time } from "../../lib/format";
import { focusRing } from "../../lib/styles";
import { useOverview } from "../../stores/overview";
import { useRun } from "../../stores/run";
import { useGlobalLight } from "../../stores/scene";
import { useSchedule } from "../../stores/schedule";
import { pathOf, SCREENS, useUi, type ScreenId } from "../../stores/ui";
import AppIcon from "../ui/AppIcon.vue";
import type { IconName } from "../ui/icons";
import MiniCube from "../ui/MiniCube.vue";
import SegmentMeter from "../ui/SegmentMeter.vue";

const { t } = useI18n();
const ui = useUi();
const ov = useOverview();
const run = useRun();
const sched = useSchedule();
const light = useGlobalLight();
const web = !isTauri();

const cube = computed(() => (light.value === "off" ? "unlit" : light.value));
const status = computed(() => {
  if (ov.measuring) return t("sidebar.measuring");
  const at = ov.measuredAt;
  if (!at || ov.neverMeasured) return t("sidebar.never");
  return isToday(at) ? t("sidebar.measuredAt", { time: time(at) }) : t("sidebar.measuredOn", { day: day(at), time: time(at) });
});
const savedLine = computed(() => {
  const s = ov.savings;
  if (ov.engineMissing) return t("sidebar.engineMissing");
  if (!s) return "";
  return s.unattributedBytes > 0
    ? t("sidebar.savedOutside", { saved: bytes(s.savedBytes), outside: bytes(s.unattributedBytes) })
    : t("sidebar.saved", { saved: bytes(s.savedBytes) });
});
const usedFraction = computed(() => (ov.disk && ov.disk.totalBytes > 0 ? ov.disk.usedBytes / ov.disk.totalBytes : 0));

interface Badge {
  dot?: "amber" | "red";
  text: string;
  /** hidden first when the sidebar narrows */
  soft?: boolean;
}
const badges = computed<Partial<Record<ScreenId, Badge>>>(() => ({
  runs: run.phase === "running" ? { dot: "amber", text: t("nav.running") } : run.phase === "failed" ? { dot: "red", text: t("nav.failed") } : undefined,
  schedule: ov.ready && !sched.installed ? { text: t("nav.manual"), soft: true } : undefined,
  targets: ov.ready ? { text: t("nav.count", { on: ov.enabled.length, total: ov.targets.length }) } : undefined,
}));
</script>

<template>
  <aside class="relative z-20 flex w-[232px] shrink-0 flex-col border-r border-aluminum-deep/70 bg-frost/70 backdrop-blur-2xl max-lg:w-[208px] max-md:w-full max-md:border-b max-md:border-r-0">
    <!-- the title bar: native traffic lights sit here in Tauri; a quiet stand-in in the browser -->
    <div class="flex h-13 shrink-0 items-center gap-2 px-5 max-md:hidden" data-tauri-drag-region>
      <template v-if="web">
        <span class="size-3 rounded-full bg-aluminum-deep" aria-hidden="true" />
        <span class="size-3 rounded-full bg-aluminum-deep" aria-hidden="true" />
        <span class="size-3 rounded-full bg-aluminum-deep" aria-hidden="true" />
      </template>
    </div>

    <div class="px-3 max-md:pt-3">
      <div class="flex items-center gap-3 rounded-key bg-aluminum/60 px-3 py-3 ring-1 ring-aluminum-deep max-md:bg-transparent max-md:px-1 max-md:py-1 max-md:ring-0" data-tauri-drag-region>
        <MiniCube :light="cube" :size="30" />
        <span class="min-w-0 flex-1">
          <span class="block font-semibold">{{ t("app.name") }}</span>
          <span class="flex items-center gap-2 truncate text-micro text-anodized-muted" aria-live="polite">
            <span v-if="ov.measuring" class="pulse size-1.5 shrink-0 rounded-full bg-magsafe-amber" aria-hidden="true" />{{ status }}
          </span>
        </span>
      </div>
    </div>

    <nav :aria-label="t('nav.label')" class="mt-4 flex flex-col gap-1 px-3 max-md:mt-2 max-md:flex-row max-md:pb-2">
      <a
        v-for="(s, i) in SCREENS"
        :key="s.id"
        :href="`#${pathOf(s.id)}`"
        :aria-current="ui.screen === s.id ? 'page' : undefined"
        :title="t('nav.shortcut', { name: t(`nav.${s.id}`), n: i + 1 })"
        :class="[
          'flex h-10 items-center gap-3 rounded-key px-3 font-medium transition-[background-color,color,box-shadow] duration-150 ease-platen max-md:h-auto max-md:flex-1 max-md:flex-col max-md:gap-1 max-md:px-1 max-md:py-2 max-md:text-micro',
          ui.screen === s.id ? 'bg-frost text-anodized shadow-lift ring-1 ring-aluminum-deep' : 'text-anodized-muted hover:bg-aluminum/80 hover:text-anodized active:bg-aluminum-deep/60',
          focusRing,
        ]"
      >
        <AppIcon :name="s.icon as IconName" />
        <span class="truncate">{{ t(`nav.${s.id}`) }}</span>
        <span
          v-if="badges[s.id]?.dot"
          role="img"
          :aria-label="badges[s.id]!.text"
          :class="[
            'ml-auto size-2 shrink-0 rounded-full max-md:hidden',
            badges[s.id]!.dot === 'red' ? 'bg-signal-red shadow-[0_0_8px_var(--color-signal-red)]' : 'pulse bg-magsafe-amber shadow-[0_0_8px_var(--color-magsafe-amber)]',
          ]"
        />
        <span v-else-if="badges[s.id]" :class="['ml-auto text-micro tabular-nums text-anodized-muted max-md:hidden', badges[s.id]!.soft ? 'max-lg:hidden' : '']">
          {{ badges[s.id]!.text }}
        </span>
      </a>
    </nav>

    <div class="mt-auto p-3 max-md:hidden">
      <div class="px-3 pb-3">
        <div class="flex items-baseline justify-between">
          <span class="text-micro text-anodized-muted">{{ t("sidebar.free") }}</span>
          <span v-if="ov.disk" class="text-lead font-light tabular-nums">{{ bytes(ov.disk.freeBytes) }}</span>
          <span v-else class="skel inline-block h-5 w-16" />
        </div>
        <div class="mt-2"><SegmentMeter :value="usedFraction" :label="t('sidebar.diskInUse')" :min-cell="10" /></div>
        <p class="mt-2 text-micro text-pretty text-anodized-muted">{{ savedLine }}</p>
      </div>
      <a
        :href="`#${pathOf('overview')}`"
        :aria-label="t('sidebar.toOverview')"
        :class="['flex items-center gap-3 rounded-key bg-aluminum/60 p-3 ring-1 ring-aluminum-deep transition-colors duration-150 ease-platen hover:bg-aluminum active:shadow-press', focusRing]"
      >
        <span class="grid size-9 shrink-0 place-items-center rounded-full bg-frost shadow-lift"><MiniCube :light="cube" :size="22" /></span>
        <span class="min-w-0 flex-1">
          <span class="block truncate text-micro font-semibold">{{ t("app.volume") }}</span>
          <span class="block truncate text-micro text-anodized-muted">{{ ov.disk ? t("app.volumeKind", { total: bytes(ov.disk.totalBytes) }) : "" }}</span>
        </span>
      </a>
    </div>
  </aside>
</template>
