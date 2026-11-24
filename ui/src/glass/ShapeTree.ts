// Renders geometry shapes as SVG nodes (no innerHTML: the attributes are numbers and token names).
import { h, type FunctionalComponent, type VNode } from "vue";
import type { Shape } from "./geometry";

const node = (s: Shape, key: number): VNode => h(s.tag, { key, ...s.attrs }, s.children?.map(node));

const ShapeTree: FunctionalComponent<{ shapes: Shape[] }> = (props) => props.shapes.map(node);
ShapeTree.props = ["shapes"];
export default ShapeTree;
