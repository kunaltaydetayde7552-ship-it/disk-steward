<script setup lang="ts">
// Visão geral: the data volume as a glass block (light = data in use, clear glass = free space) and the four
// numbers that explain it. The free space is the thesis, so it is the largest thing on the screen.
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { age, bytes, bytesParts, int, percent, time } from "../../lib/format";
import { btnGhostSmall, divided, focusRing } from "../../lib/styles";
import { isMeasured, isPseudo, savedRatio, targetLight, targetShort } from "../../lib/targets";
import { useOverview } from "../../stores/overview";
import { useRun } from "../../stores/run";
import { pathOf } from "../../stores/ui";
import AppIcon from "../ui/AppIcon.vue";
import IconDisc from "../ui/IconDisc.vue";
import InfoRow from "../ui/InfoRow.vue";
import MiniCube from "../ui/MiniCube.vue";
import Numeral from "../ui/Numeral.vue";
import SavingsSplit from "../ui/SavingsSplit.vue";
import SegmentMeter from "../ui/SegmentMeter.vue";
import SizeGlyph from "../ui/SizeGlyph.vue";

const { t, locale } = useI18n();
const ov = useOverview();
const run = useRun();

const disk = computed(() => ov.disk!);
const free = computed(() => bytesParts(disk.value.freeBytes));
const used = computed(() => (disk.value.totalBytes > 0 ? disk.value.usedBytes / disk.value.totalBytes : 0));
const cachedState = computed(() => ov.data?.measurement.state ?? "never");

const measureLine = computed(() => {
  if (ov.measuring) return t("overview.measuring");
  if (cachedState.value === "never") return t("overview.never");
  const at = ov.measuredAt ?? disk.value.measuredAt;
  return cachedState.value === "stale" ? t("overview.stale", { age: age(at) }) : t("overview.measuredAt", { time: time(at) });
});

const savings = computed(() => ov.savings);
const docker = computed(() => {
  const d = ov.docker;
  if (d?.available) return { value: bytes(d.imagesBytes + d.buildCacheBytes + d.volumesBytes), note: t("overview.dockerNote", { reclaimable: bytes(d.reclaimableBytes) }), off: false };
  // a sizeless overview never asked Docker: "not measured" is not "stopped"
  if (cachedState.value === "never") return { value: t("overview.dockerNotMeasured"), note: t("overview.dockerNotMeasuredNote"), off: true };
  return { value: t("overview.dockerOff"), note: t("overview.dockerOffNote"), off: true };
});

const list = (names: string[]) => new Intl.ListFormat(locale.value, { type: "conjunction" }).format(names);
const blockedNote = computed(() => {
  const b = ov.blocked;
  if (!b.length) return t("overview.blockedNone");
  const shared = b.filter((x) => !isPseudo(x)).map(targetShort);
  const rewritten = b.filter(isPseudo).map(targetShort);
  const parts = [
    shared.length ? t("overview.blockedClones", { names: list(shared) }, shared.length) : "",
    rewritten.length ? t("overview.blockedRewritten", { names: list(rewritten) }) : "",
  ].filter(Boolean);
  return `${parts.join("; ")}.`;
});

const chips = computed(() =>
  ov.enabled.map((x) => ({
    id: x.id,
    name: targetShort(x),
    light: targetLight(x, run.failedPath),
    saved: x.compressedFiles > 0 ? `−${percent(savedRatio(x))}` : null,
    note: !isMeasured(x) || ov.neverMeasured ? t("overview.chipNotMeasured") : t("overview.chipNotYet"),
  })),
);
</script>

