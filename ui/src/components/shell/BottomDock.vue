<script setup lang="ts">
// The dock: one line of what is pending and the action that moves it. It sits in flow under the scroll
// viewport, so content and the hero never slide beneath it. The situation swaps with the platen ease.
import { useI18n } from "vue-i18n";
import type { DockModel } from "../../composables/useDock";
import { btnGhost, btnPrimary, btnSecondary } from "../../lib/styles";
import { useRun } from "../../stores/run";
import AppIcon from "../ui/AppIcon.vue";
import SegmentMeter from "../ui/SegmentMeter.vue";
import Spinner from "../ui/Spinner.vue";

defineProps<{ model: DockModel }>();
const { t } = useI18n();
const run = useRun();
const KIND = { primary: btnPrimary, secondary: btnSecondary, ghost: btnGhost } as const;
</script>

<template>
  <div class="relative z-30 shrink-0 px-6 pb-5 pt-2 max-md:px-3 max-md:pb-3">
    <section
      :aria-label="t('dock.progress')"
      class="relative flex min-h-16 items-center gap-x-4 overflow-hidden rounded-panel bg-frost/88 px-4 py-3 shadow-float ring-1 ring-aluminum-deep backdrop-blur-xl max-md:flex-wrap max-md:gap-y-3"
    >
      <Transition
        mode="out-in"
        enter-active-class="transition-[opacity,translate,scale] duration-300 ease-platen"
        enter-from-class="opacity-0 translate-y-1.5"
        leave-active-class="transition-[opacity,translate,scale] duration-150 ease-platen"
        leave-to-class="opacity-0 -translate-y-1"
      >
        <div :key="model.key" class="flex min-w-0 flex-1 items-center gap-x-4 max-md:flex-wrap max-md:gap-y-3">
          <div class="flex min-w-0 max-w-[520px] items-center gap-3" aria-live="polite">
            <span v-if="model.tone === 'busy' || model.tone === 'run'" class="grid size-9 shrink-0 place-items-center rounded-full bg-magsafe-amber/15 text-magsafe-amber-deep"><Spinner /></span>
            <span v-else-if="model.tone === 'count'" class="grid h-9 min-w-9 shrink-0 place-items-center rounded-full bg-magsafe-amber/15 px-2 text-micro font-semibold tabular-nums text-magsafe-amber-deep">{{ model.count }}</span>
            <span v-else-if="model.tone === 'ok'" class="grid size-9 shrink-0 place-items-center rounded-full bg-magsafe-green text-frost shadow-[0_0_12px_color-mix(in_oklab,var(--color-magsafe-green)_60%,transparent)] [&_svg]:size-4"><AppIcon :name="model.icon ?? 'check'" /></span>
            <span v-else-if="model.tone === 'red'" class="grid size-9 shrink-0 place-items-center rounded-full bg-signal-red text-frost [&_svg]:size-4"><AppIcon :name="model.icon ?? 'alert'" /></span>
            <span v-else class="grid size-9 shrink-0 place-items-center rounded-full bg-aluminum text-anodized-muted [&_svg]:size-4"><AppIcon :name="model.icon ?? 'eye'" /></span>
            <span class="min-w-0">
              <span :class="['block font-medium md:truncate max-md:line-clamp-2', model.tone === 'red' ? 'text-signal-red-deep' : '']">{{ model.title }}</span>
              <span class="block truncate text-micro text-anodized-muted max-md:hidden">{{ model.sub }}</span>
            </span>
          </div>
          <div v-if="model.bar" class="min-w-[120px] flex-1 @max-[44rem]:hidden">
            <SegmentMeter continuous :value="run.fraction" :label="t('dock.progress')" tone="bg-magsafe-amber" height="h-1" :min-cell="8" :max-cells="40" />
          </div>
          <div class="ml-auto flex shrink-0 items-center gap-2">
            <button
              v-for="a in model.actions"
              :key="a.id"
              type="button"
              :data-dock-primary="a.primary ? '' : undefined"
              :disabled="a.disabled"
              :aria-keyshortcuts="a.primary ? 'Meta+Enter' : undefined"
              :class="[KIND[a.kind], '[&_svg]:size-4']"
              @click="a.run()"
            >
              <Spinner v-if="a.busy" />
              <AppIcon v-else-if="a.icon" :name="a.icon" />
              {{ a.label }}
            </button>
          </div>
        </div>
      </Transition>
      <!-- the run's progress also runs along the dock's lower edge, visible even when the meter is hidden -->
      <span
        v-if="model.bar"
        class="absolute inset-x-0 bottom-0 h-[2px] origin-left bg-magsafe-amber transition-transform duration-300 ease-platen"
        :style="{ transform: `scaleX(${run.fraction})` }"
        aria-hidden="true"
      />
    </section>
  </div>
</template>
