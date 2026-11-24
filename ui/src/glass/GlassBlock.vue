<script setup lang="ts">
// The hero object. Parameters tween with the platen ease, so the press travels, the glass grows or
// shrinks and the light pool swells instead of jumping. The light color itself comes from --hl.
import { animate } from "motion-v";
import { computed, onBeforeUnmount, shallowRef, useId, watch } from "vue";
import { BV, type BlockParams } from "./geometry";
import { blockShapes } from "./scene";
import ShapeTree from "./ShapeTree";

const props = withDefaults(defineProps<{ params: BlockParams; label: string; instant?: boolean }>(), { instant: false });

const EASE = [0.5, 0, 0.1, 1] as const; // --ease-platen
const TWEEN_KEYS = ["G", "H", "priv", "shared", "pb", "pool"] as const;
const reduce = typeof matchMedia !== "undefined" && matchMedia("(prefers-reduced-motion: reduce)").matches;
const id = `gb${useId().replace(/[^a-zA-Z0-9]/g, "")}`;

const current = shallowRef<BlockParams>({ ...props.params });
const shapes = computed(() => blockShapes(id, current.value));
let anim: { stop: () => void } | null = null;

watch(
  () => props.params,
  (next) => {
    anim?.stop();
    if (props.instant || reduce) {
      current.value = { ...next };
      return;
    }
    const from = { ...current.value };
    const to = { ...next };
    // a press arriving from nowhere drops in from above the block
    if (from.pb == null && to.pb != null) from.pb = to.H + 140;
    anim = animate(0, 1, {
      duration: 0.75,
      ease: EASE,
      onUpdate: (k: number) => {
        const mid: BlockParams = { ...to };
        for (const key of TWEEN_KEYS) {
          const a = from[key];
          const b = to[key];
          if (a == null || b == null) continue;
          mid[key] = a + (b - a) * k;
        }
        current.value = mid;
      },
      onComplete: () => (current.value = { ...to }),
    });
  },
  { deep: true },
);
onBeforeUnmount(() => anim?.stop());

const u = (n: string) => `${id}-${n}`;
</script>

