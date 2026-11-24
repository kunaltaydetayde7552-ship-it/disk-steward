<script setup lang="ts">
// A row: an icon in a raised circle, a label, one plain line, and a large value on the right.
import IconDisc from "./IconDisc.vue";
import type { IconName } from "./icons";

withDefaults(defineProps<{ icon?: IconName; tone?: string; busy?: boolean; label: string; note?: string; value?: string; valueTone?: string }>(), {
  icon: undefined, tone: "text-anodized", busy: false, note: "", value: "", valueTone: "",
});
</script>

<template>
  <div class="grid grid-cols-[40px_1fr] gap-4 py-4 [@media(max-height:700px)]:py-3">
    <slot name="icon"><IconDisc :icon="icon" :tone="tone" :busy="busy" /></slot>
    <div class="min-w-0">
      <div class="flex items-baseline justify-between gap-4">
        <span class="min-w-0 text-lead text-anodized-muted [overflow-wrap:anywhere]">{{ label }}</span>
        <span :class="['shrink-0 text-right text-value font-light tabular-nums', valueTone]"><slot name="value">{{ value }}</slot></span>
      </div>
      <p v-if="note || $slots.note" class="mt-1 max-w-[56ch] text-micro text-anodized-muted"><slot name="note">{{ note }}</slot></p>
      <slot />
    </div>
  </div>
</template>
