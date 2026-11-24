<script setup lang="ts">
// The big number of a screen with its unit beside it, extra light, proportional figures.
const SIZE = { hero: "text-hero", numeral: "text-numeral", display: "text-display", heading: "text-heading" } as const;
withDefaults(defineProps<{ value: string; unit?: string; size?: keyof typeof SIZE }>(), { unit: "", size: "numeral" });
</script>

<template>
  <span class="flex flex-wrap items-baseline gap-x-3">
    <span :class="[SIZE[size], 'font-extralight tracking-[-0.03em] [font-variant-numeric:proportional-nums]']"><slot>{{ value }}</slot></span>
    <span v-if="unit" class="text-title font-light text-anodized-muted">{{ unit }}</span>
  </span>
</template>
