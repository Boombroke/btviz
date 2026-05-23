import type { Component } from "solid-js";
import { tree, type BtEdge as BtEdgeT } from "@/stores/tree";

interface Props {
  edge: BtEdgeT;
}

/** Cubic Bézier from parent's bottom-center to child's top-center. */
const BtEdge: Component<Props> = (p) => {
  const points = () => {
    const from = tree.nodes.find((n) => n.id === p.edge.from);
    const to = tree.nodes.find((n) => n.id === p.edge.to);
    if (!from || !to) return null;
    const x1 = from.x + from.w / 2;
    const y1 = from.y + from.h;
    const x2 = to.x + to.w / 2;
    const y2 = to.y;
    const dy = (y2 - y1) / 2;
    return `M ${x1} ${y1} C ${x1} ${y1 + dy}, ${x2} ${y2 - dy}, ${x2} ${y2}`;
  };

  return (
    <path
      d={points() ?? ""}
      stroke="var(--color-edge)"
      stroke-width="1.5"
      fill="none"
      stroke-linecap="round"
      opacity="0.6"
    />
  );
};

export default BtEdge;
