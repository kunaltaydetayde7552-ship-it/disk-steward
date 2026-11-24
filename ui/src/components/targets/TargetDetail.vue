<script setup lang="ts">
// The selected folder: a fixed head (name, state, the block beside the number that matters), a body that
// scrolls on its own and a fixed foot with the controls, so at 960 x 640 the switch and Finder stay in view.
import { computed, nextTick, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { api, type Target } from "../../api/client";
import { useScrollEdges } from "../../composables/useScrollEdges";
import { errorText } from "../../lib/errors";
import { bytes, int, percent, tilde } from "../../lib/format";
import { btnGhost } from "../../lib/styles";
import { isMeasured, isPseudo, kindLabel, sharedBytes, targetName } from "../../lib/targets";
import { useOverview } from "../../stores/overview";
import { useRun } from "../../stores/run";
import { useUi } from "../../stores/ui";
import AppIcon from "../ui/AppIcon.vue";
import Numeral from "../ui/Numeral.vue";
import SegmentMeter from "../ui/SegmentMeter.vue";
import SizeGlyph from "../ui/SizeGlyph.vue";
import ToggleSwitch from "../ui/ToggleSwitch.vue";
import { numeralCopy, riskCopy, sharedNote } from "./copy";

const props = defineProps<{ target: Target }>();
const { t } = useI18n();
const ui = useUi();
const ov = useOverview();
const run = useRun();

const x = computed(() => props.target);
const name = computed(() => targetName(x.value));
const blocked = computed(() => !x.value.allowed);
const measuring = computed(() => ov.measuring && !isPseudo(x.value));
const known = computed(() => isMeasured(x.value) && !ov.neverMeasured);
const num = computed(() => numeralCopy(x.value, ov.neverMeasured));
const risk = computed(() => riskCopy(x.value));
const shared = computed(() => sharedBytes(x.value));
const coreFill = computed(() =>
  x.value.enabled ? "color-mix(in oklab, var(--color-magsafe-green) 70%, white)" : blocked.value ? "color-mix(in oklab, var(--color-quartz) 45%, white)" : "var(--core-unlit-left)",
);
interface SizeRow {
  kind: "logical" | "disk" | "private" | "shared";
  label: string;
  note: string;
  value: string;
}
const sizes = computed<SizeRow[]>(() => {
  const v = x.value;
  if (isPseudo(v)) return [];
  const rows: SizeRow[] = [
    { kind: "logical", label: t("targets.sizes.logical"), note: t("targets.sizes.logicalNote"), value: known.value ? bytes(v.logicalBytes) : t("targets.sizes.noData") },
    { kind: "disk", label: t("targets.sizes.disk"), note: t("targets.sizes.diskNote"), value: known.value ? bytes(v.onDiskBytes) : t("targets.sizes.noData") },
    { kind: "private", label: t("targets.sizes.private"), note: t("targets.sizes.privateNote"), value: v.privateBytes == null ? t("targets.sizes.noData") : bytes(v.privateBytes) },
  ];
  if (known.value && shared.value > v.onDiskBytes * 0.05) rows.push({ kind: "shared", label: t("targets.sizes.shared"), note: sharedNote(v), value: bytes(shared.value) });
  return rows;
});
const filesShare = computed(() => (x.value.totalFiles > 0 ? x.value.compressedFiles / x.value.totalFiles : 0));
const copiedPattern = computed(() => ui.copied === x.value.path);

async function reveal() {
  try {
    await api.revealInFinder(x.value.path);
  } catch (e) {
    ui.toast("error", errorText(e));
  }
}

// the body scrolls on its own; a hairline appears under the head once it has moved, and over the foot
// while there is more below
const body = ref<HTMLElement | null>(null);
const { scrolled, more, measure } = useScrollEdges(body);
watch(() => x.value.id, async () => {
  body.value?.scrollTo({ top: 0 });
  await nextTick();
  measure();
});

// a press on a locked switch brings the reason into view and nudges it
const riskEl = ref<HTMLElement | null>(null);
const nudgeKey = computed(() => (ui.nudge?.id === x.value.id ? ui.nudge.n : 0));
watch(nudgeKey, async (n) => {
  if (!n) return;
  await nextTick();
  riskEl.value?.scrollIntoView({ block: "nearest", behavior: matchMedia("(prefers-reduced-motion: reduce)").matches ? "auto" : "smooth" });
  // the highlight is a moment, not a state
  setTimeout(() => {
    if (ui.nudge?.n === n) ui.nudge = null;
  }, 900);
}, { immediate: true });
</script>

<template>
  <section :aria-label="t('targets.detail', { name })" class="@container/panel flex min-h-0 flex-col overflow-hidden rounded-panel bg-frost shadow-float ring-1 ring-aluminum-deep @min-[44rem]:max-h-full">
    <header :class="['shrink-0 px-6 pt-5 transition-[border-color] duration-200 max-md:px-4 max-md:pt-4', scrolled ? 'border-b border-aluminum-deep' : 'border-b border-transparent']">
      <div class="flex items-start justify-between gap-4">
        <div class="min-w-0">
          <h2 class="text-title font-medium text-pretty">{{ name }}</h2>
          <p class="text-micro text-anodized-muted [overflow-wrap:anywhere]">{{ t("targets.pathKind", { path: tilde(x.path), kind: kindLabel(x) }) }}</p>
        </div>
        <span class="shrink-0 pt-1 text-micro">
          <span v-if="blocked" class="inline-flex items-center gap-2 text-quartz-deep [&_svg]:size-4"><AppIcon name="lock" />{{ t("targets.status.blocked") }}</span>
          <span v-else-if="x.enabled" class="inline-flex items-center gap-2 text-magsafe-green-deep"><span class="size-2 rounded-full bg-magsafe-green shadow-[0_0_8px_var(--color-magsafe-green)]" />{{ t("targets.status.on") }}</span>
          <span v-else class="inline-flex items-center gap-2 text-anodized-muted"><span class="size-2 rounded-full bg-aluminum-deep" />{{ t("targets.status.off") }}</span>
        </span>
      </div>
      <div class="mt-2 grid grid-cols-[104px_minmax(0,1fr)] items-center gap-5 pb-4 @max-[22rem]/panel:grid-cols-[80px_minmax(0,1fr)] @max-[22rem]/panel:gap-3 [@media(max-height:700px)]:grid-cols-[72px_minmax(0,1fr)] [@media(max-height:700px)]:pb-3">
        <div data-hero class="aspect-[520/600] w-full" />
        <div class="min-w-0" aria-live="polite">
          <span v-if="measuring" class="skel inline-block h-10 w-32" />
          <Numeral v-else :value="num.value" :unit="num.unit" size="display" />
          <p class="mt-2 text-micro text-pretty text-anodized-muted">{{ num.sub }}</p>
        </div>
      </div>
    </header>

    <div
      ref="body"
      :class="['min-h-0 flex-1 overflow-y-auto overscroll-contain px-6 pb-4 max-md:px-4', more ? '[mask-image:linear-gradient(to_bottom,black_calc(100%-32px),transparent)]' : '']"
      @scroll.passive="measure"
    >
      <div>
        <div v-if="risk" ref="riskEl" :key="nudgeKey" :class="['mt-1 rounded-key bg-quartz/15 p-4 ring-1 ring-quartz/50', nudgeKey ? 'nudge ring-2 ring-quartz' : '']">
          <div class="flex gap-3">
            <span class="grid size-8 shrink-0 place-items-center rounded-full bg-frost text-quartz-deep shadow-lift [&_svg]:size-4"><AppIcon name="lock" /></span>
            <div class="min-w-0">
              <p class="font-medium">{{ risk.title }}</p>
              <p class="mt-1 text-anodized-muted">{{ risk.body }}</p>
              <p class="mt-2 text-quartz-deep">{{ risk.how }}</p>
            </div>
          </div>
        </div>

        <dl v-if="sizes.length" :aria-label="t('targets.sizes.label')" class="mt-4 divide-y divide-aluminum-deep border-y border-aluminum-deep">
          <div v-for="r in sizes" :key="r.kind" class="flex items-center gap-3 py-2">
            <span class="text-anodized-muted"><SizeGlyph :kind="r.kind" :fill="coreFill" /></span>
            <dt class="min-w-0 flex-1">
              <span>{{ r.label }}</span>
              <span class="ml-2 text-micro text-anodized-muted @max-[22rem]/panel:ml-0 @max-[22rem]/panel:block">{{ r.note }}</span>
            </dt>
            <dd class="text-lead font-light tabular-nums">
              <span v-if="measuring" class="skel inline-block h-5 w-16 align-middle" />
              <template v-else>{{ r.value }}</template>
            </dd>
          </div>
        </dl>

        <div v-if="!blocked && known && x.totalFiles > 0" class="mt-4">
          <div class="mb-2 flex items-baseline justify-between gap-3 text-micro text-anodized-muted">
            <span>{{ t("targets.files", { done: int(x.compressedFiles), total: int(x.totalFiles) }) }}</span>
            <span class="tabular-nums">{{ percent(filesShare) }}</span>
          </div>
          <SegmentMeter :value="filesShare" :label="t('targets.filesAria')" :tone="x.enabled ? 'bg-magsafe-green' : 'bg-anodized-muted'" :min-cell="10" />
        </div>
      </div>
    </div>

    <footer :class="['flex shrink-0 flex-wrap items-center justify-between gap-3 border-t px-6 py-4 transition-[border-color] duration-200 max-md:px-4', more ? 'border-aluminum-deep' : 'border-transparent']">
      <template v-if="blocked">
        <span class="text-micro text-anodized-muted">{{ t("targets.refused") }}</span>
        <button v-if="isPseudo(x)" type="button" :class="[btnGhost, '[&_svg]:size-4']" @click="ui.copy(x.path)">
          <AppIcon :name="copiedPattern ? 'check' : 'copy'" />{{ copiedPattern ? t("targets.copied") : t("targets.copyPattern") }}
        </button>
        <button v-else type="button" :class="[btnGhost, '[&_svg]:size-4']" :title="t('targets.showInFinder')" @click="reveal"><AppIcon name="finder" /><span class="@max-[26rem]/panel:sr-only">{{ t("targets.showInFinder") }}</span></button>
      </template>
      <template v-else>
        <div class="flex items-center gap-3">
          <ToggleSwitch
            :checked="x.enabled"
            :pending="ov.pending.includes(x.id)"
            :label="t('targets.toggle', { name })"
            :locked-reason="run.busy ? t('targets.runningLock') : undefined"
            @change="(v) => ov.setEnabled(x, v)"
          />
          <span>
            <span class="block font-medium">{{ t("targets.compressThis") }}</span>
            <span class="block text-micro text-anodized-muted">{{ run.busy ? t("targets.runningLock") : x.enabled ? t("targets.inPreview") : t("targets.outOfPreview") }}</span>
          </span>
        </div>
        <button type="button" :class="[btnGhost, '[&_svg]:size-4']" :title="t('targets.showInFinder')" @click="reveal"><AppIcon name="finder" /><span class="@max-[26rem]/panel:sr-only">{{ t("targets.showInFinder") }}</span></button>
      </template>
    </footer>
  </section>
</template>
