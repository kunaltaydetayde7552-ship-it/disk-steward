import { afterEach, describe, expect, test } from "bun:test";
import { errorCode, errorMessage, errorText } from "../errors";
import { i18n } from "../../i18n";
import { useLocale } from "../../__tests__/helpers";

// docs/API.md, "Codes"
const DOCUMENTED_CODES = ["engine_missing", "target_blocked", "clone_shared", "config_io", "run_in_progress", "run_failed", "schedule_failed", "not_found", "internal"];

describe("errorText", () => {
  let restore = () => {};
  afterEach(() => restore());

  for (const locale of ["pt-BR", "en"] as const) {
    test.each(DOCUMENTED_CODES)(`${locale}: %s maps to a translated message`, (code) => {
      restore = useLocale(locale);
      const text = errorText({ code, message: "english log line" });
      expect(text).not.toBe(`errors.${code}`);
      expect(text.length).toBeGreaterThan(10);
    });
  }

  test("every code has its own message", () => {
    const texts = DOCUMENTED_CODES.map((code) => errorText({ code, message: "" }));
    expect(new Set(texts).size).toBe(DOCUMENTED_CODES.length);
  });

  test("the message follows the active language", () => {
    const pt = errorText({ code: "not_found", message: "" });
    restore = useLocale("en");
    expect(errorText({ code: "not_found", message: "" })).not.toBe(pt);
  });

  test("the backend English message is never shown as copy", () => {
    expect(errorText({ code: "engine_missing", message: "No such file or directory (os error 2)" })).not.toContain("os error");
  });

  test.each([
    ["unknown code", { code: "brand_new", message: "x" }],
    ["null", null],
    ["undefined", undefined],
    ["string", "boom"],
    ["Error instance", new Error("boom")],
    ["empty object", {}],
  ])("falls back to the internal message for %s", (_name, input) => {
    expect(errorText(input)).toBe(i18n.global.t("errors.internal"));
  });
});

describe("errorCode", () => {
  test("returns documented codes as they are", () => {
    expect(DOCUMENTED_CODES.map((code) => errorCode({ code }))).toEqual(DOCUMENTED_CODES);
  });

  test("returns internal for anything else", () => {
    expect([errorCode({ code: 42 }), errorCode("run_failed"), errorCode(null)]).toEqual(["internal", "internal", "internal"]);
  });
});

describe("errorMessage", () => {
  test("prefers the ApiError message", () => {
    expect(errorMessage({ code: "run_failed", message: "cancelled" })).toBe("cancelled");
  });

  test("reads an Error message", () => {
    expect(errorMessage(new Error("boom"))).toBe("boom");
  });

  test("stringifies other values", () => {
    expect(errorMessage("plain")).toBe("plain");
  });
});
