// The glass block, back to front: floor (pool, contact shadows, caustic), back edges, foot, core, scan,
// glass skin (fresnel faces, specular streaks, bevels, glint), ghost outline and clone cubes.
// Material notes: glass reflects more at grazing angles (fresnel bands on every face), thick glass shows its
// iron-oxide tint in the foot and the edges, a lit core bleeds its color into the glass around it, and the
// thick front edge displaces what sits behind it (the refraction band).

import { BV, FOOT, P, group, line, path, poly, prism, pts, type BlockParams, type Pt, type Shape } from "./geometry";

const HL_SOFT = "fill:color-mix(in oklab, var(--hl) 28%, white)";

export function floorLayer(u: (n: string) => string, p: BlockParams): Shape[] {
  const a = BV.a;
  const out: Shape[] = [];
  if (p.pool > 0) out.push({ tag: "ellipse", attrs: { cx: BV.cx, cy: BV.cy + 8, rx: 252, ry: 140, fill: u("pool"), opacity: p.pool.toFixed(2) } });
  // three shadows: the wide penumbra of a heavy object, a mid falloff, and the dark line where glass meets floor
  out.push(poly(prism(a + 14, 0, 0, 22, 10).top, { fill: "var(--floor-shadow)", filter: u("b18"), opacity: 0.42 }));
  out.push(poly(prism(a + 4, 0, 0, 7, 3).top, { fill: "var(--floor-shadow)", filter: u("b5"), opacity: 0.36 }));
  out.push(poly(prism(a + 1, 0, 0).top, { fill: "var(--floor-shadow)", filter: u("b2"), opacity: 0.5 }));
  if (p.mode !== "wire") {
    // caustic: the block focuses light into a bright fringe just outside its front edges
    const lit = p.light !== "off" && p.mode === "solid" && p.priv > 8;
    const fringe = line(P(-a + 10, a + 13, 0), P(a + 13, a + 13, 0), P(a + 13, -a + 10, 0));
    out.push(path(fringe, { style: `stroke:${lit ? "color-mix(in oklab, var(--hl) 55%, white)" : "white"}`, "stroke-width": 3.5, "stroke-linejoin": "round", filter: u("b2"), opacity: lit ? 0.7 : 0.4 }));
    if (lit) {
      const [cx, cy] = P(a + 44, 24, 0);
      out.push({ tag: "ellipse", attrs: { cx: cx.toFixed(1), cy: cy.toFixed(1), rx: 62, ry: 20, style: "fill:var(--hl)", opacity: 0.42, filter: u("b12") } });
    }
  }
  return out;
}

export function backLayer(H: number): Shape[] {
  const a = BV.a;
  const d = `${line(P(-a, -a, 0), P(-a, -a, H))}${line(P(-a, -a, 0), P(a, -a, 0))}${line(P(-a, -a, 0), P(-a, a, 0))}`;
  return [path(d, { stroke: "var(--glass-back)", "stroke-width": 1.2 })];
}

/** The foot: a slab of thicker glass whose tint shows in the edge, like a glass paperweight's base. */
export function footLayer(p: BlockParams): Shape[] {
  if (p.mode === "wire") return [];
  const f = prism(BV.a, 0, FOOT);
  const a = BV.a;
  return [
    poly(f.left, { style: "fill:var(--glass-body)", opacity: 0.5 }),
    poly(f.right, { style: "fill:var(--glass-body)", opacity: 0.75 }),
    path(line(P(-a, a, FOOT), P(a, a, FOOT), P(a, -a, FOOT)), { stroke: "white", "stroke-opacity": 0.55, "stroke-width": 0.9 }),
  ];
}

export interface CoreTops {
  privTop: number;
  shTop: number;
}

