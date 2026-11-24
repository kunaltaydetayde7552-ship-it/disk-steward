import { afterEach, describe, expect, test } from "bun:test";
import { cloneCount, kindLabel, savedBytes, savedRatio, sharedBytes, targetLight, targetName, targetShort } from "../targets";
import { mockTargets } from "../../api/mockData";
import { useLocale } from "../../__tests__/helpers";
import type { Target } from "../../api/types";

const byId = (id: string): Target => mockTargets().find((x) => x.id === id)!;
const custom = { id: "custom-1", path: "/Users/dev/work/big-cache" };

describe("target names", () => {
  let restore = () => {};
  afterEach(() => restore());

  test("a known target uses the translated name (pt-BR)", () => {
    restore = useLocale("pt-BR");
    expect(targetName({ id: "rustup", path: "/x" })).toBe("Toolchains do Rust");
  });

  test("a known target follows the language (en)", () => {
    restore = useLocale("en");
    expect(targetName({ id: "rustup", path: "/x" })).toBe("Rust toolchains");
  });

  test("a custom target shows its folder name", () => {
    expect(targetName(custom)).toBe("big-cache");
  });

  test("short name is the tool people type", () => {
    expect(targetShort({ id: "cargo-registry", path: "/x" })).toBe("cargo");
  });

  test("a custom target short name is its folder name too", () => {
    expect(targetShort(custom)).toBe("big-cache");
  });

  test("a trailing slash does not produce an empty name", () => {
    expect(targetName({ id: "custom-2", path: "/Users/dev/work/cache/" })).toBe("cache");
  });
});

describe("kindLabel", () => {
  test.each([
    ["toolchain", "toolchain"],
    ["package_cache", "package cache"],
    ["store", "store"],
    ["build", "build"],
    ["custom", "custom folder"],
  ] as const)("%s reads as %s in en", (kind, label) => {
    const restore = useLocale("en");
    try {
      expect(kindLabel({ kind })).toBe(label);
    } finally {
      restore();
    }
  });
});

describe("derived numbers", () => {
  test("savedBytes and savedRatio come from logical vs on disk", () => {
    const x = { ...byId("rustup"), logicalBytes: 1000, onDiskBytes: 400 };
    expect([savedBytes(x), savedRatio(x)]).toEqual([600, 0.6]);
  });

  test("never negative or divided by zero", () => {
    const x = { ...byId("rustup"), logicalBytes: 0, onDiskBytes: 50 };
    expect([savedBytes(x), savedRatio(x)]).toEqual([0, 0]);
  });

  test("sharedBytes is 0 when the private size is unknown", () => {
    expect(sharedBytes({ ...byId("rustup"), privateBytes: null })).toBe(0);
  });

  test("sharedBytes is on disk minus private", () => {
    expect(sharedBytes({ ...byId("rustup"), onDiskBytes: 100, privateBytes: 30 })).toBe(70);
  });

  test("cloneCount reads the venv count from the backend detail", () => {
    expect(cloneCount(byId("uv-cache"))).toBe(23);
  });

  test("cloneCount is null without detail", () => {
    expect(cloneCount(byId("rustup"))).toBeNull();
  });
});

describe("targetLight", () => {
  test("pseudo target is a wireframe", () => expect(targetLight(byId("rust-target"), null)).toBe("wire"));
  test("blocked target is cold", () => expect(targetLight(byId("uv-cache"), null)).toBe("cold"));
  test("disabled target is unlit", () => expect(targetLight(byId("mise"), null)).toBe("unlit"));
  test("enabled target is green", () => expect(targetLight(byId("rustup"), null)).toBe("green"));
  test("enabled target holding the failed path is red", () => {
    const x = byId("npm");
    expect(targetLight(x, `${x.path}/_cacache/x`)).toBe("red");
  });
});
