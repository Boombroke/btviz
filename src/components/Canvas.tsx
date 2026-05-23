import type { Component } from "solid-js";
import { For, Show, createMemo, createSignal, onCleanup, onMount } from "solid-js";
import { tree, selection } from "@/stores/tree";
import BtNode from "./BtNode";
import BtEdge from "./BtEdge";

const PADDING = 48;
const MIN_SCALE = 0.2;
const MAX_SCALE = 4.0;

const Canvas: Component = () => {
  let svgRef: SVGSVGElement | undefined;

  /** User pan/zoom on top of the auto-fit base viewBox. */
  const [scale, setScale] = createSignal(1);
  const [panX, setPanX] = createSignal(0);
  const [panY, setPanY] = createSignal(0);

  /** Auto-fit viewBox: snug bounding box + padding. */
  const baseBox = createMemo(() => {
    const ns = tree.nodes;
    if (ns.length === 0) return { x: 0, y: 0, w: 100, h: 100 };
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
    return {
      x: minX - PADDING,
      y: minY - PADDING,
      w: maxX - minX + PADDING * 2,
      h: maxY - minY + PADDING * 2,
    };
  });

  const viewBox = createMemo(() => {
    const b = baseBox();
    return `${b.x} ${b.y} ${b.w} ${b.h}`;
  });

  const resetView = () => {
    setScale(1);
    setPanX(0);
    setPanY(0);
  };

  /** Convert a client pixel position into SVG user coords. */
  const clientToSvg = (clientX: number, clientY: number) => {
    if (!svgRef) return { x: 0, y: 0 };
    const ctm = svgRef.getScreenCTM();
    if (!ctm) return { x: 0, y: 0 };
    const pt = svgRef.createSVGPoint();
    pt.x = clientX;
    pt.y = clientY;
    const p = pt.matrixTransform(ctm.inverse());
    return { x: p.x, y: p.y };
  };

  /** Wheel zoom anchored at cursor. */
  const onWheel = (e: WheelEvent) => {
    e.preventDefault();
    const factor = Math.exp(-e.deltaY * 0.0015);
    const nextRaw = scale() * factor;
    const next = Math.min(MAX_SCALE, Math.max(MIN_SCALE, nextRaw));
    if (next === scale()) return;

    // Anchor: keep the SVG point under the cursor invariant. The transform
    // applied to the inner <g> is `translate(pan) scale(s)`, so the pre-image
    // of cursor SVG point P is (P - pan) / s. After updating s, set pan such
    // that pan' = P - s' * pre.
    const { x: px, y: py } = clientToSvg(e.clientX, e.clientY);
    const s = scale();
    const preX = (px - panX()) / s;
    const preY = (py - panY()) / s;
    setScale(next);
    setPanX(px - next * preX);
    setPanY(py - next * preY);
  };

  /** Drag-to-pan with primary or middle mouse button. */
  let dragging = false;
  let lastClient = { x: 0, y: 0 };
  const onMouseDown = (e: MouseEvent) => {
    // Left button on empty space, or middle button anywhere
    if (e.button !== 0 && e.button !== 1) return;
    if (e.button === 0 && (e.target as Element).closest("g[data-bt-node]")) return;
    dragging = true;
    lastClient = { x: e.clientX, y: e.clientY };
    (e.currentTarget as Element).classList.add("cursor-grabbing");
  };
  const onMouseMove = (e: MouseEvent) => {
    if (!dragging || !svgRef) return;
    // Convert pixel delta to SVG-unit delta. The CTM maps SVG → screen, so
    // the inverse on a pure translation gives the SVG-space delta.
    const ctm = svgRef.getScreenCTM();
    if (!ctm) return;
    const dx = (e.clientX - lastClient.x) / ctm.a;
    const dy = (e.clientY - lastClient.y) / ctm.d;
    lastClient = { x: e.clientX, y: e.clientY };
    setPanX(panX() + dx);
    setPanY(panY() + dy);
  };
  const onMouseUpOrLeave = (e: MouseEvent) => {
    dragging = false;
    (e.currentTarget as Element).classList.remove("cursor-grabbing");
  };

  /** Keyboard: 0 to fit, +/- to zoom. */
  const onKey = (e: KeyboardEvent) => {
    if (e.target && (e.target as HTMLElement).tagName === "INPUT") return;
    if (e.key === "0") resetView();
    else if (e.key === "+" || e.key === "=") setScale(Math.min(MAX_SCALE, scale() * 1.2));
    else if (e.key === "-" || e.key === "_") setScale(Math.max(MIN_SCALE, scale() / 1.2));
  };
  onMount(() => window.addEventListener("keydown", onKey));
  onCleanup(() => window.removeEventListener("keydown", onKey));

  return (
    <section class="relative flex-1 overflow-hidden">
      <div class="dot-grid pointer-events-none absolute inset-0" />

      <svg
        ref={svgRef}
        class="absolute inset-0 h-full w-full cursor-grab select-none"
        preserveAspectRatio="xMidYMid meet"
        viewBox={viewBox()}
        onWheel={onWheel}
        onMouseDown={onMouseDown}
        onMouseMove={onMouseMove}
        onMouseUp={onMouseUpOrLeave}
        onMouseLeave={onMouseUpOrLeave}
        onClick={(e) => {
          // Bare-canvas click clears selection. Don't fire while dragging.
          if (e.target === svgRef) selection.set(null);
        }}
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
        <g transform={`translate(${panX()}, ${panY()}) scale(${scale()})`}>
          <For each={tree.edges}>{(edge) => <BtEdge edge={edge} />}</For>
          <For each={tree.nodes}>{(node) => <BtNode node={node} />}</For>
        </g>
      </svg>

      <Show when={tree.nodes.length === 0}>
        <Placeholder />
      </Show>

      <Show when={tree.nodes.length > 0}>
        <ViewportControls scale={scale} onReset={resetView} onZoomIn={() => setScale(Math.min(MAX_SCALE, scale() * 1.2))} onZoomOut={() => setScale(Math.max(MIN_SCALE, scale() / 1.2))} />
      </Show>
    </section>
  );
};

const ViewportControls: Component<{
  scale: () => number;
  onReset: () => void;
  onZoomIn: () => void;
  onZoomOut: () => void;
}> = (p) => (
  <div class="absolute bottom-3 left-3 flex items-center gap-1 rounded-lg border border-white/5 bg-surface-1/80 p-1 backdrop-blur shadow-floating">
    <button
      type="button"
      onClick={p.onZoomOut}
      class="grid h-7 w-7 place-items-center rounded-md text-zinc-300 transition hover:bg-white/10"
      title="Zoom out (-)"
    >
      <svg viewBox="0 0 16 16" class="h-4 w-4" fill="none" stroke="currentColor" stroke-width="1.6">
        <path d="M4 8h8" stroke-linecap="round" />
      </svg>
    </button>
    <button
      type="button"
      onClick={p.onReset}
      class="rounded-md px-2 py-1 font-mono text-[11px] text-zinc-300 transition hover:bg-white/10"
      title="Fit (0)"
    >
      {Math.round(p.scale() * 100)}%
    </button>
    <button
      type="button"
      onClick={p.onZoomIn}
      class="grid h-7 w-7 place-items-center rounded-md text-zinc-300 transition hover:bg-white/10"
      title="Zoom in (+)"
    >
      <svg viewBox="0 0 16 16" class="h-4 w-4" fill="none" stroke="currentColor" stroke-width="1.6">
        <path d="M4 8h8M8 4v8" stroke-linecap="round" />
      </svg>
    </button>
  </div>
);

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
