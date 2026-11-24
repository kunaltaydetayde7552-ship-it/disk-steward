<script setup lang="ts">
// Alvos: the folders in three groups (on, can be on, stay out) and the selected one in detail.
// At window sizes the screen fits the viewport: the list and the panel scroll inside themselves.
import { computed, onMounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useScrollEdges } from "../../composables/useScrollEdges";
import { btnSecondary, screenLead, screenTitle } from "../../lib/styles";
import { useOverview } from "../../stores/overview";
import { useRun } from "../../stores/run";
import { useSelectedTarget } from "../../stores/scene";
import TargetDetail from "../targets/TargetDetail.vue";
import TargetRow from "../targets/TargetRow.vue";
import AppIcon from "../ui/AppIcon.vue";
import Spinner from "../ui/Spinner.vue";

const { t } = useI18n();
const ov = useOverview();
const run = useRun();
const selected = useSelectedTarget();

const groups = computed(() =>
  [
    { id: "on", label: t("targets.groups.on"), list: ov.enabled },
    { id: "available", label: t("targets.groups.available"), list: ov.available },
    { id: "blocked", label: t("targets.groups.blocked"), list: ov.blocked },
  ].filter((g) => g.list.length),
);

// the list fades out over its last row while more lies below, so a cut never slices through a line
const list = ref<HTMLElement | null>(null);
const { more, measure } = useScrollEdges(list);

// A deep link (#/alvos/rust-target) can select a row below the list's fold; bring it into view.
const revealSelected = () => document.querySelector('[data-target-link][aria-current="true"]')?.scrollIntoView({ block: "nearest" });
watch(() => selected.value?.id, revealSelected, { flush: "post" });
onMounted(revealSelected);

/** Up and Down walk the list like a source list: focus and selection move together. */
function onKey(e: KeyboardEvent) {
  if (e.key !== "ArrowDown" && e.key !== "ArrowUp") return;
  const links = [...document.querySelectorAll<HTMLAnchorElement>("[data-target-link]")];
  if (!links.length) return;
  e.preventDefault();
  const at = links.indexOf(document.activeElement as HTMLAnchorElement);
  const from = at >= 0 ? at : links.findIndex((a) => a.getAttribute("aria-current") === "true");
  const next = links[Math.max(0, Math.min(links.length - 1, from + (e.key === "ArrowDown" ? 1 : -1)))];
  next.focus();
  next.click();
}
</script>

<template>
  <div class="flex flex-col @min-[44rem]:h-full">
    <header class="flex shrink-0 items-start justify-between gap-4 @max-[44rem]:flex-wrap">
      <div class="min-w-0">
        <h1 :class="screenTitle" tabindex="-1">{{ t("targets.title") }}</h1>
        <p :class="[screenLead, 'text-balance']">{{ t("targets.lead") }}</p>
      </div>
      <button type="button" :class="[btnSecondary, '[&_svg]:size-4']" :disabled="ov.measuring || run.busy" aria-keyshortcuts="Meta+R" @click="ov.refresh()">
        <Spinner v-if="ov.measuring" /><AppIcon v-else name="refresh" />{{ ov.measuring ? t("targets.measuring") : t("targets.remeasure") }}
      </button>
    </header>

    <div class="mt-5 grid min-h-0 flex-1 grid-cols-[minmax(280px,min(400px,44%))_minmax(0,1fr)] grid-rows-[minmax(0,1fr)] items-start gap-6 @max-[44rem]:grid-cols-1 @max-[44rem]:grid-rows-none [@media(max-height:700px)]:mt-4">
      <div
        ref="list"
        :class="['-mx-1 min-h-0 overflow-y-auto overscroll-contain px-1 pb-4 @min-[44rem]:max-h-full', more ? '[mask-image:linear-gradient(to_bottom,black_calc(100%-56px),transparent)]' : '']"
        @scroll.passive="measure"
        @keydown="onKey"
      >
        <div>
          <template v-for="g in groups" :key="g.id">
            <h2 class="mb-1 mt-4 px-3 text-micro text-anodized-muted first:mt-0">{{ g.label }}</h2>
            <ul class="flex flex-col gap-1">
              <TargetRow v-for="x in g.list" :key="x.id" :target="x" :selected="selected?.id === x.id" />
            </ul>
          </template>
        </div>
      </div>
      <TargetDetail v-if="selected" :target="selected" />
    </div>
  </div>
</template>
