<script setup lang="ts">
// A glyph in a raised circle: the lead of every row.
import AppIcon from "./AppIcon.vue";
import Spinner from "./Spinner.vue";
import type { IconName } from "./icons";
withDefaults(defineProps<{ icon?: IconName; tone?: string; busy?: boolean }>(), { icon: undefined, tone: "text-anodized", busy: false });
</script>

<template>
  <span :class="['grid size-10 shrink-0 place-items-center rounded-full bg-frost shadow-lift ring-1 ring-aluminum-deep', tone]">
    <Spinner v-if="busy" class="!size-[18px]" />
    <AppIcon v-else-if="icon" :name="icon" />
    <slot />
  </span>
</template>
