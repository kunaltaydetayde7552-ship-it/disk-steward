<script setup lang="ts">
// Short confirmations bottom right, above the dock. Errors are translated text only; raw engine lines live in the run log.
import { useUi } from "../../stores/ui";
import AppIcon from "../ui/AppIcon.vue";

const ui = useUi();
</script>

<template>
  <div role="status" aria-live="polite" class="pointer-events-none fixed bottom-28 right-6 z-50 flex flex-col items-end gap-2 pl-6">
    <TransitionGroup
      enter-active-class="transition-[opacity,translate,scale] duration-300 ease-platen"
      enter-from-class="opacity-0 translate-y-2 scale-[0.98]"
      leave-active-class="transition-[opacity,translate,scale] duration-200 ease-platen"
      leave-to-class="opacity-0 translate-y-1"
      move-class="transition-transform duration-300 ease-platen"
    >
      <p
        v-for="x in ui.toasts"
        :key="x.id"
        :class="[
          'flex max-w-[440px] items-center gap-2 rounded-full bg-frost/95 py-2 pl-2 pr-4 shadow-float ring-1 backdrop-blur-xl [&_svg]:size-4',
          x.kind === 'error' ? 'text-signal-red-deep ring-signal-red/40' : 'text-anodized ring-aluminum-deep',
        ]"
      >
        <span :class="['grid size-6 shrink-0 place-items-center rounded-full text-frost', x.kind === 'error' ? 'bg-signal-red' : 'bg-magsafe-green']">
          <AppIcon :name="x.kind === 'error' ? 'alert' : 'check'" />
        </span>
        <span class="text-pretty">{{ x.text }}</span>
      </p>
    </TransitionGroup>
  </div>
</template>
