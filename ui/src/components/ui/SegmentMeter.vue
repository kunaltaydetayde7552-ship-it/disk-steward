<script setup lang="ts">
// A meter of machined cells. The cell count follows the measured width (cells keep their size, the count
// changes), so a sidebar meter and a full-width run meter read as the same instrument at every window size.
import { computed, onBeforeUnmount, onMounted, ref } from "vue";

const props = withDefaults(
  defineProps<{
    value: number;
    label: string;
    valueText?: string;
    /** fill utility of the lit cells */
    tone?: string;
    /** height utility */
    height?: string;
    /** smallest cell width in px */
    minCell?: number;
    maxCells?: number;
    /** progress: the boundary cell fills partially so slow runs still move; static readings snap to cells */
    continuous?: boolean;
  }>(),
  { valueText: undefined, tone: "bg-anodized", height: "h-1.5", minCell: 12, maxCells: 48, continuous: false },
);

const el = ref<HTMLElement | null>(null);
const width = ref(0);
let ro: ResizeObserver | null = null;
onMounted(() => {
  if (!el.value) return;
  width.value = el.value.getBoundingClientRect().width;
  if (typeof ResizeObserver === "undefined") return;
  ro = new ResizeObserver(([entry]) => (width.value = entry.contentRect.width));
  ro.observe(el.value);
});
onBeforeUnmount(() => ro?.disconnect());

const gap = computed(() => (width.value < 200 ? 2 : 4));
const count = computed(() => {
  if (width.value <= 0) return 12;
  const n = Math.floor((width.value + gap.value) / (props.minCell + gap.value));
  return Math.max(4, Math.min(props.maxCells, n));
});
const v = computed(() => (Number.isFinite(props.value) ? Math.max(0, Math.min(1, props.value)) : 0));
/** fill of each cell, 0..1 */
const fills = computed(() => {
  const lit = props.continuous ? v.value * count.value : Math.round(v.value * count.value);
  return Array.from({ length: count.value }, (_, i) => Math.max(0, Math.min(1, lit - i)));
});
</script>

<template>
  <div
    ref="el"
    role="meter"
    aria-valuemin="0"
    aria-valuemax="100"
    :aria-valuenow="Math.round(v * 100)"
    :aria-valuetext="valueText"
    :aria-label="label"
    :class="['flex w-full', height]"
    :style="{ gap: `${gap}px` }"
  >
    <span v-for="(f, i) in fills" :key="i" class="relative flex-1 overflow-hidden rounded-[2px] bg-aluminum-deep">
      <span :class="['absolute inset-y-0 left-0 rounded-[2px] transition-[width] duration-300 ease-platen', tone]" :style="{ width: `${(f * 100).toFixed(1)}%` }" />
    </span>
  </div>
</template>
