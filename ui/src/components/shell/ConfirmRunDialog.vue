<script setup lang="ts">
// The only door to apply(): a native modal dialog (Esc cancels, focus is trapped and returned by the
// platform). Opens from the dock, Cmd+Return, "Tentar de novo" and the tray's "Comprimir agora".
import { nextTick, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { bytes, int, tilde } from "../../lib/format";
import { btnGhost, btnPrimary } from "../../lib/styles";
import { useRun } from "../../stores/run";
import { useUi } from "../../stores/ui";
import AppIcon from "../ui/AppIcon.vue";
import IconDisc from "../ui/IconDisc.vue";

const { t } = useI18n();
const ui = useUi();
const run = useRun();
const dlg = ref<HTMLDialogElement | null>(null);
const go = ref<HTMLButtonElement | null>(null);

watch(
  () => ui.confirmOpen,
  async (open) => {
    const d = dlg.value;
    if (!d) return;
    if (open && !d.open) {
      d.returnValue = "";
      d.showModal();
      await nextTick();
      go.value?.focus();
    } else if (!open && d.open) d.close("cancel");
  },
);

function onClose() {
  const confirmed = dlg.value?.returnValue === "go";
  ui.confirmOpen = false;
  if (confirmed && run.phase === "preview") void run.apply();
}
</script>

<template>
  <dialog
    ref="dlg"
    aria-labelledby="confirm-title"
    aria-describedby="confirm-body"
    class="m-auto w-[440px] max-w-[calc(100vw-32px)] rounded-panel bg-frost p-0 text-anodized shadow-float ring-1 ring-aluminum-deep backdrop:bg-anodized/30 backdrop:backdrop-blur-sm"
    @close="onClose"
  >
    <form v-if="run.preview" method="dialog" class="p-6">
      <div class="flex items-start gap-4">
        <IconDisc icon="compress" tone="text-magsafe-amber-deep" />
        <div>
          <h2 id="confirm-title" class="text-title font-medium">{{ t("confirm.title", { files: int(run.preview.files) }) }}</h2>
          <p id="confirm-body" class="mt-2 text-anodized-muted">{{ t("confirm.body") }}</p>
        </div>
      </div>
      <dl class="mt-5 divide-y divide-aluminum-deep border-y border-aluminum-deep">
        <div class="flex justify-between gap-4 py-2"><dt class="text-anodized-muted">{{ t("confirm.logical") }}</dt><dd class="tabular-nums">{{ bytes(run.preview.bytes) }}</dd></div>
        <div class="py-2">
          <div class="flex justify-between gap-4"><dt class="text-anodized-muted">{{ t("confirm.roots") }}</dt><dd class="tabular-nums">{{ run.preview.roots.length }}</dd></div>
          <ul class="mt-1 space-y-0.5">
            <li v-for="r in run.preview.roots" :key="r.path" class="flex justify-between gap-4 text-micro text-anodized-muted">
              <span class="min-w-0 truncate">{{ tilde(r.path) }}</span><span class="shrink-0 tabular-nums">{{ bytes(r.bytes) }}</span>
            </li>
          </ul>
        </div>
        <div class="flex justify-between gap-4 py-2"><dt class="text-anodized-muted">{{ t("confirm.skipped") }}</dt><dd class="tabular-nums">{{ int(run.preview.skipped) }}</dd></div>
      </dl>
      <div class="mt-6 flex justify-end gap-2">
        <button type="submit" value="cancel" :class="btnGhost">{{ t("confirm.cancel") }}</button>
        <button ref="go" type="submit" value="go" :class="[btnPrimary, '[&_svg]:size-4']"><AppIcon name="compress" />{{ t("confirm.go", { bytes: bytes(run.preview.bytes) }) }}</button>
      </div>
    </form>
  </dialog>
</template>
