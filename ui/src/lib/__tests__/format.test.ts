import { afterEach, beforeEach, describe, expect, test } from "bun:test";
import { bytes, bytesParts, GiB, int, MiB, NBSP, percent } from "../format";
import { useLocale } from "../../__tests__/helpers";

describe("bytes in pt-BR", () => {
  let restore: () => void;
  beforeEach(() => (restore = useLocale("pt-BR")));
  afterEach(() => restore());

  test("formats GiB with a decimal comma and a no-break space", () => {
    expect(bytes(3.5 * GiB)).toBe(`3,5${NBSP}GiB`);
  });

  test("formats MiB without decimals", () => {
    expect(bytes(421 * MiB)).toBe(`421${NBSP}MiB`);
  });

  test("zero reads as 0 MiB", () => {
    expect(bytes(0)).toBe(`0${NBSP}MiB`);
  });

  test("under 1 MiB rounds to the nearest MiB", () => {
    expect([bytes(1), bytes(0.4 * MiB), bytes(0.6 * MiB)]).toEqual([`0${NBSP}MiB`, `0${NBSP}MiB`, `1${NBSP}MiB`]);
  });

  test("switches to GiB at 0.95 GiB", () => {
    expect([bytesParts(0.95 * GiB - 1)[1], bytesParts(0.95 * GiB)[1]]).toEqual(["MiB", "GiB"]);
  });

  test("very large values group thousands with a dot", () => {
    expect(bytes(5120 * GiB)).toBe(`5.120,0${NBSP}GiB`);
  });

  test("negative values use a real minus sign", () => {
    expect(bytes(-198 * MiB)).toBe(`−198${NBSP}MiB`);
  });

  test("the separator is U+00A0, never a plain space", () => {
    expect(bytes(2 * GiB)).not.toContain(" ");
  });

  test("int and percent follow the locale", () => {
    expect([int(23703), percent(0.714)]).toEqual(["23.703", "71%"]);
  });
});

describe("bytes in en", () => {
  let restore: () => void;
  beforeEach(() => (restore = useLocale("en")));
  afterEach(() => restore());

  test("formats GiB with a decimal point", () => {
    expect(bytes(3.5 * GiB)).toBe(`3.5${NBSP}GiB`);
  });

  test("zero reads as 0 MiB", () => {
    expect(bytes(0)).toBe(`0${NBSP}MiB`);
  });

  test("very large values group thousands with a comma", () => {
    expect(bytes(5120 * GiB)).toBe(`5,120.0${NBSP}GiB`);
  });

  test("negative GiB keeps the minus sign", () => {
    expect(bytes(-2 * GiB)).toBe(`−2.0${NBSP}GiB`);
  });
});
