<script setup lang="ts">
// Execuções: preview (read only), the run under the press, and the proof by df. The block on the left is
// the run: it drops from full height to what df measured as the files are swapped.
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { bytes, bytesParts, int, percent, tilde } from "../../lib/format";
import { btnSecondary, divided, screenLead, screenTitle } from "../../lib/styles";
import { useOverview } from "../../stores/overview";
import { useRun } from "../../stores/run";
import { pathOf } from "../../stores/ui";
import RunIdle from "../runs/RunIdle.vue";
import RunOutcome from "../runs/RunOutcome.vue";
import RunProgressView from "../runs/RunProgressView.vue";
import AppIcon from "../ui/AppIcon.vue";
import IconDisc from "../ui/IconDisc.vue";
import InfoRow from "../ui/InfoRow.vue";
import Numeral from "../ui/Numeral.vue";
import SegmentMeter from "../ui/SegmentMeter.vue";
import Spinner from "../ui/Spinner.vue";

const { t } = useI18n();
const ov = useOverview();
const run = useRun();

const empty = computed(() => !ov.enabled.length && (run.phase === "idle" || run.phase === "previewing" || run.phase === "preview"));
const p = computed(() => run.preview);
const parts = computed(() => bytesParts(p.value?.bytes ?? 0));
const share = (b: number) => (p.value && p.value.bytes > 0 ? b / p.value.bytes : 0);
</script>

<template>
  <header>
    <h1 :class="screenTitle" tabindex="-1">{{ t("runs.title") }}</h1>
    <p :class="screenLead">{{ t("runs.lead") }}</p>
  </header>
  <div class="mt-4 grid grid-cols-[4fr_6fr] items-start gap-8 @max-[44rem]:grid-cols-1 @max-[44rem]:gap-2">
    <div class="flex justify-center pt-2 @min-[44rem]:sticky @min-[44rem]:top-0">
      <div data-hero class="aspect-[520/600] w-full max-w-[380px] max-h-[calc(100dvh-280px)] @max-[44rem]:max-w-[240px]" />
    </div>

    <section aria-live="polite" class="min-w-0 max-w-[540px] pt-4" :aria-busy="run.phase === 'previewing'">
      <div v-if="empty" class="flex flex-col items-start gap-4 rounded-panel bg-frost/60 p-6 ring-1 ring-aluminum-deep">
        <IconDisc icon="targets" tone="text-anodized-muted" />
        <div>
          <h2 class="text-title font-medium">{{ t("runs.empty.title") }}</h2>
          <p class="mt-1 max-w-[48ch] text-anodized-muted">{{ t("runs.empty.body") }}</p>
        </div>
        <a :href="`#${pathOf('targets')}`" :class="[btnSecondary, '[&_svg]:size-4']"><AppIcon name="targets" />{{ t("runs.empty.open") }}</a>
      </div>

      <template v-else-if="run.phase === 'previewing'">
        <p><Numeral value="" :unit="t('runs.previewing.unit')"><span class="skel inline-block h-14 w-36 align-middle" /></Numeral></p>
        <p class="mt-3 flex items-center gap-2 text-lead text-anodized-muted"><Spinner />{{ t("runs.previewing.lead", { n: ov.enabled.length }) }}</p>
        <div :class="['mt-6', divided]">
          <div v-for="x in ov.enabled" :key="x.id" class="grid grid-cols-[40px_1fr] gap-4 py-4">
            <IconDisc icon="folder" tone="text-anodized-muted" />
            <div><span class="skel block h-5 w-48" /><span class="skel mt-2 block h-3 w-32" /></div>
          </div>
        </div>
      </template>

      <template v-else-if="run.phase === 'preview' && p">
        <div v-if="p.files === 0" class="flex flex-col items-start gap-4 rounded-panel bg-frost/60 p-6 ring-1 ring-aluminum-deep">
          <IconDisc icon="check" tone="text-magsafe-green-deep" />
          <div>
            <h2 class="text-title font-medium">{{ t("runs.preview.nothingTitle") }}</h2>
            <p class="mt-1 max-w-[48ch] text-anodized-muted">{{ t("runs.preview.nothingBody") }}</p>
          </div>
        </div>
        <template v-else>
          <p><Numeral :value="parts[0]" :unit="t('runs.preview.unit', { unit: parts[1] })" /></p>
          <p :class="[screenLead, 'mt-3 max-w-[52ch]']">{{ t("runs.preview.lead", { files: int(p.files), roots: p.roots.length }) }}</p>
          <ul :class="['mt-6', divided]">
            <li v-for="r in p.roots" :key="r.path">
              <InfoRow icon="folder" :label="tilde(r.path)" :note="t('runs.preview.rootNote', { pct: percent(share(r.bytes)) })" :value="bytes(r.bytes)">
                <div class="mt-2"><SegmentMeter :value="share(r.bytes)" :label="t('runs.preview.rootNote', { pct: percent(share(r.bytes)) })" tone="bg-anodized-muted" height="h-1" :min-cell="8" /></div>
              </InfoRow>
            </li>
            <li>
              <InfoRow icon="file" tone="text-anodized-muted" :label="t('runs.preview.skipped')" :note="t('runs.preview.skippedNote')" :value="t('runs.preview.skippedValue', { n: int(p.skipped) }, p.skipped)" />
            </li>
          </ul>
        </template>
      </template>

      <RunProgressView v-else-if="run.phase === 'running'" />
      <RunOutcome v-else-if="run.phase === 'done' || run.phase === 'failed'" />
      <RunIdle v-else />
    </section>
  </div>
</template>
