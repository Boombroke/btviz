import type { Component } from "solid-js";
import { createMemo } from "solid-js";
import { type BtNode as BtNodeT, type NodeStatus, selection } from "@/stores/tree";

interface Props {
  node: BtNodeT;
}

const statusFill: Record<NodeStatus, string> = {
  idle: "#52525b",
  running: "#14b8a6",
  success: "#22c55e",
  failure: "#ef4444",
  skipped: "#a1a1aa",
};

const haloOpacity: Record<NodeStatus, number> = {
  idle: 0,
  running: 1,
  success: 0.45,
  failure: 0.45,
  skipped: 0,
};

const cardStroke: Record<NodeStatus, string> = {
  idle: "rgba(255,255,255,0.08)",
  running: "rgba(20, 184, 166, 0.55)",
  success: "rgba(34, 197, 94, 0.45)",
  failure: "rgba(239, 68, 68, 0.45)",
  skipped: "rgba(255,255,255,0.10)",
};

/**
 * Single BT node card.
 *
 * Status changes are animated with a 220ms cubic-out CSS transition on
 * `fill`, `opacity` and `stroke`. Animations remain smooth even when the BT
 * tick rate is irregular because we drive the values directly off the
 * reactive `node.status` instead of motion-one (whose SVG attr typings are
 * narrow and would force `as any` casts everywhere).
 */
const BtNode: Component<Props> = (p) => {
  const isSelected = createMemo(() => selection.id() === p.node.id);
  const onSelect = (e: MouseEvent) => {
    e.stopPropagation();
    selection.set(p.node.id);
  };

  // CSS easing reused on every animated SVG attribute below.
  const ease = "cubic-bezier(0.22, 1, 0.36, 1)";

  return (
    <g
      transform={`translate(${p.node.x}, ${p.node.y})`}
      class="cursor-pointer"
      filter="url(#node-shadow)"
      onClick={onSelect}
    >
      {/* Halo behind the card. Used as a "running" pulse and a faint glow on success/failure. */}
      <rect
        x={-4}
        y={-4}
        width={p.node.w + 8}
        height={p.node.h + 8}
        rx={16}
        ry={16}
        fill="rgba(20, 184, 166, 0.25)"
        opacity={haloOpacity[p.node.status]}
        style={{ transition: `opacity 300ms ${ease}` }}
        pointer-events="none"
      />

      <rect
        width={p.node.w}
        height={p.node.h}
        rx={12}
        ry={12}
        fill="var(--color-surface-2)"
        stroke={isSelected() ? "rgba(94, 234, 212, 0.7)" : cardStroke[p.node.status]}
        stroke-width={isSelected() ? 1.6 : 1}
        style={{ transition: `stroke 220ms ${ease}, stroke-width 150ms ease-out` }}
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
      {/* Status indicator stripe (left edge) */}
      <rect
        x={0}
        y={0}
        width={4}
        height={p.node.h}
        rx={2}
        ry={2}
        fill={statusFill[p.node.status]}
        style={{ transition: `fill 220ms ${ease}` }}
      />
      <text
        x={16}
        y={22}
        font-size="12"
        font-weight="600"
        fill="rgb(228, 228, 231)"
        font-family="var(--font-sans)"
      >
        {p.node.displayName ?? p.node.type}
      </text>
      <text
        x={16}
        y={40}
        font-size="10"
        fill="rgb(161, 161, 170)"
        font-family="var(--font-mono)"
      >
        {p.node.displayName ? `${p.node.type} · ` : ""}#{p.node.id}
      </text>
    </g>
  );
};

export default BtNode;
