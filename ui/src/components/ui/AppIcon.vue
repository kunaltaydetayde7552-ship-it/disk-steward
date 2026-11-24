<script setup lang="ts">
import { ICONS, type IconName } from "./icons";
defineProps<{ name: IconName }>();
</script>

<template>
  <!-- eslint-disable-next-line vue/no-v-html -- static glyph table, no user data -->
  <svg viewBox="0 0 24 24" class="size-[18px] shrink-0" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" v-html="ICONS[name]" />
</template>
