<script setup lang="ts">
// One folder in the list: its cube carries its light and floods the selected row with it.
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import type { Target } from "../../api/types";
import { bytes, percent, tilde } from "../../lib/format";
import { LIGHT_VAR } from "../../lib/light";
import { isMeasured, isPseudo, savedRatio, targetLight, targetName } from "../../lib/targets";
import { useOverview } from "../../stores/overview";
import { useRun } from "../../stores/run";
import { pathOf, useUi } from "../../stores/ui";
import LockSwitch from "../ui/LockSwitch.vue";
import MiniCube from "../ui/MiniCube.vue";
import ToggleSwitch from "../ui/ToggleSwitch.vue";

const props = defineProps<{ target: Target; selected: boolean }>();
const { t } = useI18n();
const ui = useUi();
const ov = useOverview();
const run = useRun();

const x = computed(() => props.target);
const name = computed(() => targetName(x.value));
const light = computed(() => targetLight(x.value, run.failedPath));
const hl = computed(() => (light.value === "green" || light.value === "red" || light.value === "amber" ? LIGHT_VAR[light.value] : LIGHT_VAR.off));
const skeleton = computed(() => ov.measuring && !isPseudo(x.value));
const value = computed(() => (isPseudo(x.value) ? t("targets.notMeasured") : isMeasured(x.value) && !ov.neverMeasured ? bytes(x.value.onDiskBytes) : t("targets.noSize")));
const sub = computed<{ text: string; green?: boolean }>(() => {
  const v = x.value;
  if (isPseudo(v)) return { text: t("targets.sub.rewritten") };
  if (!v.allowed) return { text: t("targets.sub.blocked") };
  if (!v.enabled) return { text: t("targets.sub.off") };
  const r = savedRatio(v);
  return v.compressedFiles > 0 && r > 0 ? { text: t("targets.sub.saved", { pct: percent(r) }), green: true } : { text: t("targets.status.on").toLowerCase() };
});

function locked() {
  ui.goScreen("targets", x.value.id);
  ui.nudgeRisk(x.value.id);
}
</script>

<template>
  <li>
    <div
      :class="[
        'relative grid grid-cols-[28px_minmax(0,1fr)_auto_auto] items-center gap-3 rounded-key px-3 py-2 transition-[background-color,box-shadow] duration-150 ease-platen',
        selected ? 'flood shadow-lift ring-1 ring-aluminum-deep' : 'hover:bg-frost/70',
      ]"
      :style="{ '--hl': hl }"
    >
      <MiniCube :light="light" />
      <a
        :href="`#${pathOf('targets', x.id)}`"
        :aria-current="selected ? 'true' : undefined"
        data-target-link
        class="min-w-0 after:absolute after:inset-0 after:rounded-key focus-visible:outline-none focus-visible:after:outline-2 focus-visible:after:outline-offset-2 focus-visible:after:outline-anodized"
      >
        <span class="block truncate font-medium">{{ name }}</span>
        <span class="block truncate text-micro text-anodized-muted">{{ tilde(x.path) }}</span>
      </a>
      <span class="text-right">
        <span v-if="skeleton" class="skel inline-block h-5 w-16 align-middle" />
        <span v-else class="block text-lead font-light tabular-nums">{{ value }}</span>
        <span :class="['block text-micro', sub.green ? 'text-magsafe-green-deep' : 'text-anodized-muted']">{{ sub.text }}</span>
      </span>
      <ToggleSwitch
        v-if="x.allowed"
        :checked="x.enabled"
        :pending="ov.pending.includes(x.id)"
        :label="t('targets.toggle', { name })"
        :locked-reason="run.busy ? t('targets.runningLock') : undefined"
        @change="(v) => ov.setEnabled(x, v)"
      />
      <LockSwitch v-else :label="t('targets.locked', { name })" :reason="t('targets.refused')" @press="locked" />
    </div>
  </li>
</template>
