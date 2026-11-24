// launchd agents. The overview carries them; install/uninstall answer with the fresh list.

import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { api, type ApiError, type ScheduleAgent } from "../api/client";
import { t } from "../i18n";
import { errorCode, errorMessage } from "../lib/errors";
import { useOverview } from "./overview";
import { useUi } from "./ui";

export const COMPRESS_AGENT = "local.auto-compress-apfs";
export const DOCKER_AGENT = "com.lucasbrito.docker-gc";

export const useSchedule = defineStore("schedule", () => {
  const ov = useOverview();
  const ui = useUi();
  /** set by install/uninstall; the overview's list is used until then */
  const local = ref<ScheduleAgent[] | null>(null);
  const busy = ref<"install" | "uninstall" | null>(null);
  const error = ref<ApiError | null>(null);

  const agents = computed(() => local.value ?? ov.data?.schedule ?? []);
  const compress = computed(() => agents.value.find((a) => a.label === COMPRESS_AGENT) ?? null);
  const docker = computed(() => agents.value.find((a) => a.label === DOCKER_AGENT) ?? null);
  const installed = computed(() => compress.value?.loaded ?? false);

  async function change(kind: "install" | "uninstall") {
    if (busy.value) return;
    busy.value = kind;
    error.value = null;
    try {
      local.value = kind === "install" ? await api.installSchedule() : await api.uninstallSchedule();
      ui.toast("ok", t(kind === "install" ? "schedule.installedToast" : "schedule.removedToast"));
    } catch (e) {
      ov.noteError(e);
      error.value = { code: errorCode(e), message: errorMessage(e) };
    } finally {
      busy.value = null;
    }
  }

  return { agents, compress, docker, installed, busy, error, install: () => change("install"), uninstall: () => change("uninstall") };
});
