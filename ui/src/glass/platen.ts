// The press: a bead-blasted aluminum platen with a chamfered top, brushed grain drawn in its own
// isometric plane, polished guide posts, the block's light caught on its lower edge, and the lock badge.

import { BV, COS, LOCK_GLYPH, P, SIN, group, line, path, poly, prism, type BlockParams, type Shape } from "./geometry";

const AP = 108; // half width of the platen
const T = 18; // thickness

/** A polished vertical cylinder from the floor to `top`, at floor offset (ox, oz). */
function post(u: (n: string) => string, ox: number, oz: number, top: number): Shape[] {
  const r = 7;
  const [cx, by] = P(0, 0, 0, ox, oz);
  const ty = by - top;
  const ry = r * SIN;
  return [
    { tag: "ellipse", attrs: { cx: (cx + 6).toFixed(1), cy: (by + 2).toFixed(1), rx: r + 4, ry: ry + 2, fill: "var(--floor-shadow)", opacity: 0.45, filter: u("b2") } },
    path(`M${(cx - r).toFixed(1)},${ty.toFixed(1)}L${(cx - r).toFixed(1)},${by.toFixed(1)}A${r} ${ry} 0 0 0 ${(cx + r).toFixed(1)},${by.toFixed(1)}L${(cx + r).toFixed(1)},${ty.toFixed(1)}Z`, { fill: u("chrome") }),
    { tag: "ellipse", attrs: { cx: cx.toFixed(1), cy: ty.toFixed(1), rx: r, ry, fill: "var(--chrome-light)", stroke: "var(--chrome-mid)", "stroke-width": 0.6 } },
  ];
}

export function platenLayer(u: (n: string) => string, p: BlockParams, H: number): Shape[] {
  if (p.pb == null) return [];
  const pb = Math.max(p.pb, H);
  const gap = pb - H;
  const out: Shape[] = [];
  // the platen's shadow on the glass top deepens as it comes down
  if (gap > 1) out.push(poly(prism(BV.a - 4, H, H).top, { fill: "var(--floor-shadow)", opacity: Math.max(0.08, 0.42 - gap / 160).toFixed(2), filter: u("b5") }));
  if (p.posts) for (const [ox, oz] of [[-AP - 4, AP + 4], [AP + 4, -AP - 4]] as const) out.push(...post(u, ox, oz, pb + T - 4));

  const k = prism(AP, pb, pb + T);
  out.push(poly(k.left, { fill: u("al") }), poly(k.right, { fill: u("ar") }));
  // light from a lit block caught on the platen's lower edge, strongest when it is close
  const catchLight = p.light === "lit" ? Math.max(0, 0.55 - gap / 110) : 0;
  if (catchLight > 0.02) {
    const band = prism(AP, pb, pb + 5);
    out.push(poly(band.left, { style: "fill:var(--hl)", opacity: catchLight.toFixed(2) }), poly(band.right, { style: "fill:var(--hl)", opacity: (catchLight * 0.8).toFixed(2) }));
  }
  // top face drawn as a square in its own plane, so the brushed grain runs along the platen
  const top = pb + T;
  out.push(group({ transform: `matrix(${COS} ${SIN} ${-COS} ${SIN} ${BV.cx} ${BV.cy - top})` }, [
    { tag: "rect", attrs: { x: -AP, y: -AP, width: AP * 2, height: AP * 2, fill: u("at"), filter: u("brush") } },
  ]));
  // chamfer: a crisp outer edge, a bright bevel just inside it
  out.push(poly(k.top, { fill: "none", stroke: "var(--glass-bevel-shade)", "stroke-width": 1 }));
  out.push(poly(prism(AP - 2.5, 0, top).top, { fill: "none", stroke: "white", "stroke-opacity": 0.85, "stroke-width": 1.2, "stroke-linejoin": "round" }));
  out.push(path(line(k.p(-AP, AP, pb), k.p(AP, AP, pb), k.p(AP, -AP, pb)), { stroke: "var(--glass-contact)", "stroke-width": 1.2 }));
  out.push(path(line(k.p(AP, AP, pb), k.p(AP, AP, top)), { stroke: "white", "stroke-opacity": 0.75, "stroke-width": 1.2 }));
  out.push(path(line(k.p(-AP, AP, top - 1), k.p(AP, AP, top - 1), k.p(AP, -AP, top - 1)), { stroke: "white", "stroke-opacity": 0.35, "stroke-width": 1 }));
  if (p.lock) {
    const [lx, ly] = k.p(0, AP, pb + T / 2);
    out.push(group({ transform: `translate(${(lx - 28).toFixed(1)} ${(ly - 28).toFixed(1)})` }, [
      { tag: "circle", attrs: { cx: 28, cy: 31, r: 27, fill: "var(--floor-shadow)", opacity: 0.35, filter: u("b2") } },
      { tag: "circle", attrs: { cx: 28, cy: 28, r: 27, fill: "var(--color-frost)", stroke: "var(--color-quartz)", "stroke-width": 2 } },
      group({ transform: "translate(10 10) scale(1.5)", fill: "none", stroke: "var(--color-quartz-deep)", "stroke-width": 1.6, "stroke-linecap": "round", "stroke-linejoin": "round" }, LOCK_GLYPH),
    ]));
  }
  return out;
}