export function coreLayer(u: (n: string) => string, p: BlockParams, H: number): { shapes: Shape[]; tops: CoreTops } {
  const ac = BV.a - 9;
  const y0 = FOOT;
  const out: Shape[] = [];
  const privTop = Math.min(y0 + p.priv, H - 4);
  const shTop = Math.min(privTop + p.shared, H - 4);
  if (privTop > y0 + 1) {
    const c = prism(ac, y0, privTop);
    if (p.light === "off") {
      const f = p.cold
        ? ["color-mix(in oklab, var(--color-quartz) 18%, var(--core-unlit-top))", "color-mix(in oklab, var(--color-quartz) 30%, var(--core-unlit-left))", "color-mix(in oklab, var(--color-quartz) 45%, var(--core-unlit-right))"]
        : ["var(--core-unlit-top)", "var(--core-unlit-left)", "var(--core-unlit-right)"];
      out.push(poly(c.left, { style: `fill:${f[1]}` }), poly(c.right, { style: `fill:${f[2]}` }), poly(c.top, { style: `fill:${f[0]}` }));
      out.push(poly(c.top, { fill: "none", stroke: "white", "stroke-opacity": 0.6, "stroke-width": 1 }));
    } else {
      // the halo: the core's own light diffused by the glass around it
      out.push(group({ class: p.pulse ? "pulse" : undefined, opacity: 0.6 }, [
        poly(c.left, { style: "fill:var(--hl)", filter: u("b12") }),
        poly(c.right, { style: "fill:var(--hl)", filter: u("b12") }),
        poly(c.top, { style: "fill:var(--hl)", filter: u("b12") }),
      ]));
      out.push(poly(c.left, { fill: u("cl") }), poly(c.right, { fill: u("cr") }), poly(c.top, { style: HL_SOFT }));
      // a soft vertical sheen down the core's left face and a crisp front edge
      out.push(poly(pts([c.p(-ac + 14, ac, y0 + 4), c.p(-ac + 34, ac, y0 + 4), c.p(-ac + 34, ac, privTop - 4), c.p(-ac + 14, ac, privTop - 4)]), { fill: "white", opacity: 0.22, filter: u("b5") }));
      out.push(path(line(c.p(ac, ac, y0), c.p(ac, ac, privTop)), { stroke: "white", "stroke-opacity": 0.55, "stroke-width": 1 }));
      out.push(poly(c.top, { fill: "none", stroke: "white", "stroke-opacity": 0.85, "stroke-width": 1.2 }));
    }
  }
  if (shTop > privTop + 1) {
    const s = prism(ac, privTop, shTop);
    out.push(
      poly(s.left, { style: "fill:color-mix(in oklab, var(--color-quartz) 22%, var(--core-unlit-left))", opacity: 0.9 }),
      poly(s.right, { style: "fill:color-mix(in oklab, var(--color-quartz) 35%, var(--core-unlit-right))", opacity: 0.9 }),
      poly(s.top, { style: "fill:var(--core-unlit-top)", opacity: 0.9 }),
    );
    const lines: string[] = [];
    for (let y = privTop + 6; y < shTop - 2; y += 7) lines.push(line(s.p(-ac, ac, y), s.p(ac, ac, y), s.p(ac, -ac, y)));
    out.push(path(lines.join(""), { stroke: "var(--color-quartz-deep)", "stroke-opacity": 0.28, "stroke-width": 0.9 }));
  }
  return { shapes: out, tops: { privTop, shTop } };
}

/** Light that crosses the glass: the inner glow through the top and the tint the lit core gives the faces. */
export function glowLayer(u: (n: string) => string, p: BlockParams, tops: CoreTops): Shape[] {
  if (p.mode !== "solid" || p.light === "off" || tops.privTop < FOOT + 8) return [];
  const a = BV.a;
  const g = prism(a, FOOT, tops.privTop);
  const [gx, gy] = P(0, 0, tops.privTop);
  return [
    poly(g.left, { style: "fill:var(--hl)", opacity: 0.1 }),
    poly(g.right, { style: "fill:var(--hl)", opacity: 0.14 }),
    { tag: "ellipse", attrs: { cx: gx.toFixed(1), cy: gy.toFixed(1), rx: 96, ry: 52, fill: u("glow"), class: p.pulse ? "pulse" : undefined } },
  ];
}

export function scanLayer(H: number): Shape[] {
  const k = prism(BV.a - 4, FOOT, FOOT);
  return [group({ class: "scan", style: `--scan-to:-${Math.max(0, H - FOOT - 12).toFixed(0)}px` }, [
    poly(k.top, { style: "fill:var(--hl)", opacity: 0.3 }),
    poly(k.top, { fill: "none", style: "stroke:var(--hl)", "stroke-width": 1.6 }),
  ])];
}

