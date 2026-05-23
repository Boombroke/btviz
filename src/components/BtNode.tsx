import type { Component } from "solid-js";
import type { BtNode as BtNodeT } from "@/stores/tree";

interface Props {
  node: BtNodeT;
}

/** Color tokens by node status. Animated transition lands in Day 2/3. */
const statusFill: Record<BtNodeT["status"], string> = {
  idle: "var(--color-status-idle)",
  running: "var(--color-status-running)",
  success: "var(--color-status-success)",
  failure: "var(--color-status-failure)",
  skipped: "var(--color-status-skipped)",
};

const BtNode: Component<Props> = (p) => {
  return (
    <g
      transform={`translate(${p.node.x}, ${p.node.y})`}
      class="transition-transform duration-150 ease-out"
      filter="url(#node-shadow)"
    >
      <rect
        width={p.node.w}
        height={p.node.h}
        rx={12}
        ry={12}
        fill="var(--color-surface-2)"
        stroke="rgba(255,255,255,0.08)"
        stroke-width="1"
      />
      {/* Top gradient sheen */}
      <rect
        width={p.node.w}
        height={p.node.h}
        rx={12}
        ry={12}
        fill="url(#node-grad)"
        pointer-events="none"
      />
      {/* Status indicator stripe */}
      <rect
        x={0}
        y={0}
        width={4}
        height={p.node.h}
        rx={2}
        ry={2}
        fill={statusFill[p.node.status]}
        style={{ transition: "fill 200ms cubic-bezier(0.22, 1, 0.36, 1)" }}
      />
      <text
        x={16}
        y={22}
        font-size="12"
        font-weight="600"
        fill="rgb(228, 228, 231)"
        font-family="var(--font-sans)"
      >
        {p.node.type}
      </text>
      <text
        x={16}
        y={40}
        font-size="10"
        fill="rgb(161, 161, 170)"
        font-family="var(--font-mono)"
      >
        #{p.node.id}
      </text>
    </g>
  );
};

export default BtNode;
