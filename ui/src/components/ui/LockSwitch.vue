<script setup lang="ts">
// Where a switch would be on a blocked folder: a quartz lock. Pressing it brings the reason into view.
import { focusRing } from "../../lib/styles";
import AppIcon from "./AppIcon.vue";

defineProps<{ label: string; reason: string }>();
const emit = defineEmits<{ press: [] }>();
</script>

<template>
  <button
    type="button"
    aria-disabled="true"
    :aria-label="label"
    :title="reason"
    :class="['relative z-10 grid h-6 w-10 shrink-0 place-items-center rounded-full bg-quartz/25 text-quartz-deep ring-1 ring-quartz/60 transition-[background-color] duration-150 ease-platen hover:bg-quartz/35 [&_svg]:size-4', focusRing]"
    @click.stop.prevent="emit('press')"
  >
    <AppIcon name="lock" />
  </button>
</template>
