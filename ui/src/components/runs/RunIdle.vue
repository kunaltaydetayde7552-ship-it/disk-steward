<script setup lang="ts">
// No run in flight: the last result as the number, the history under it.
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { CANCELLED, type RunResult } from "../../api/types";
import { bytes, bytesParts, day, duration, int, time } from "../../lib/format";
import { divided, screenLead } from "../../lib/styles";
import { useRun } from "../../stores/run";
import InfoRow from "../ui/InfoRow.vue";
import Numeral from "../ui/Numeral.vue";

const { t } = useI18n();
const run = useRun();
const last = computed(() => run.lastRun);
const lastParts = computed(() => bytesParts(Math.max(0, last.value?.freedBytes ?? 0)));

function kind(h: RunResult): "ok" | "stopped" | "error" {
  if (h.errors.includes(CANCELLED)) return "stopped";
  return h.errors.some((e) => /^\s*ERROR\b/.test(e)) ? "error" : "ok";
}
const NOTE = { ok: "runs.idle.row", stopped: "runs.idle.rowStopped", error: "runs.idle.rowErrors" } as const;
const ICON = { ok: ["check", "text-magsafe-green-deep"], stopped: ["stop", "text-anodized-muted"], error: ["alert", "text-signal-red-deep"] } as const;
</script>

<template>
  <template v-if="last">
    <p><Numeral :value="lastParts[0]" :unit="t('runs.idle.unit', { unit: lastParts[1] })" /></p>
    <p :class="[screenLead, 'mt-3 max-w-[52ch]']">{{ t("runs.idle.lead", { day: day(last.startedAt), time: time(last.startedAt) }) }}</p>
    <h2 class="mb-1 mt-6 text-micro text-anodized-muted">{{ t("runs.idle.history") }}</h2>
    <ul :class="divided">
      <li v-for="h in run.history.slice(0, 6)" :key="h.startedAt">
        <InfoRow
          :icon="ICON[kind(h)][0]"
          :tone="ICON[kind(h)][1]"
          :label="`${day(h.startedAt)}, ${time(h.startedAt)}`"
          :note="t(NOTE[kind(h)], { files: int(h.preview.files), duration: duration(h.durationMs) })"
          :value="bytes(Math.max(0, h.freedBytes))"
        />
      </li>
    </ul>
  </template>
  <template v-else>
    <p><Numeral :value="t('runs.idle.firstNumeral')" :unit="t('runs.idle.firstUnit')" size="display" /></p>
    <p :class="[screenLead, 'mt-3 max-w-[52ch]']">{{ t("runs.idle.firstLead") }}</p>
  </template>
</template>
