// Numbers, sizes and times in the active UI language. Binary units (GiB/MiB), as Finder's "GB" means GiB here.
// Values are joined with a no-break space so "3,5 GiB" never wraps; negatives use a real minus sign.

import { currentLocale, t } from "../i18n";

export const GiB = 1024 ** 3;
export const MiB = 1024 ** 2;
export const NBSP = " ";
const MINUS = "−";

const cache = new Map<string, Intl.NumberFormat | Intl.DateTimeFormat | Intl.RelativeTimeFormat>();
function memo<T extends Intl.NumberFormat | Intl.DateTimeFormat | Intl.RelativeTimeFormat>(key: string, make: () => T): T {
  const k = `${currentLocale()}|${key}`;
  let f = cache.get(k);
  if (!f) {
    f = make();
    cache.set(k, f);
  }
  return f as T;
}
const nf1 = () => memo("nf1", () => new Intl.NumberFormat(currentLocale(), { minimumFractionDigits: 1, maximumFractionDigits: 1 }));
const nf0 = () => memo("nf0", () => new Intl.NumberFormat(currentLocale(), { maximumFractionDigits: 0 }));
const timeFmt = () => memo("time", () => new Intl.DateTimeFormat(currentLocale(), { hour: currentLocale() === "en" ? "numeric" : "2-digit", minute: "2-digit" }));
const dayFmt = () => memo("day", () => new Intl.DateTimeFormat(currentLocale(), { day: "numeric", month: "short" }));
const relFmt = () => memo("rel", () => new Intl.RelativeTimeFormat(currentLocale(), { numeric: "auto" }));

const sign = (n: number, s: string) => (n < 0 ? `${MINUS}${s}` : s);

/** [number, unit]: GiB with one decimal from 0.95 GiB, MiB without decimals below. */
export function bytesParts(b: number): [string, string] {
  const a = Math.abs(b);
  if (a >= GiB * 0.95) return [sign(b, nf1().format(a / GiB)), "GiB"];
  return [sign(b, nf0().format(Math.round(a / MiB))), "MiB"];
}
export const bytes = (b: number): string => bytesParts(b).join(NBSP);
export const int = (n: number): string => sign(n, nf0().format(Math.abs(n)));
export const percent = (x: number): string => `${nf0().format(Math.round(x * 100))}%`;
export const time = (iso: string): string => timeFmt().format(new Date(iso));

/** "21 nov" in pt-BR, "Nov 21" in en: no "de", no trailing period. */
export function day(iso: string | Date): string {
  const d = typeof iso === "string" ? new Date(iso) : iso;
  return dayFmt()
    .formatToParts(d)
    .map((p) => (p.type === "month" ? p.value.replace(".", "") : p.type === "literal" && p.value.trim() === "de" ? " " : p.value))
    .join("")
    .replace(/\s+/g, " ")
    .trim();
}
/** `YYYY-MM-DD` read as a local calendar day (no timezone shift). */
export function dayFromYmd(ymd: string): string {
  const [y, m, d] = ymd.split("-").map(Number);
  return day(new Date(y, m - 1, d, 12));
}
export function isToday(iso: string, now = new Date()): boolean {
  const d = new Date(iso);
  return d.getFullYear() === now.getFullYear() && d.getMonth() === now.getMonth() && d.getDate() === now.getDate();
}

export function duration(ms: number): string {
  const s = Math.max(0, Math.round(ms / 1000));
  return s < 60 ? t("units.seconds", { n: s }) : t("units.minutes", { m: Math.floor(s / 60), s: s % 60 });
}

/** "há 3 dias", "ontem", "3 days ago". */
export function age(iso: string, now = Date.now()): string {
  const sec = Math.round((new Date(iso).getTime() - now) / 1000);
  const a = Math.abs(sec);
  if (a < 3600) return relFmt().format(Math.round(sec / 60), "minute");
  if (a < 86400) return relFmt().format(Math.round(sec / 3600), "hour");
  return relFmt().format(Math.round(sec / 86400), "day");
}

/** Home folder shown as "~" (macOS and Linux home layouts). */
export const tilde = (p: string): string => p.replace(/^\/(?:Users|home)\/[^/]+(?=\/|$)/, "~");

/** Last path segment, for "comprimindo serde.rs". */
export const basename = (p: string): string => p.split("/").filter(Boolean).pop() ?? p;