export function glassLayer(u: (n: string) => string, p: BlockParams, H: number, tops: CoreTops): Shape[] {
  const a = BV.a;
  const g = prism(a, 0, H);
  const out: Shape[] = [];
  const wire = p.mode === "wire";
  const faint = p.mode === "scan" || wire ? 0.45 : 1;
  if (!wire) {
    out.push(poly(g.left, { fill: u("gl"), opacity: faint }), poly(g.right, { fill: u("gr"), opacity: faint }), poly(g.top, { fill: u("gt"), opacity: faint }));
    // fresnel: each face brightens toward its edges
    out.push(poly(g.left, { fill: u("fl"), opacity: faint }), poly(g.right, { fill: u("fr"), opacity: faint }), poly(g.top, { fill: u("ft"), opacity: faint }));
    if (H > 40) {
      // refraction band: the core's front edge seen displaced through the thick front corner
      if (p.mode === "solid" && tops.privTop > FOOT + 12) {
        const tone = p.light === "off" ? "white" : "color-mix(in oklab, var(--hl) 45%, white)";
        out.push(path(line(g.p(a - 6, a - 2, FOOT + 4), g.p(a - 6, a - 2, tops.privTop - 2)), { style: `stroke:${tone}`, "stroke-width": 4, opacity: 0.55, filter: u("b2") }));
      }
      // specular: a wide soft streak and a sharp core streak across the left face, a sheen down the right face
      const wide: Pt[] = [g.p(-a + 18, a, 12), g.p(-a + 52, a, 12), g.p(-a + 34, a, H - 12), g.p(-a + 4, a, H - 12)];
      out.push(poly(pts(wide), { fill: "white", opacity: 0.4, filter: u("b5") }));
      const sharp: Pt[] = [g.p(-a + 34, a, 16), g.p(-a + 41, a, 16), g.p(-a + 26, a, H - 16), g.p(-a + 20, a, H - 16)];
      out.push(poly(pts(sharp), { fill: "white", opacity: 0.75, filter: u("b1") }));
      const sheen: Pt[] = [g.p(a, a - 20, 10), g.p(a, a - 44, 10), g.p(a, a - 44, H - 10), g.p(a, a - 20, H - 10)];
      out.push(poly(pts(sheen), { fill: "white", opacity: 0.2, filter: u("b5") }));
      // gloss: the window light on the back half of the top face
      out.push(poly(pts([g.p(-a + 10, -a + 10, H), g.p(a * 0.2, -a + 10, H), g.p(-a + 10, a * 0.2, H)]), { fill: "white", opacity: 0.55, filter: u("b12") }));
      // top bevel: a highlight ring and its shaded inner lip
      out.push(poly(prism(a - 7, 0, H).top, { fill: "none", stroke: "white", "stroke-opacity": 0.5, "stroke-width": 1 }));
      out.push(poly(prism(a - 3, 0, H).top, { fill: "none", stroke: "var(--glass-bevel-shade)", "stroke-width": 0.8 }));
      // the thick front corner catches more light than any other edge
      out.push(path(line(g.p(a - 3, a - 3, 4), g.p(a - 3, a - 3, H - 4)), { stroke: "white", "stroke-opacity": 0.5, "stroke-width": 6, filter: u("b2") }));
    }
  }
  const dash = wire ? "5 6" : undefined;
  const edge = wire ? "var(--color-anodized-muted)" : "var(--glass-edge)";
  // vertical edges: soft halo plus crisp line, and the dark side of the front bevel
  if (!wire) {
    for (const [x, z] of [[-a, a], [a, -a]] as const) out.push(path(line(g.p(x, z, 0), g.p(x, z, H)), { stroke: "white", "stroke-opacity": 0.3, "stroke-width": 5, filter: u("b2") }));
    out.push(path(line(g.p(a - 4, a, 2), g.p(a - 4, a, H - 2)), { stroke: "var(--glass-bevel-shade)", "stroke-width": 1 }));
  }
  out.push(path(`${line(g.p(-a, a, 0), g.p(-a, a, H))}${line(g.p(a, -a, 0), g.p(a, -a, H))}`, { stroke: edge, "stroke-opacity": 0.75, "stroke-width": 1.2, "stroke-dasharray": dash }));
  out.push(path(line(g.p(a, a, 0), g.p(a, a, H)), { stroke: edge, "stroke-width": 1.8, "stroke-dasharray": dash }));
  out.push(poly(g.top, { fill: "none", stroke: edge, "stroke-width": 1.4, "stroke-linejoin": "round", "stroke-dasharray": dash }));
  out.push(path(line(g.p(-a, a, 0), g.p(a, a, 0), g.p(a, -a, 0)), { stroke: wire ? edge : "var(--glass-contact)", "stroke-width": 1.2, "stroke-dasharray": dash }));
  if (!wire && H > 40) {
    // glint: the point light caught on the top front corner
    const [gx, gy] = g.p(a, a, H);
    out.push({ tag: "circle", attrs: { cx: gx.toFixed(1), cy: gy.toFixed(1), r: 5, fill: "white", opacity: 0.9, filter: u("b2") } });
    out.push(path(`M${(gx - 11).toFixed(1)},${gy.toFixed(1)}h22M${gx.toFixed(1)},${(gy - 9).toFixed(1)}v18`, { stroke: "white", "stroke-opacity": 0.85, "stroke-width": 0.8, "stroke-linecap": "round" }));
  }
  if (p.seam) {
    const top = prism(a + 1, H, H).top;
    out.push(poly(top, { fill: "none", stroke: "var(--color-signal-red)", "stroke-width": 5, opacity: 0.55, filter: u("b5") }));
    out.push(poly(top, { fill: "none", stroke: "var(--color-signal-red)", "stroke-width": 1.8 }));
  }
  return out;
}