<template>
  <svg :viewBox="`0 0 ${BV.w} ${BV.h}`" width="100%" height="100%" class="overflow-visible" role="img" :aria-label="label">
    <defs>
      <radialGradient :id="u('pool')">
        <stop offset="0" style="stop-color: color-mix(in oklab, var(--hl) 70%, transparent)" />
        <stop offset="0.5" style="stop-color: color-mix(in oklab, var(--hl) 22%, transparent)" />
        <stop offset="1" style="stop-color: transparent" />
      </radialGradient>
      <radialGradient :id="u('glow')">
        <stop offset="0" style="stop-color: color-mix(in oklab, var(--hl) 55%, white); stop-opacity: 0.85" />
        <stop offset="0.45" style="stop-color: var(--hl); stop-opacity: 0.32" />
        <stop offset="1" style="stop-color: var(--hl); stop-opacity: 0" />
      </radialGradient>
      <linearGradient :id="u('gt')" x1="0" y1="0" x2=".4" y2="1">
        <stop offset="0" style="stop-color: var(--glass-top-a)" /><stop offset="1" style="stop-color: var(--glass-top-b)" />
      </linearGradient>
      <linearGradient :id="u('gl')" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0" style="stop-color: var(--glass-left-a)" /><stop offset="1" style="stop-color: var(--glass-left-b)" />
      </linearGradient>
      <linearGradient :id="u('gr')" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0" style="stop-color: var(--glass-right-a)" /><stop offset="1" style="stop-color: var(--glass-right-b)" />
      </linearGradient>
      <!-- fresnel bands: bright at the face edges, clear in the middle -->
      <linearGradient :id="u('fl')" x1="0" y1="0" x2="1" y2="0">
        <stop offset="0" style="stop-color: var(--glass-fresnel)" />
        <stop offset="0.13" style="stop-color: rgb(255 255 255 / 0)" />
        <stop offset="0.85" style="stop-color: rgb(255 255 255 / 0)" />
        <stop offset="1" style="stop-color: var(--glass-fresnel); stop-opacity: 0.7" />
      </linearGradient>
      <linearGradient :id="u('fr')" x1="0" y1="0" x2="1" y2="0">
        <stop offset="0" style="stop-color: var(--glass-fresnel); stop-opacity: 0.8" />
        <stop offset="0.16" style="stop-color: rgb(255 255 255 / 0)" />
        <stop offset="0.88" style="stop-color: rgb(255 255 255 / 0)" />
        <stop offset="1" style="stop-color: var(--glass-fresnel); stop-opacity: 0.55" />
      </linearGradient>
      <linearGradient :id="u('ft')" x1="0" y1="0" x2="1" y2="1">
        <stop offset="0" style="stop-color: var(--glass-fresnel)" />
        <stop offset="0.4" style="stop-color: rgb(255 255 255 / 0)" />
        <stop offset="1" style="stop-color: var(--glass-fresnel); stop-opacity: 0.45" />
      </linearGradient>
      <linearGradient :id="u('cl')" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0" style="stop-color: color-mix(in oklab, var(--hl) 45%, white)" />
        <stop offset="1" style="stop-color: color-mix(in oklab, var(--hl) 92%, white)" />
      </linearGradient>
      <linearGradient :id="u('cr')" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0" style="stop-color: color-mix(in oklab, var(--hl) 75%, white)" />
        <stop offset="1" style="stop-color: color-mix(in oklab, var(--hl) 78%, black)" />
      </linearGradient>
      <!-- the platen: bead-blasted top with a soft diagonal reflection, shaded sides -->
      <linearGradient :id="u('at')" x1="0" y1="0" x2="1" y2="1">
        <stop offset="0" style="stop-color: var(--alu-a)" />
        <stop offset="0.32" style="stop-color: var(--alu-b)" />
        <stop offset="0.5" style="stop-color: var(--alu-c)" />
        <stop offset="0.74" style="stop-color: var(--alu-b)" />
        <stop offset="1" style="stop-color: var(--alu-a)" />
      </linearGradient>
      <linearGradient :id="u('al')" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0" style="stop-color: var(--alu-c)" />
        <stop offset="0.3" style="stop-color: var(--alu-left)" />
        <stop offset="1" style="stop-color: color-mix(in oklab, var(--alu-left) 82%, black)" />
      </linearGradient>
      <linearGradient :id="u('ar')" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0" style="stop-color: var(--alu-left)" />
        <stop offset="0.3" style="stop-color: var(--alu-right)" />
        <stop offset="1" style="stop-color: color-mix(in oklab, var(--alu-right) 78%, black)" />
      </linearGradient>
      <linearGradient :id="u('chrome')" x1="0" y1="0" x2="1" y2="0">
        <stop offset="0" style="stop-color: var(--chrome-dark)" />
        <stop offset="0.22" style="stop-color: var(--chrome-light)" />
        <stop offset="0.42" style="stop-color: var(--chrome-mid)" />
        <stop offset="0.66" style="stop-color: var(--chrome-light)" />
        <stop offset="0.86" style="stop-color: var(--chrome-mid)" />
        <stop offset="1" style="stop-color: var(--chrome-dark)" />
      </linearGradient>
      <filter :id="u('brush')" x="0" y="0" width="100%" height="100%" color-interpolation-filters="sRGB">
        <feTurbulence type="fractalNoise" baseFrequency="0.008 0.75" numOctaves="2" seed="7" result="noise" />
        <feColorMatrix in="noise" type="matrix" values="0 0 0 0 1  0 0 0 0 1  0 0 0 0 1  0.9 0 0 0 -0.32" result="grain" />
        <feComposite in="grain" in2="SourceGraphic" operator="in" result="brushed" />
        <feMerge><feMergeNode in="SourceGraphic" /><feMergeNode in="brushed" /></feMerge>
      </filter>
      <filter :id="u('b18')" x="-60%" y="-60%" width="220%" height="220%"><feGaussianBlur stdDeviation="18" /></filter>
      <filter :id="u('b12')" x="-50%" y="-50%" width="200%" height="200%"><feGaussianBlur stdDeviation="12" /></filter>
      <filter :id="u('b5')" x="-50%" y="-50%" width="200%" height="200%"><feGaussianBlur stdDeviation="5" /></filter>
      <filter :id="u('b2')" x="-30%" y="-30%" width="160%" height="160%"><feGaussianBlur stdDeviation="2" /></filter>
      <filter :id="u('b1')" x="-30%" y="-30%" width="160%" height="160%"><feGaussianBlur stdDeviation="1.1" /></filter>
    </defs>
    <ShapeTree :shapes="shapes" />
  </svg>
</template>
