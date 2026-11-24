<script setup lang="ts">
// A small iso cube carrying the object's light: sidebar, rows, chips and the menu bar.
import { computed } from "vue";
import type { CubeLight } from "../../lib/targets";

const props = withDefaults(defineProps<{ light: CubeLight | "off"; size?: number }>(), { size: 28 });
const COLOR: Record<string, string> = { green: "var(--color-magsafe-green)", amber: "var(--color-magsafe-amber)", red: "var(--color-signal-red)" };

const faces = computed<[string, string, string]>(() => {
  const c = COLOR[props.light];
  if (c) return [`color-mix(in oklab, ${c} 22%, white)`, `color-mix(in oklab, ${c} 62%, white)`, `color-mix(in oklab, ${c} 86%, black)`];
  if (props.light === "cold") return ["color-mix(in oklab, var(--color-quartz) 22%, white)", "color-mix(in oklab, var(--color-quartz) 48%, white)", "var(--color-quartz)"];
  return ["var(--core-unlit-top)", "var(--core-unlit-left)", "var(--core-unlit-right)"];
});
const glow = computed(() => (COLOR[props.light] ? { filter: `drop-shadow(0 2px 4px color-mix(in oklab, ${COLOR[props.light]} 55%, transparent))` } : undefined));
</script>

<template>
  <svg v-if="light === 'wire'" viewBox="0 0 28 28" :width="size" :height="size" class="shrink-0" aria-hidden="true">
    <g fill="none" stroke="var(--color-anodized-muted)" stroke-width="1" stroke-dasharray="2 2"><path d="M14 3 24.4 9 14 15 3.6 9z" /><path d="M3.6 9v11L14 26V15M24.4 9v11L14 26" /></g>
  </svg>
  <svg v-else viewBox="0 0 28 28" :width="size" :height="size" class="shrink-0 transition-[filter] duration-500 ease-platen" :style="glow" aria-hidden="true">
    <path d="M14 3 24.4 9 14 15 3.6 9z" :style="{ fill: faces[0] }" />
    <path d="M3.6 9 14 15v11L3.6 20z" :style="{ fill: faces[1] }" />
    <path d="M14 15 24.4 9v11L14 26z" :style="{ fill: faces[2] }" />
    <path d="M14 3 24.4 9 14 15 3.6 9z M14 15v11" fill="none" stroke="white" stroke-opacity=".85" stroke-width=".8" stroke-linejoin="round" />
  </svg>
</template>
