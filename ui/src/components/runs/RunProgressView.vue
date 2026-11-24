<script setup lang="ts">
// The run itself: files done of the total, the meter, and each preview root as a step that turns from
// waiting to compressing to done as the engine reaches it.
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { basename, bytes, int, tilde } from "../../lib/format";
import { divided, screenLead } from "../../lib/styles";
import { useRun } from "../../stores/run";
import InfoRow from "../ui/InfoRow.vue";
import Numeral from "../ui/Numeral.vue";
import SegmentMeter from "../ui/SegmentMeter.vue";

const { t } = useI18n();
const run = useRun();

const status = computed(() => {
  const p = run.progress?.phase;
  if (run.stopRequested) return t("runs.running.stopping");
  if (!p || p === "starting") return t("runs.running.starting");
  if (p === "verifying") return t("runs.running.verifying");
  return "";
});

const roots = computed(() => {
  const all = run.preview?.roots ?? [];
  const verifying = run.progress?.phase === "verifying";
  const n = run.reached.length;
  return all.map((r) => {
    const i = run.reached.indexOf(r.path);
    const state = verifying || (i >= 0 && i < n - 1) ? "done" : i === n - 1 && i >= 0 ? "active" : "waiting";
    const file = run.progress?.currentPath ? basename(run.progress.currentPath) : "";
    return {
      path: r.path,
      bytes: r.bytes,
      state,
      note: state === "done" ? t("runs.running.rootDone") : state === "active" ? t("runs.running.rootActive", { file }) : t("runs.running.rootWaiting"),
    };
  });
});
</script>

<template>
  <p><Numeral :value="int(run.files)" :unit="t('runs.running.unit', { total: int(run.totalFiles) })" /></p>
  <p :class="[screenLead, 'mt-3 max-w-[52ch]']">{{ t("runs.running.lead") }}</p>
  <div class="mt-5">
    <SegmentMeter continuous :value="run.fraction" :label="t('runs.running.meter')" :value-text="`${int(run.files)} / ${int(run.totalFiles)}`" tone="bg-magsafe-amber" :min-cell="10" :max-cells="40" />
    <p class="mt-2 h-5 text-micro text-anodized-muted">{{ status }}</p>
  </div>
  <ol :class="['mt-2', divided]">
    <li v-for="r in roots" :key="r.path">
      <InfoRow
        :icon="r.state === 'done' ? 'check' : 'folder'"
        :tone="r.state === 'done' ? 'text-magsafe-green-deep' : r.state === 'active' ? 'text-magsafe-amber-deep' : 'text-anodized-muted'"
        :busy="r.state === 'active'"
        :label="tilde(r.path)"
        :note="r.note"
        :value="bytes(r.bytes)"
        :value-tone="r.state === 'waiting' ? 'text-anodized-muted' : ''"
      />
    </li>
  </ol>
</template>