export function ghostLayer(H: number, G: number): Shape[] {
  if (G <= H + 3) return [];
  const a = BV.a;
  const g = prism(a, H, G);
  const d = ([[-a, a], [a, a], [a, -a]] as const).map(([x, z]) => line(g.p(x, z, H), g.p(x, z, G))).join("");
  return [group({ fill: "none", stroke: "var(--color-anodized)", "stroke-opacity": 0.38, "stroke-width": 1.2, "stroke-dasharray": "5 6" }, [
    poly(g.left, { fill: "var(--glass-left-b)", stroke: "none", opacity: 0.5 }),
    poly(g.right, { fill: "var(--glass-right-b)", stroke: "none", opacity: 0.5 }),
    path(d),
    poly(g.top),
    path(line(g.p(-a, -a, H), g.p(-a, -a, G)), { "stroke-opacity": 0.16 }),
  ])];
}

/** Small cubes beside the block, each tied by a thread to the shared slab: the clones elsewhere on disk. */
export function clonesLayer(u: (n: string) => string, p: BlockParams, tops: CoreTops): Shape[] {
  if (!p.clones) return [];
  const a = BV.a;
  const cols = [a + 40, a + 62, a + 84, a + 106];
  const rows = [-74, -52, -30, -8, 14, 36];
  const threads: string[] = [];
  const cubes: { ox: number; oz: number; depth: number }[] = [];
  for (let i = 0; i < p.clones; i++) {
    const ox = cols[i % 4];
    const oz = rows[Math.floor(i / 4) % 6];
    const zs = -a + 14 + ((i * 5) % 12) * ((2 * a - 28) / 11);
    const ys = tops.privTop + (tops.shTop - tops.privTop) * (0.15 + ((i * 7) % 10) / 13);
    const [sx, sy] = P(a, zs, ys);
    const [ex, ey] = P(0, 0, 13, ox, oz);
    threads.push(`M${sx.toFixed(1)},${sy.toFixed(1)}Q${((sx + ex) / 2).toFixed(1)},${(Math.min(sy, ey) - 26).toFixed(1)} ${ex.toFixed(1)},${ey.toFixed(1)}`);
    cubes.push({ ox, oz, depth: ox + oz });
  }
  cubes.sort((m, n) => m.depth - n.depth);
  const out: Shape[] = [path(threads.join(""), { stroke: "var(--color-quartz-deep)", "stroke-opacity": 0.32, "stroke-width": 0.9 })];
  for (const { ox, oz } of cubes) {
    const k = prism(7, 0, 12, ox, oz);
    out.push(
      poly(prism(8, 0, 0, ox + 3, oz + 1).top, { fill: "var(--floor-shadow)", opacity: 0.35, filter: u("b2") }),
      poly(k.left, { style: "fill:color-mix(in oklab, var(--color-quartz) 30%, var(--core-unlit-left))" }),
      poly(k.right, { style: "fill:color-mix(in oklab, var(--color-quartz) 45%, var(--core-unlit-right))" }),
      poly(k.top, { style: "fill:var(--core-unlit-top)", stroke: "var(--glass-edge)", "stroke-width": 0.8 }),
    );
  }
  return out;
}
