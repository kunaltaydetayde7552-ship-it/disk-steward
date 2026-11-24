<script setup lang="ts">
// The switch of a target. While the backend answers it holds its new position with a spinner in the knob;
// when it cannot change (a run is going), it stays focusable and says why in its title.
import { focusRing } from "../../lib/styles";
import Spinner from "./Spinner.vue";

const props = withDefaults(defineProps<{ checked: boolean; label: string; pending?: boolean; lockedReason?: string }>(), { pending: false, lockedReason: undefined });
const emit = defineEmits<{ change: [value: boolean] }>();

function press() {
  if (props.pending || props.lockedReason) return;
  emit("change", !props.checked);
}
</script>

<template>
  <button
    type="button"
    role="switch"
    :aria-checked="checked"
    :aria-label="label"
    :aria-busy="pending || undefined"
    :aria-disabled="lockedReason ? 'true' : undefined"
    :title="lockedReason"
    :class="[
      'relative z-10 h-6 w-10 shrink-0 rounded-full transition-[background-color,box-shadow,opacity] duration-200 ease-platen active:shadow-press',
      checked ? 'bg-anodized' : 'bg-aluminum-deep hover:bg-anodized-muted/40',
      lockedReason ? 'opacity-50' : '',
      focusRing,
    ]"
    @click.stop.prevent="press"
  >
    <span :class="['absolute left-1 top-1 grid size-4 place-items-center rounded-full bg-frost text-anodized-muted shadow-lift transition-transform duration-200 ease-platen', checked ? 'translate-x-4' : '']">
      <Spinner v-if="pending" class="!size-2.5 !border-[1.5px]" />
    </span>
  </button>
</template>
