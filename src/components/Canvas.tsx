import type { Component } from "solid-js";
import { For, Show, createMemo } from "solid-js";
import { tree, selection } from "@/stores/tree";
import BtNode from "./BtNode";
import BtEdge from "./BtEdge";

const PADDING = 48;

const Canvas: Component = () => {
  /** Compute a viewBox that fits the whole tree with padding. */
  const viewBox = createMemo(() => {
    const ns = tree.nodes;
    if (ns.length === 0) return "0 0 100 100";
    let minX = Infinity,
      minY = Infinity,
      maxX = -Infinity,
      maxY = -Infinity;
    for (const n of ns) {
      if (n.x < minX) minX = n.x;
      if (n.y < minY) minY = n.y;
      if (n.x + n.w > maxX) maxX = n.x + n.w;
      if (n.y + n.h > maxY) maxY = n.y + n.h;
    }
    const x = minX - PADDING;
    const y = minY - PADDING;
    const w = maxX - minX + PADDING * 2;
    const h = maxY - minY + PADDING * 2;
    return `${x} ${y} ${w} ${h}`;
  });

  return (
    <section class="relative flex-1 overflow-hidden">
      <div class="dot-grid absolute inset-0" />

      <svg
        class="absolute inset-0 h-full w-full"
        preserveAspectRatio="xMidYMid meet"
        viewBox={viewBox()}
        onClick={() => selection.set(null)}
      >
        <defs>
          <linearGradient id="node-grad" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="rgba(255,255,255,0.06)" />
            <stop offset="100%" stop-color="rgba(255,255,255,0.0)" />
          </linearGradient>
          <filter id="node-shadow" x="-20%" y="-20%" width="140%" height="140%">
            <feDropShadow dx="0" dy="2" stdDeviation="4" flood-color="#000" flood-opacity="0.35" />
          </filter>
        </defs>
        <For each={tree.edges}>{(edge) => <BtEdge edge={edge} />}</For>
        <For each={tree.nodes}>{(node) => <BtNode node={node} />}</For>
      </svg>

      <Show when={tree.nodes.length === 0}>
        <Placeholder />
      </Show>
    </section>
  );
};

const Placeholder: Component = () => (
  <div class="pointer-events-none absolute inset-0 flex flex-col items-center justify-center text-center">
    <div class="text-sm font-medium text-zinc-400">No tree loaded</div>
    <div class="mt-1 max-w-md text-xs text-zinc-500">
      Open a <span class="font-mono text-zinc-400">.xml</span> file or connect to a running BT.CPP
      <span class="font-mono"> Groot2Publisher</span> via the toolbar above.
    </div>
  </div>
);

export default Canvas;
