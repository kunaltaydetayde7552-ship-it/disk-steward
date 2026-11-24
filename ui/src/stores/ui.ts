// Shell state: the hash route (no vue-router: five routes, one store, like the reference app), toasts,
// the run confirmation and the short-lived "Copiado" labels.

import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { t } from "../i18n";

export type ScreenId = "overview" | "targets" | "runs" | "schedule";

export interface Route {
  name: ScreenId | "tray";
  /** `#/alvos/:id` */
  targetId: string | null;
}

export const SCREENS: { id: ScreenId; path: string; icon: string }[] = [
  { id: "overview", path: "/", icon: "overview" },
  { id: "targets", path: "/alvos", icon: "targets" },
  { id: "runs", path: "/execucoes", icon: "runs" },
  { id: "schedule", path: "/agendamento", icon: "schedule" },
];

export function parseHash(hash: string): Route {
  const path = decodeURIComponent(hash.replace(/^#/, "")) || "/";
  const [, head = "", tail = null] = path.split("/");
  if (head === "tray") return { name: "tray", targetId: null };
  if (head === "alvos") return { name: "targets", targetId: tail || null };
  if (head === "execucoes") return { name: "runs", targetId: null };
  if (head === "agendamento") return { name: "schedule", targetId: null };
  return { name: "overview", targetId: null };
}

export const pathOf = (id: ScreenId, targetId?: string | null): string =>
  id === "targets" && targetId ? `/alvos/${encodeURIComponent(targetId)}` : SCREENS.find((s) => s.id === id)!.path;

export interface Toast {
  id: number;
  kind: "ok" | "error";
  text: string;
}

export const useUi = defineStore("ui", () => {
  const route = ref<Route>(parseHash(location.hash));
  const screen = computed<ScreenId>(() => (route.value.name === "tray" ? "overview" : route.value.name));
  const toasts = ref<Toast[]>([]);
  const confirmOpen = ref(false);
  /** text recently copied: its button reads "Copiado" for a moment */
  const copied = ref<string | null>(null);
  /** id of the risk box to bring into view and nudge (a click on a locked switch) */
  const nudge = ref<{ id: string; n: number } | null>(null);

  window.addEventListener("hashchange", () => (route.value = parseHash(location.hash)));

  function go(path: string) {
    if (location.hash !== `#${path}`) location.hash = path;
  }
  const goScreen = (id: ScreenId, targetId?: string | null) => go(pathOf(id, targetId));

  let seq = 0;
  function toast(kind: Toast["kind"], text: string) {
    const item = { id: ++seq, kind, text };
    toasts.value = [...toasts.value.slice(-2), item];
    setTimeout(() => (toasts.value = toasts.value.filter((x) => x.id !== item.id)), kind === "error" ? 5200 : 2800);
  }

  let copiedTimer: ReturnType<typeof setTimeout> | undefined;
  async function copy(text: string) {
    try {
      await navigator.clipboard.writeText(text);
    } catch {
      // WebKit refuses clipboard writes outside a trusted gesture chain: fall back to a selection copy
      const area = document.createElement("textarea");
      area.value = text;
      area.setAttribute("readonly", "");
      area.style.position = "fixed";
      area.style.opacity = "0";
      document.body.append(area);
      area.select();
      const ok = document.execCommand("copy");
      area.remove();
      if (!ok) return toast("error", t("toast.copyFailed"));
    }
    copied.value = text;
    clearTimeout(copiedTimer);
    copiedTimer = setTimeout(() => (copied.value = null), 1600);
  }

  function nudgeRisk(id: string) {
    nudge.value = { id, n: (nudge.value?.n ?? 0) + 1 };
  }

  return { route, screen, toasts, confirmOpen, copied, nudge, go, goScreen, toast, copy, nudgeRisk };
});
