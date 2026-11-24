<script setup lang="ts">
// Legend glyphs that match the block: dashed ghost = logical, clear glass = on disk, lit core = private,
// hatched slab = shared by clone.
withDefaults(defineProps<{ kind: "logical" | "disk" | "private" | "shared"; fill?: string }>(), { fill: "var(--core-unlit-left)" });
</script>

<template>
  <svg viewBox="0 0 16 16" class="size-4 shrink-0" aria-hidden="true">
    <rect v-if="kind === 'logical'" x="1.5" y="1.5" width="13" height="13" rx="2" fill="none" stroke="currentColor" stroke-width="1.2" stroke-dasharray="2.4 2" />
    <rect v-else-if="kind === 'private'" x="1.5" y="1.5" width="13" height="13" rx="2" :style="{ fill }" stroke="currentColor" stroke-opacity=".5" stroke-width="1.2" />
    <template v-else>
      <rect x="1.5" y="1.5" width="13" height="13" rx="2" fill="var(--color-frost)" stroke="currentColor" stroke-width="1.2" />
      <path v-if="kind === 'shared'" d="M3 5.5h10M3 8h10M3 10.5h10" stroke="currentColor" stroke-opacity=".55" stroke-width=".9" />
    </template>
  </svg>
</template>
