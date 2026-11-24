<script setup lang="ts">
// The main window: sidebar, a scroll viewport with the traveling glass block over it, and the dock in flow
// below. The floor (main) carries the object's light, so the pool of light runs under the dock as well.
import { computed, nextTick, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useDock } from "../../composables/useDock";
import { useShortcuts } from "../../composables/useShortcuts";
import HeroStage from "../../glass/HeroStage.vue";
import { LIGHT_VAR } from "../../lib/light";
import { btnSecondary, focusRing } from "../../lib/styles";
import { useOverview } from "../../stores/overview";
import { useGlobalLight } from "../../stores/scene";
import { useUi } from "../../stores/ui";
import OverviewScreen from "../screens/OverviewScreen.vue";
import RunsScreen from "../screens/RunsScreen.vue";
import ScheduleScreen from "../screens/ScheduleScreen.vue";
import TargetsScreen from "../screens/TargetsScreen.vue";
import AppIcon from "../ui/AppIcon.vue";
import Spinner from "../ui/Spinner.vue";
import BottomDock from "./BottomDock.vue";
import ConfirmRunDialog from "./ConfirmRunDialog.vue";
import Sidebar from "./Sidebar.vue";
import ToastStack from "./ToastStack.vue";

const { t } = useI18n();
const ui = useUi();
const ov = useOverview();
const light = useGlobalLight();
const dock = useDock();
useShortcuts(dock);

const SCREENS = { overview: OverviewScreen, targets: TargetsScreen, runs: RunsScreen, schedule: ScheduleScreen } as const;
const scroller = ref<HTMLElement | null>(null);
/** Targets fits the viewport and scrolls inside its list and panel; the rest scroll as a page. */
const fit = computed(() => ui.screen === "targets");

const skip = () => document.querySelector<HTMLElement>("#screen h1, #screen")?.focus();

watch(
  () => ui.screen,
  async () => {
    scroller.value?.scrollTo({ top: 0 });
    await nextTick();
    // land keyboard and screen reader users on the new screen's title
    document.querySelector<HTMLElement>("#screen h1")?.focus({ preventScroll: true });
  },
);
</script>

<template>
  <div class="relative flex h-full overflow-hidden bg-aluminum max-md:flex-col">
    <!-- a button, not an anchor: the hash is the route -->
    <button type="button" :class="['sr-only z-[60] rounded-key bg-frost px-3 py-2 shadow-float focus:not-sr-only focus:absolute focus:left-3 focus:top-3', focusRing]" @click="skip">{{ t("app.skip") }}</button>
    <Sidebar />
    <main data-floor class="@container relative flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden" :style="{ '--hl': LIGHT_VAR[light] }">
      <div class="ambient pointer-events-none absolute inset-0" aria-hidden="true" />
      <div class="absolute inset-x-0 top-0 z-10 h-7" data-tauri-drag-region aria-hidden="true" />
      <div class="relative min-h-0 flex-1">
        <div ref="scroller" class="absolute inset-0 overflow-y-auto overflow-x-hidden overscroll-contain">
          <div :class="['relative px-8 pb-8 pt-10 max-lg:px-6 max-md:px-4 max-md:pt-6 [@media(max-height:700px)]:pt-8', fit ? 'min-h-full @min-[44rem]:h-full' : 'min-h-full']">
            <div v-if="!ov.ready && ov.loadError" class="grid min-h-[60vh] place-items-center">
              <div class="flex max-w-[44ch] flex-col items-center gap-4 text-center">
                <span class="grid size-12 place-items-center rounded-full bg-signal-red text-frost"><AppIcon name="alert" /></span>
                <h1 class="text-title font-medium" tabindex="-1">{{ t("app.startFailed") }}</h1>
                <p class="text-anodized-muted">{{ ov.loadError.message }}</p>
                <button type="button" :class="[btnSecondary, '[&_svg]:size-4']" @click="ov.load()"><AppIcon name="refresh" />{{ t("app.retry") }}</button>
              </div>
            </div>
            <div v-else-if="!ov.ready" class="grid min-h-[60vh] place-items-center text-anodized-muted" aria-busy="true">
              <span class="inline-flex items-center gap-3"><Spinner />{{ t("app.loading") }}</span>
            </div>
            <div v-else id="screen" :key="ui.screen" tabindex="-1" :class="['screen-in outline-none', fit ? '@min-[44rem]:h-full' : '']">
              <component :is="SCREENS[ui.screen]" />
            </div>
          </div>
        </div>
        <HeroStage />
      </div>
      <BottomDock :model="dock" />
    </main>
    <ConfirmRunDialog />
    <ToastStack />
  </div>
</template>
