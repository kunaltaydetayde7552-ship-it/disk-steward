<script setup lang="ts">
// Where the engine's savings are: the part the enabled folders explain (green) and the part no folder
// accounts for (hatched quartz, like the shared slab of the block). Honest, not hidden in a footnote.
import { computed, ref, useId } from "vue";
import { useI18n } from "vue-i18n";
import { bytes } from "../../lib/format";
import { focusRing } from "../../lib/styles";

const props = defineProps<{ saved: number; outside: number }>();
const { t } = useI18n();
const outside = computed(() => Math.max(0, Math.min(props.saved, props.outside)));
const inside = computed(() => Math.max(0, props.saved - outside.value));
const share = computed(() => (props.saved > 0 ? (outside.value / props.saved) * 100 : 0));
const open = ref(false);
const whyId = `why${useId().replace(/[^a-zA-Z0-9]/g, "")}`;
const label = computed(() => t("overview.savedSplit", { inside: bytes(inside.value), outside: bytes(outside.value) }));
</script>

<template>
  <div class="mt-3">
    <div role="img" :aria-label="label" class="flex h-2 w-full gap-[3px]">
      <span class="h-full rounded-[2px] bg-magsafe-green transition-[flex-basis] duration-500 ease-platen" :style="{ flexBasis: `${100 - share}%` }" />
      <span v-if="share > 0" class="hatch h-full rounded-[2px] ring-1 ring-quartz/60 ring-inset transition-[flex-basis] duration-500 ease-platen" :style="{ flexBasis: `${share}%`, minWidth: '6px' }" />
    </div>
    <div class="mt-2 flex flex-wrap items-center gap-x-4 gap-y-1 text-micro text-anodized-muted">
      <span class="inline-flex items-center gap-2" aria-hidden="true"><span class="size-2.5 rounded-[2px] bg-magsafe-green" />{{ bytes(inside) }} {{ t("overview.savedInside") }}</span>
      <span class="inline-flex items-center gap-2" aria-hidden="true"><span class="hatch size-2.5 rounded-[2px] ring-1 ring-quartz/60 ring-inset" />{{ bytes(outside) }} {{ t("overview.savedOutsideLegend") }}</span>
      <button
        type="button"
        :aria-expanded="open"
        :aria-controls="whyId"
        :class="['-mx-1 rounded-[6px] px-1 font-medium text-anodized underline decoration-aluminum-deep underline-offset-4 transition-colors duration-150 hover:decoration-anodized', focusRing]"
        @click="open = !open"
      >{{ open ? t("overview.savedWhyHide") : t("overview.savedWhy") }}</button>
    </div>
    <Transition
      enter-active-class="transition-[opacity,translate,scale] duration-300 ease-platen"
      enter-from-class="opacity-0 -translate-y-1"
      leave-active-class="transition-opacity duration-150"
      leave-to-class="opacity-0"
    >
      <p v-if="open" :id="whyId" class="mt-2 max-w-[56ch] text-micro text-pretty text-anodized-muted">{{ t("overview.savedOutside", { outside: bytes(outside) }) }}</p>
    </Transition>
  </div>
</template>
