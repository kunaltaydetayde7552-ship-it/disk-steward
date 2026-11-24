// The numbers of the approved prototype (.design-arena/variantes/claude-a.html), in docs/API.md shapes.

import type { DockerUsage, Overview, Preview, RunResult, ScheduleAgent, Target } from "./types";

export const MOCK_HOME = "/Users/dev";
const GiB = 1024 ** 3;
const MiB = 1024 ** 2;
const gib = (n: number) => Math.round(n * GiB);
const mib = (n: number) => Math.round(n * MiB);
const home = (p: string) => `${MOCK_HOME}/${p}`;

export const MEASURED_AT = "2026-11-24T18:42:00Z";
/** measured three days ago: older than the backend's 6 h freshness window */
export const STALE_AT = "2026-11-21T12:12:00Z";

export function mockTargets(): Target[] {
  return [
    { id: "rustup", path: home(".rustup"), kind: "toolchain", enabled: true, allowed: true, logicalBytes: gib(3.1), onDiskBytes: gib(1.3), privateBytes: gib(1.3), totalFiles: 9840, compressedFiles: 9512, risk: "none", riskDetail: null },
    { id: "cargo-registry", path: home(".cargo/registry"), kind: "package_cache", enabled: true, allowed: true, logicalBytes: gib(2.0), onDiskBytes: gib(1.1), privateBytes: gib(1.1), totalFiles: 11630, compressedFiles: 10904, risk: "none", riskDetail: null },
    { id: "npm", path: home(".npm"), kind: "package_cache", enabled: true, allowed: true, logicalBytes: gib(1.4), onDiskBytes: gib(1.1), privateBytes: gib(1.1), totalFiles: 3412, compressedFiles: 3287, risk: "none", riskDetail: null },
    { id: "mise", path: home(".local/share/mise"), kind: "toolchain", enabled: false, allowed: true, logicalBytes: gib(1.4), onDiskBytes: gib(1.4), privateBytes: gib(1.4), totalFiles: 6215, compressedFiles: 0, risk: "none", riskDetail: null },
    {
      id: "uv-cache", path: home(".cache/uv"), kind: "package_cache", enabled: false, allowed: false,
      logicalBytes: gib(7.6), onDiskBytes: gib(7.4), privateBytes: gib(4.7), totalFiles: 182400, compressedFiles: 0, risk: "clone_shared",
      riskDetail: "23 venvs hold APFS clones of files in this cache; compressing it would give each venv its own copy.",
    },
    {
      id: "bun-store", path: home(".bun/install"), kind: "store", enabled: false, allowed: false,
      logicalBytes: gib(4.3), onDiskBytes: gib(4.2), privateBytes: mib(111), totalFiles: 268900, compressedFiles: 0, risk: "blocked_by_rule",
      riskDetail: "bun installs by clonefile; compressing the store un-shares the clones.",
    },
    { id: "pnpm-store", path: home("Library/pnpm"), kind: "store", enabled: false, allowed: true, logicalBytes: gib(3.0), onDiskBytes: gib(2.9), privateBytes: mib(453), totalFiles: 41200, compressedFiles: 0, risk: "none", riskDetail: null },
    {
      id: "rust-target", path: "~/**/target", kind: "build", enabled: false, allowed: false,
      logicalBytes: 0, onDiskBytes: 0, privateBytes: null, totalFiles: 0, compressedFiles: 0, risk: "rewritten_often",
      riskDetail: "cargo rewrites target directories on every build.",
    },
  ];
}

export function mockSchedule(): ScheduleAgent[] {
  return [
    { label: "local.auto-compress-apfs", loaded: false, lastExitStatus: null, logPath: "~/Library/Logs/auto-compress-apfs/launchd.log" },
    { label: "com.lucasbrito.docker-gc", loaded: true, lastExitStatus: 0, logPath: "~/Library/Logs/docker-gc.log" },
  ];
}

export function mockDocker(available = true): DockerUsage {
  return available
    ? { available: true, imagesBytes: gib(21.7), buildCacheBytes: gib(8.1), volumesBytes: gib(17.1), reclaimableBytes: gib(14.6) }
    : { available: false, imagesBytes: 0, buildCacheBytes: 0, volumesBytes: 0, reclaimableBytes: 0 };
}

export function mockOverview(): Overview {
  return {
    disk: { volume: "/System/Volumes/Data", totalBytes: gib(460.4), freeBytes: gib(75.2), usedBytes: gib(385.2), measuredAt: MEASURED_AT },
    // enabled targets explain 1.8 + 0.9 + 0.3 = 3.0 GiB of the 3.5 GiB the engine reports
    savings: { files: 23703, originalBytes: gib(4.9), onDiskBytes: gib(1.4), savedBytes: gib(3.5), ratio: 0.714, quarantined: 0, lastRunDay: "2026-11-21", unattributedBytes: gib(0.5) },
    targets: mockTargets(),
    schedule: mockSchedule(),
    docker: mockDocker(),
    measurement: { state: "fresh", measuredAt: MEASURED_AT },
  };
}

export const PREVIEW: Preview = {
  files: 1312,
  bytes: mib(421),
  skipped: 37,
  roots: [
    { path: home(".cargo/registry"), bytes: mib(212) },
    { path: home(".npm"), bytes: mib(148) },
    { path: home(".rustup"), bytes: mib(61) },
  ],
};
/** files of each preview root, in order: the mock walks them to emit `currentPath` */
export const ROOT_FILES = [655, 498, 159];

export const RUN_FREED = mib(198);
export const RUN_DURATION_MS = 48200;
export const FAILED_AT_FILES = 604;
export const FAILED_FREED = mib(74);
export const FAILED_ERRORS = [
  `ERROR verify: ${home(".npm/_cacache/content-v2/sha512/9c/41/e07a")}… differs from the original after compression; original kept`,
  `ERROR stopping after 1 verification failure (${FAILED_AT_FILES} of ${PREVIEW.files} files swapped)`,
];

export function mockHistory(): RunResult[] {
  return [
    {
      startedAt: "2026-11-21T12:12:00Z", durationMs: 41000, freedBytes: gib(1.0), errors: [],
      preview: { files: 4180, bytes: gib(2.1), skipped: 12, roots: [{ path: home(".rustup"), bytes: gib(1.4) }, { path: home(".npm"), bytes: gib(0.7) }] },
    },
    {
      startedAt: "2026-11-19T01:30:00Z", durationMs: 192000, freedBytes: gib(2.3), errors: [],
      preview: { files: 19523, bytes: gib(4.6), skipped: 88, roots: [{ path: home(".cargo/registry"), bytes: gib(2.6) }, { path: home(".rustup"), bytes: gib(2.0) }] },
    },
  ];
}
