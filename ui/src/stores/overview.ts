// The cached overview and the measurement around it. Event driven, so the tray popup and the main window
// (two webviews, two stores) stay in step without talking to each other.

import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { api, on, type ApiError, type MeasurementState, type Overview, type Target } from "../api/client";
import { t } from "../i18n";
import { errorCode, errorText } from "../lib/errors";
import { targetName } from "../lib/targets";
import { useUi } from "./ui";

export const useOverview = defineStore("overview", () => {
  const ui = useUi();
  const data = ref<Overview | null>(null);
  const loadError = ref<ApiError | null>(null);
  /** a measurement is running (event or cached state) */
  const measuring = ref(false);
  /** the engine binary was not found by the last call that needed it */
  const engineMissing = ref(false);
  /** "Procurar de novo" is waiting for the backend to answer */
  const engineBusy = ref(false);
  /** target ids whose switch is waiting for the backend */
  const pending = ref<string[]>([]);

  const targets = computed<Target[]>(() => data.value?.targets ?? []);
  const enabled = computed(() => targets.value.filter((x) => x.enabled));
  const blocked = computed(() => targets.value.filter((x) => !x.allowed));
  const available = computed(() => targets.value.filter((x) => x.allowed && !x.enabled));
  const disk = computed(() => data.value?.disk ?? null);
  const savings = computed(() => data.value?.savings ?? null);
  const docker = computed(() => data.value?.docker ?? null);
  const measurement = computed<MeasurementState>(() => (measuring.value ? "measuring" : (data.value?.measurement.state ?? "never")));
  const measuredAt = computed(() => data.value?.measurement.measuredAt ?? null);
  const neverMeasured = computed(() => measurement.value === "never");
  const ready = computed(() => data.value !== null);

  function noteError(e: unknown) {
    if (errorCode(e) === "engine_missing") engineMissing.value = true;
  }

  function apply(o: Overview) {
    data.value = o;
    if (o.measurement.state !== "measuring") measuring.value = false;
    else measuring.value = true;
  }

  async function load() {
    loadError.value = null;
    try {
      apply(await api.getOverview());
    } catch (e) {
      noteError(e);
      loadError.value = { code: errorCode(e), message: errorText(e) };
    }
  }

  /** Free space and totals after a run; sizes keep their measurement. */
  async function reload() {
    try {
      apply(await api.getOverview());
    } catch (e) {
      noteError(e);
    }
  }

  async function refresh() {
    if (measuring.value) return;
    measuring.value = true;
    if (engineMissing.value) engineBusy.value = true;
    try {
      await api.refreshOverview();
    } catch (e) {
      measuring.value = false;
      engineBusy.value = false;
      noteError(e);
      ui.toast("error", errorText(e));
    }
  }

  async function setEnabled(x: Target, value: boolean) {
    if (pending.value.includes(x.id)) return;
    pending.value = [...pending.value, x.id];
    try {
      const next = await api.setTargetEnabled(x.id, value);
      if (data.value) data.value.targets = data.value.targets.map((old) => (old.id === next.id ? next : old));
      ui.toast("ok", t(next.enabled ? "targets.toastOn" : "targets.toastOff", { name: targetName(next) }));
    } catch (e) {
      noteError(e);
      ui.toast("error", errorText(e));
    } finally {
      pending.value = pending.value.filter((id) => id !== x.id);
    }
  }

  let started = false;
  async function init() {
    if (started) return;
    started = true;
    await Promise.all([
      on("overview://measuring", () => (measuring.value = true)),
      on("overview://updated", (o) => {
        const wasMeasuring = measuring.value;
        apply({ ...o, measurement: o.measurement.state === "measuring" ? { ...o.measurement, state: "fresh" } : o.measurement });
        measuring.value = false;
        engineMissing.value = false;
        engineBusy.value = false;
        if (wasMeasuring) ui.toast("ok", t("toast.measured"));
      }),
      on("overview://failed", (e) => {
        measuring.value = false;
        engineBusy.value = false;
        noteError(e);
        ui.toast("error", errorText(e));
      }),
    ]);
    await load();
  }

  return {
    data, loadError, measuring, engineMissing, engineBusy, pending,
    targets, enabled, blocked, available, disk, savings, docker, measurement, measuredAt, neverMeasured, ready,
    init, load, reload, refresh, setEnabled, noteError,
  };
});
