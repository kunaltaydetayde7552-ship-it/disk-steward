// Isometric projection of the hero object, a glass block under an aluminum press, in a 520 x 600 viewBox.
// Heights are bytes mapped to units (max 232). Pure data: GlassBlock.vue turns shapes into SVG nodes.

export const BV = { w: 520, h: 600, cx: 260, cy: 424, a: 96, max: 232 } as const;
export const COS = 0.866;
export const SIN = 0.5;
/** the glass foot: thicker, tinted glass the core rests on */
export const FOOT = 10;

export type Pt = [number, number];
export type Attrs = Record<string, string | number | undefined>;
export interface Shape {
  tag: "polygon" | "path" | "ellipse" | "circle" | "rect" | "g" | "line";
  attrs: Attrs;
  children?: Shape[];
}

export interface BlockParams {
  mode: "solid" | "scan" | "wire";
  /** height of the glass (bytes on disk) */
  H: number;
  /** height of the ghost outline (logical bytes) */
  G: number;
  /** private core height and the hatched shared slab above it */
  priv: number;
  shared: number;
  /** platen bottom height; null = no press */
  pb: number | null;
  /** strength of the light pool on the floor, 0..1 */
  pool: number;
  light: "lit" | "off";
  pulse?: boolean;
  seam?: boolean;
  posts?: boolean;
  lock?: boolean;
  cold?: boolean;
  clones?: number;
}

/** (x, z) on the floor plane, y up, optionally offset by (ox, oz). */
export const P = (x: number, z: number, y: number, ox = 0, oz = 0): Pt => [BV.cx + (x + ox - z - oz) * COS, BV.cy + (x + ox + z + oz) * SIN - y];
export const xy = ([x, y]: Pt): string => `${x.toFixed(1)},${y.toFixed(1)}`;
export const pts = (list: Pt[]): string => list.map(xy).join(" ");
export const line = (...list: Pt[]): string => `M${list.map(xy).join("L")}`;

export function prism(a: number, y0: number, y1: number, ox = 0, oz = 0) {
  const p = (x: number, z: number, y: number) => P(x, z, y, ox, oz);
  return {
    top: pts([p(-a, -a, y1), p(a, -a, y1), p(a, a, y1), p(-a, a, y1)]),
    left: pts([p(-a, a, y0), p(a, a, y0), p(a, a, y1), p(-a, a, y1)]),
    right: pts([p(a, a, y0), p(a, -a, y0), p(a, -a, y1), p(a, a, y1)]),
    p,
  };
}

export const poly = (points: string, attrs: Attrs = {}): Shape => ({ tag: "polygon", attrs: { points, ...attrs } });
export const path = (d: string, attrs: Attrs = {}): Shape => ({ tag: "path", attrs: { d, fill: "none", ...attrs } });
export const group = (attrs: Attrs, children: Shape[]): Shape => ({ tag: "g", attrs, children });

/** The 24-grid lock glyph, reused on the platen badge. */
export const LOCK_GLYPH: Shape[] = [
  { tag: "rect", attrs: { x: 5, y: 10.5, width: 14, height: 9.5, rx: 2.5 } },
  path("M8.5 10.5V8a3.5 3.5 0 0 1 7 0v2.5"),
];
