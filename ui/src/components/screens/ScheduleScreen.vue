<script setup lang="ts">
// Agendamento: the two launchd agents. The press over the block is lit when the daily agent is loaded.
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { errorText } from "../../lib/errors";
import { btnGhost, btnSecondary, divided, screenLead, screenTitle } from "../../lib/styles";
import { useOverview } from "../../stores/overview";
import { COMPRESS_AGENT, DOCKER_AGENT, useSchedule } from "../../stores/schedule";
import PathLine from "../schedule/PathLine.vue";
import AppIcon from "../ui/AppIcon.vue";
import InfoRow from "../ui/InfoRow.vue";
import Spinner from "../ui/Spinner.vue";

const { t } = useI18n();
const ov = useOverview();
const sched = useSchedule();

const compress = computed(() => {
  const a = sched.compress;
  if (!a?.loaded) return { value: t("schedule.off"), note: t("schedule.notInstalled"), tone: "text-anodized-muted", bad: false };
  if (a.lastExitStatus == null) return { value: t("schedule.loaded"), note: t("schedule.installed"), tone: "text-magsafe-green-deep", bad: false };
  if (a.lastExitStatus === 0) return { value: t("schedule.loaded"), note: t("schedule.installedOk"), tone: "text-magsafe-green-deep", bad: false };
  return { value: t("schedule.loaded"), note: t("schedule.exitFailed", { code: a.lastExitStatus }), tone: "text-signal-red-deep", bad: true };
});
const docker = computed(() => {
  const a = sched.docker;
  if (!a?.loaded) return { value: t("schedule.off"), note: t("schedule.dockerNotLoaded"), tone: "text-anodized-muted" };
  if (a.lastExitStatus == null) return { value: t("schedule.loaded"), note: t("schedule.dockerNoExit"), tone: "text-anodized" };
  if (a.lastExitStatus === 0) return { value: t("schedule.loaded"), note: t("schedule.dockerOk"), tone: "text-magsafe-green-deep" };
  return { value: t("schedule.loaded"), note: t("schedule.exitFailed", { code: a.lastExitStatus }), tone: "text-signal-red-deep" };
});
</script>

<template>
  <header>
    <h1 :class="screenTitle" tabindex="-1">{{ t("schedule.title") }}</h1>
    <p :class="screenLead">{{ t("schedule.lead") }}</p>
  </header>
  <div class="mt-4 grid grid-cols-[4fr_6fr] items-start gap-8 @max-[44rem]:grid-cols-1 @max-[44rem]:gap-2">
    <div class="flex justify-center pt-2 @min-[44rem]:sticky @min-[44rem]:top-0">
      <div data-hero class="aspect-[520/600] w-full max-w-[340px] max-h-[calc(100dvh-280px)] @max-[44rem]:max-w-[220px]" />
    </div>
    <section class="min-w-0 max-w-[560px] pt-4">
      <div :class="divided">
        <InfoRow icon="compress" :tone="compress.tone" :label="t('schedule.compress')" :note="compress.note" :value="compress.value" :value-tone="sched.installed ? '' : 'text-anodized-muted'">
          <p class="mt-1 text-micro text-anodized-muted">{{ sched.compress?.label ?? COMPRESS_AGENT }}</p>
          <div class="mt-3 flex flex-wrap items-center gap-2">
            <button v-if="sched.installed" type="button" :disabled="sched.busy !== null" :class="[btnGhost, '[&_svg]:size-4']" @click="sched.uninstall()">
              <Spinner v-if="sched.busy === 'uninstall'" />{{ sched.busy === "uninstall" ? t("schedule.removing") : t("schedule.remove") }}
            </button>
            <button
              v-else
              type="button"
              :disabled="sched.busy !== null || ov.engineMissing"
              :title="ov.engineMissing ? t('schedule.needsEngine') : undefined"
              :class="[btnSecondary, '[&_svg]:size-4']"
              @click="sched.install()"
            >
              <Spinner v-if="sched.busy === 'install'" /><AppIcon v-else name="install" />{{ sched.busy === "install" ? t("schedule.installing") : t("schedule.install") }}
            </button>
            <span v-if="ov.engineMissing && !sched.installed" class="text-micro text-anodized-muted">{{ t("schedule.needsEngine") }}</span>
          </div>
          <div v-if="sched.error" role="alert" class="enter mt-3 rounded-key bg-signal-red/8 p-3 ring-1 ring-signal-red/30">
            <p class="flex items-center gap-2 font-medium text-signal-red-deep [&_svg]:size-4"><AppIcon name="alert" />{{ t("schedule.failTitle") }}</p>
            <p class="mt-1 text-micro text-anodized-muted">{{ errorText(sched.error) }}</p>
          </div>
          <PathLine :path="sched.compress?.logPath ?? null" />
        </InfoRow>

        <InfoRow icon="docker" :tone="docker.tone" :label="t('schedule.docker')" :note="docker.note" :value="docker.value" :value-tone="sched.docker?.loaded ? '' : 'text-anodized-muted'">
          <p class="mt-1 text-micro text-anodized-muted">{{ sched.docker?.label ?? DOCKER_AGENT }}</p>
          <PathLine :path="sched.docker?.logPath ?? null" />
        </InfoRow>
      </div>
      <p class="mt-4 max-w-[56ch] text-micro text-anodized-muted">{{ t("schedule.footnote") }}</p>
    </section>
  </div>
</template>
