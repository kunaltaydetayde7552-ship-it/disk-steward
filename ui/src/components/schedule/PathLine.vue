<script setup lang="ts">
// A log path with a copy button whose label confirms the copy for a moment.
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { tilde } from "../../lib/format";
import { btnGhostSmall } from "../../lib/styles";
import { useUi } from "../../stores/ui";
import AppIcon from "../ui/AppIcon.vue";

const props = defineProps<{ path: string | null }>();
const { t } = useI18n();
const ui = useUi();
const copied = computed(() => props.path != null && ui.copied === props.path);
</script>

<template>
  <div class="mt-3 flex items-center gap-2">
    <template v-if="path">
      <code class="min-w-0 max-w-full truncate rounded-[6px] bg-aluminum/70 px-2 py-1 text-micro text-anodized-muted">{{ tilde(path) }}</code>
      <button type="button" :class="[btnGhostSmall, 'shrink-0 [&_svg]:size-4']" @click="ui.copy(path)">
        <AppIcon :name="copied ? 'check' : 'copy'" />{{ copied ? t("schedule.copied") : t("schedule.copyPath") }}
      </button>
    </template>
    <span v-else class="text-micro text-anodized-muted">{{ t("schedule.noLog") }}</span>
  </div>
</template>
