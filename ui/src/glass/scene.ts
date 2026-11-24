// Composes the layers of one block from its parameters.
import { BV, type BlockParams, type Shape } from "./geometry";
import { backLayer, clonesLayer, coreLayer, floorLayer, footLayer, ghostLayer, glassLayer, glowLayer, scanLayer } from "./layers";
import { platenLayer } from "./platen";

export function blockShapes(id: string, p: BlockParams): Shape[] {
  const u = (n: string) => `url(#${id}-${n})`;
  const H = Math.max(p.H, 3);
  const G = Math.max(p.G, H);
  const core = p.mode === "solid" ? coreLayer(u, p, H) : { shapes: [], tops: { privTop: 10, shTop: 10 } };
  return [
    ...floorLayer(u, p),
    ...backLayer(H),
    ...core.shapes,
    ...footLayer(p),
    ...(p.mode === "scan" ? scanLayer(H) : []),
    ...glowLayer(u, p, core.tops),
    ...glassLayer(u, p, H, core.tops),
    ...ghostLayer(H, G),
    ...clonesLayer(u, p, core.tops),
    ...platenLayer(u, p, H),
  ];
}

export const UNIT = BV.max;