<template>
  <div class="grid min-h-[calc(100dvh-176px)] grid-cols-[minmax(0,4fr)_minmax(0,6fr)] items-center gap-6 @max-[44rem]:min-h-0 @max-[44rem]:grid-cols-1 @max-[44rem]:gap-4">
    <div class="flex flex-col items-center justify-center">
      <div data-hero class="aspect-[520/600] w-full max-w-[420px] max-h-[calc(100dvh-300px)] @max-[44rem]:max-w-[220px]" />
      <p :class="['-mt-6 flex flex-wrap justify-center gap-x-4 gap-y-1 text-micro text-anodized-muted @max-[44rem]:mt-0', ov.measuring ? 'invisible' : '']">
        <span class="inline-flex items-center gap-2"><SizeGlyph kind="private" fill="color-mix(in oklab, var(--hl) 70%, white)" />{{ t("overview.legendLight") }}</span>
        <span class="inline-flex items-center gap-2"><SizeGlyph kind="disk" />{{ t("overview.legendGlass") }}</span>
      </p>
    </div>

    <section class="max-w-[560px] @max-[44rem]:order-first @max-[44rem]:max-w-none">
      <h1 class="sr-only" tabindex="-1">{{ t("overview.title") }}</h1>
      <p :aria-label="t('overview.freeAria', { bytes: bytes(disk.freeBytes) })">
        <Numeral :value="free[0]" :unit="t('overview.freeUnit', { unit: free[1] })" size="hero" />
      </p>
      <p class="mt-3 flex flex-wrap items-center gap-x-2 text-lead text-anodized-muted">
        <span>{{ t("overview.lead", { total: bytes(disk.totalBytes) }) }} {{ measureLine }}</span>
        <button v-if="cachedState === 'stale' && !ov.measuring" type="button" :class="[btnGhostSmall, '-my-1 [&_svg]:size-4']" @click="ov.refresh()">
          <AppIcon name="refresh" />{{ t("overview.remeasure") }}
        </button>
      </p>

      <div v-if="ov.neverMeasured || (ov.measuring && cachedState === 'never')" class="mt-5 flex items-start gap-4 rounded-panel bg-frost/70 p-5 shadow-lift ring-1 ring-aluminum-deep">
        <IconDisc icon="refresh" tone="text-magsafe-amber-deep" :busy="ov.measuring" />
        <div class="min-w-0">
          <h2 class="text-title font-medium">{{ t("overview.neverTitle") }}</h2>
          <p class="mt-1 max-w-[52ch] text-anodized-muted">{{ t("overview.neverBody") }}</p>
        </div>
      </div>

      <div :class="['mt-5 [@media(max-height:700px)]:mt-4', divided]">
        <InfoRow icon="disk" :label="t('overview.used')" :note="t('overview.usedNote')" :value="bytes(disk.usedBytes)">
          <div class="mt-3"><SegmentMeter :value="used" height="h-2" :label="t('overview.usedMeter', { pct: percent(used) })" /></div>
        </InfoRow>

        <InfoRow icon="compress" tone="text-magsafe-green-deep" :label="t('overview.saved')">
          <template #value>
            <span v-if="ov.engineMissing">{{ t("overview.savedNoData") }}</span>
            <span v-else-if="ov.measuring" class="skel inline-block h-6 w-24 align-middle" />
            <span v-else>{{ bytes(savings?.savedBytes ?? 0) }}</span>
          </template>
          <template #note>
            <template v-if="ov.engineMissing">{{ t("overview.savedEngineMissing") }}</template>
            <template v-else-if="ov.measuring">{{ t("overview.savedMeasuring") }}</template>
            <template v-else-if="!savings || savings.files === 0">{{ t("overview.savedNothing") }}</template>
            <template v-else>{{ t("overview.savedNote", { files: int(savings.files), original: bytes(savings.originalBytes), onDisk: bytes(savings.onDiskBytes) }) }}</template>
          </template>
          <template v-if="savings && savings.unattributedBytes > 0 && !ov.engineMissing && !ov.measuring">
            <SavingsSplit :saved="savings.savedBytes" :outside="savings.unattributedBytes" />
          </template>
        </InfoRow>

        <InfoRow icon="docker" :tone="docker.off ? 'text-anodized-muted' : 'text-anodized'" :label="t('overview.docker')" :note="docker.note" :value="docker.value" :value-tone="docker.off ? 'text-anodized-muted' : ''" />

        <InfoRow icon="lock" tone="text-quartz-deep" :label="t('overview.blocked')" :note="blockedNote" :value="t('overview.blockedValue', { n: ov.blocked.length }, ov.blocked.length)" />
      </div>

      <div class="mt-5 [@media(max-height:700px)]:mt-3">
        <template v-if="chips.length">
          <p class="mb-2 text-micro text-anodized-muted">{{ t("overview.chipsLead") }}</p>
          <ul class="flex flex-wrap gap-2">
            <li v-for="c in chips" :key="c.id">
              <a
                :href="`#${pathOf('targets', c.id)}`"
                :class="['flex h-9 items-center gap-2 rounded-full bg-frost/70 pl-1 pr-3 ring-1 ring-aluminum-deep transition-[background-color,box-shadow] duration-150 ease-platen hover:bg-frost hover:shadow-lift active:shadow-press', focusRing]"
              >
                <MiniCube :light="c.light" :size="24" />
                <span class="font-medium">{{ c.name }}</span>
                <span v-if="c.saved" class="tabular-nums text-magsafe-green-deep">{{ c.saved }}</span>
                <span v-else class="text-micro text-anodized-muted">{{ c.note }}</span>
              </a>
            </li>
          </ul>
        </template>
        <div v-else class="flex items-center gap-3 rounded-key bg-frost/60 px-4 py-3 ring-1 ring-aluminum-deep">
          <MiniCube light="wire" :size="24" />
          <i18n-t keypath="overview.noneEnabled" scope="global" tag="p" class="text-anodized-muted">
            <template #link>
              <a :href="`#${pathOf('targets')}`" :class="['font-medium text-anodized underline decoration-aluminum-deep underline-offset-4 hover:decoration-anodized', focusRing]">{{ t("overview.noneEnabledLink") }}</a>
            </template>
          </i18n-t>
        </div>
      </div>
    </section>
  </div>
</template>
