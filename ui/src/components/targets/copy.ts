// What the detail panel says about one folder: the big number, the line under it and, for a folder that
// stays out, why and how to change that. Kept apart from the template so every case reads in one place.

import type { Target } from "../../api/types";
import { t } from "../../i18n";
import { bytes, bytesParts } from "../../lib/format";
import { cloneCount, isMeasured, isPseudo, savedBytes, sharedBytes } from "../../lib/targets";

export interface NumeralCopy {
  value: string;
  unit: string;
  /** a word instead of a number: smaller type */
  word: boolean;
  sub: string;
}

export function numeralCopy(x: Target, neverMeasured: boolean): NumeralCopy {
  if (isPseudo(x)) return { value: t("targets.numeral.notMeasured"), unit: "", word: true, sub: t("targets.numeral.rewrittenSub") };
  if (neverMeasured || !isMeasured(x)) return { value: t("targets.numeral.never"), unit: "", word: true, sub: t("targets.numeral.neverSub") };
  const disk = bytes(x.onDiskBytes);
  const priv = x.privateBytes == null ? t("targets.sizes.noData") : bytes(x.privateBytes);
  if (x.risk === "clone_shared") {
    const [n, u] = bytesParts(sharedBytes(x));
    const c = cloneCount(x);
    return { value: n, unit: t("targets.numeral.clonesUnit", { unit: u }), word: false, sub: c ? t("targets.numeral.clonesSub", { n: c, disk, private: priv }) : t("targets.numeral.clonesSubUnknown", { disk, private: priv }) };
  }
  if (!x.allowed) {
    const [n, u] = bytesParts(sharedBytes(x));
    return { value: n, unit: t("targets.numeral.sharedUnit", { unit: u }), word: false, sub: t("targets.numeral.sharedSub", { disk, private: priv }) };
  }
  const [n, u] = bytesParts(x.onDiskBytes);
  if (x.enabled && x.compressedFiles > 0) {
    return { value: n, unit: t("targets.numeral.diskUnit", { unit: u }), word: false, sub: t("targets.numeral.compressedSub", { logical: bytes(x.logicalBytes), saved: bytes(savedBytes(x)) }) };
  }
  if (x.enabled) return { value: n, unit: t("targets.numeral.diskUnit", { unit: u }), word: false, sub: t("targets.numeral.enabledSub") };
  let sub = t("targets.numeral.offSub");
  if (x.id === "mise") sub = t("targets.numeral.mise");
  else if (x.id === "pnpm-store" && x.privateBytes != null) sub = t("targets.numeral.pnpm", { private: priv });
  return { value: n, unit: t("targets.numeral.offUnit", { unit: u }), word: false, sub };
}

export interface RiskCopy {
  title: string;
  body: string;
  how: string;
}

export function riskCopy(x: Target): RiskCopy | null {
  if (x.allowed) return null;
  if (isPseudo(x)) return { title: t("targets.risk.rewrittenTitle"), body: t("targets.risk.rewrittenBody"), how: t("targets.risk.rewrittenHow") };
  if (x.risk === "clone_shared") {
    const c = cloneCount(x);
    const shared = bytes(sharedBytes(x));
    return {
      title: t("targets.risk.clonesTitle"),
      body: c ? t("targets.risk.clonesBody", { n: c, shared }) : t("targets.risk.clonesBodyUnknown", { shared }),
      how: t("targets.risk.clonesHow"),
    };
  }
  const known = x.privateBytes != null && sharedBytes(x) > 0;
  return {
    title: t("targets.risk.ruleTitle"),
    body: known ? t("targets.risk.ruleBodyBun", { disk: bytes(x.onDiskBytes), private: bytes(x.privateBytes ?? 0) }) : t("targets.risk.ruleBody"),
    how: t("targets.risk.ruleHow"),
  };
}

/** Note of the "Compartilhado" size row. */
export function sharedNote(x: Target): string {
  if (x.risk === "clone_shared") {
    const c = cloneCount(x);
    return c ? t("targets.sizes.sharedClones", { n: c }) : t("targets.sizes.sharedClonesUnknown");
  }
  if (x.risk === "blocked_by_rule") return t("targets.sizes.sharedBun");
  return t("targets.sizes.sharedSkipped");
}
