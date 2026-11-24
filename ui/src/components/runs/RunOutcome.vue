<script setup lang="ts">
// After the run: what df measured (done) or where it stopped and what the engine said (failed).
// The run log is the one place raw engine text is shown.
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { CANCELLED } from "../../api/types";
import { bytes, bytesParts, duration, int, time } from "../../lib/format";
import { divided, focusRing, screenLead } from "../../lib/styles";
import { useRun } from "../../stores/run";
import AppIcon from "../ui/AppIcon.vue";
import InfoRow from "../ui/InfoRow.vue";
import Numeral from "../ui/Numeral.vue";

const { t } = useI18n();
const run = useRun();

const r = computed(() => run.result);
const freed = computed(() => Math.max(0, run.freed));
const parts = computed(() => bytesParts(freed.value));
const before = computed(() => run.freeBefore);
const lines = computed(() => {
  if (r.value) return r.value.errors.filter((e) => e !== CANCELLED);
  return run.failure ? [`${run.failure.code}: ${run.failure.message}`] : [];
});
const durationMs = computed(() => r.value?.durationMs ?? 0);
const untouched = computed(() => Math.max(0, run.totalFiles - run.files));
</script>

<template>
  <template v-if="run.phase === 'done'">
    <p v-if="freed > 0"><Numeral :value="parts[0]" :unit="t('runs.done.unit', { unit: parts[1] })" /></p>
    <p v-else><Numeral :value="t('runs.done.noGainNumeral')" :unit="t('runs.done.noGainUnit')" size="display" /></p>
    <p :class="[screenLead, 'mt-3 max-w-[52ch]']">
      {{ run.stopped ? t("runs.done.stoppedLead", { files: int(run.filesDone) }) : freed > 0 ? t("runs.done.lead", { estimate: bytes(run.preview?.bytes ?? 0) }) : t("runs.done.noGainLead") }}
    </p>
    <div :class="['mt-6', divided]">
      <InfoRow v-if="before != null && r" icon="disk" :label="t('runs.done.before')" :note="t('runs.done.beforeNote', { time: time(r.startedAt) })" :value="bytes(before)" />
      <InfoRow v-if="before != null" icon="disk" tone="text-magsafe-green-deep" :label="t('runs.done.after')" :note="t('runs.done.afterNote', { duration: duration(durationMs) })" :value="bytes(before + freed)" />
      <InfoRow icon="check" tone="text-magsafe-green-deep" :label="t('runs.done.files')" :note="t('runs.done.filesNote')" :value="int(run.filesDone)" />
      <InfoRow icon="schedule" :label="t('runs.done.duration')" :note="t('runs.done.durationNote')" :value="duration(durationMs)" />
    </div>
    <div v-if="lines.length" class="mt-5 rounded-key bg-magsafe-amber/10 p-4 ring-1 ring-magsafe-amber/40">
      <p class="flex items-center gap-2 font-medium text-magsafe-amber-deep [&_svg]:size-4"><AppIcon name="log" />{{ t("runs.done.warnings") }}</p>
      <ul class="mt-2 space-y-2"><li v-for="(l, i) in lines" :key="i" class="font-log text-micro [overflow-wrap:anywhere]">{{ l }}</li></ul>
    </div>
  </template>

  <template v-else>
    <p><Numeral :value="parts[0]" :unit="t('runs.failed.unit', { unit: parts[1] })" /></p>
    <p :class="[screenLead, 'mt-3 max-w-[52ch]']">{{ run.failedPath ? t("runs.failed.lead") : t("runs.failed.leadOther") }}</p>
    <div id="run-log" tabindex="-1" :class="['mt-5 rounded-key bg-signal-red/8 p-4 ring-1 ring-signal-red/30', focusRing]">
      <p class="flex items-center gap-2 font-medium text-signal-red-deep [&_svg]:size-4"><AppIcon name="alert" />{{ t("runs.failed.log") }}</p>
      <ul class="mt-2 space-y-2"><li v-for="(l, i) in lines" :key="i" class="font-log text-micro text-anodized [overflow-wrap:anywhere]">{{ l }}</li></ul>
    </div>
    <div v-if="run.totalFiles > 0" class="mt-2 divide-y divide-aluminum-deep">
      <InfoRow icon="check" tone="text-magsafe-green-deep" :label="t('runs.failed.swapped')" :note="t('runs.failed.swappedNote')" :value="int(run.files)" />
      <InfoRow icon="folder" tone="text-anodized-muted" :label="t('runs.failed.untouched')" :note="t('runs.failed.untouchedNote')" :value="int(untouched)" />
    </div>
  </template>
</template>
