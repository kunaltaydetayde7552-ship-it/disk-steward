// Shared test plumbing: minimal browser globals, locale switching and a fresh Pinia per test.
// Stores talk to the in-memory MockBackend (no Tauri), so events are driven with `emitInPage`.

import { createPinia, setActivePinia } from "pinia";
import { i18n, type Locale } from "../i18n";

type Globals = { window?: unknown; location?: unknown };

/** Stubs `window` and `location` (the ui store reads them) and returns the restore function. */
export function stubBrowser(search = ""): () => void {
  const g = globalThis as unknown as Globals;
  const saved = { window: g.window, location: g.location };
  g.window = { addEventListener: () => {} };
  g.location = { hash: "", search };
  return () => {
    g.window = saved.window;
    g.location = saved.location;
  };
}

/** Switches the active UI language and returns the restore function. */
export function useLocale(locale: Locale): () => void {
  const previous = i18n.global.locale.value;
  i18n.global.locale.value = locale;
  return () => {
    i18n.global.locale.value = previous;
  };
}

export function freshPinia(): void {
  setActivePinia(createPinia());
}

/** A promise settled by hand, to hold a backend call open while the test acts. */
export function deferred<T>() {
  let resolve!: (v: T) => void;
  let reject!: (e: unknown) => void;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

/** Lets pending microtasks (awaited spies, store reactions) run. */
export const flush = () => new Promise<void>((r) => setTimeout(r, 0));
