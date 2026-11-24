// vue-i18n (Composition API). pt-BR is the default and the source of truth; en is typed against it.
import { createI18n } from "vue-i18n";
import en from "./locales/en";
import ptBR, { type MessageSchema } from "./locales/pt-BR";

declare module "vue-i18n" {
  // eslint-disable-next-line @typescript-eslint/no-empty-object-type
  export interface DefineLocaleMessage extends MessageSchema {}
}

export const LOCALES = ["pt-BR", "en"] as const;
export type Locale = (typeof LOCALES)[number];

const KEY = "steward.locale";

function saved(): string | null {
  try {
    return localStorage.getItem(KEY);
  } catch {
    return null;
  }
}

/** ?lang= (screenshots and testing) first, then the saved choice, else pt-BR. */
export function initialLocale(): Locale {
  const query = typeof location === "undefined" ? null : new URLSearchParams(location.search).get("lang");
  const pick = [query, saved()].find((l) => LOCALES.includes(l as Locale));
  return (pick as Locale | undefined) ?? "pt-BR";
}

export const i18n = createI18n({
  legacy: false,
  locale: initialLocale(),
  fallbackLocale: "pt-BR",
  messages: { "pt-BR": ptBR, en },
});

export const t = i18n.global.t;
export const currentLocale = (): Locale => i18n.global.locale.value as Locale;

export function setLocale(l: Locale, persist = true): void {
  i18n.global.locale.value = l;
  document.documentElement.lang = l;
  if (!persist) return;
  try {
    localStorage.setItem(KEY, l);
  } catch {
    // private mode or storage off: the choice lasts for this session
  }
}
