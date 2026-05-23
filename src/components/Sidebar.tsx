import type { Component } from "solid-js";
import { For, Show, createMemo } from "solid-js";
import { tree, selection, type NodeStatus } from "@/stores/tree";

const statusBg: Record<NodeStatus, string> = {
  idle: "bg-status-idle/30 text-zinc-300",
  running: "bg-status-running/20 text-status-running",
  success: "bg-status-success/20 text-status-success",
  failure: "bg-status-failure/20 text-status-failure",
  skipped: "bg-status-skipped/20 text-zinc-300",
};

/** Aggregate counts for the empty-state overview. */
function statusCounts() {
  const out: Record<NodeStatus, number> = {
    idle: 0,
    running: 0,
    success: 0,
    failure: 0,
    skipped: 0,
  };
  for (const n of tree.nodes) out[n.status]++;
  return out;
}

const Sidebar: Component = () => {
  const node = selection.node;

  /** Build adjacency from edges so we can show parent type + children count. */
  const relations = createMemo(() => {
    const sel = node();
    if (!sel) return null;
    const parentId = tree.edges.find((e) => e.to === sel.id)?.from ?? null;
    const childIds = tree.edges.filter((e) => e.from === sel.id).map((e) => e.to);
    const parent = parentId == null ? null : tree.nodes.find((n) => n.id === parentId) ?? null;
    return { parent, childCount: childIds.length };
  });

  return (
    <aside class="glass shadow-floating absolute right-3 top-3 bottom-3 w-80 rounded-xl p-4 text-sm overflow-hidden">
      <Show when={node()} fallback={<EmptyState />}>
        {(n) => (
          <div class="flex h-full flex-col">
            <div class="flex items-baseline justify-between">
              <div class="text-xs font-medium uppercase tracking-wide text-zinc-500">
                Selected
              </div>
              <div
                class={`rounded-full px-2 py-0.5 text-[10px] font-mono uppercase tracking-wider transition ${statusBg[n().status]}`}
              >
                {n().status}
              </div>
            </div>
            <div class="mt-1 truncate font-semibold">{n().displayName ?? n().type}</div>
            <Show when={n().displayName}>
              <div class="font-mono text-[11px] text-zinc-400">{n().type}</div>
            </Show>
            <div class="mt-0.5 font-mono text-[11px] text-zinc-500">id #{n().id}</div>

            <Show when={relations()}>
              {(r) => (
                <div class="mt-3 grid grid-cols-2 gap-2 text-[11px]">
                  <div class="rounded-md border border-white/5 bg-surface-2/50 p-2">
                    <div class="text-[10px] uppercase tracking-wide text-zinc-500">Parent</div>
                    <Show
                      when={r().parent}
                      fallback={<div class="mt-0.5 font-mono text-zinc-500">root</div>}
                    >
                      {(par) => (
                        <button
                          type="button"
                          onClick={() => selection.set(par().id)}
                          class="mt-0.5 max-w-full truncate font-mono text-zinc-100 hover:text-brand-400"
                          title={`Jump to #${par().id}`}
                        >
                          {par().displayName ?? par().type}
                        </button>
                      )}
                    </Show>
                  </div>
                  <div class="rounded-md border border-white/5 bg-surface-2/50 p-2">
                    <div class="text-[10px] uppercase tracking-wide text-zinc-500">Children</div>
                    <div class="mt-0.5 font-mono text-zinc-100">{r().childCount}</div>
                  </div>
                </div>
              )}
            </Show>

            <div class="mt-4 flex items-baseline justify-between">
              <div class="text-[10px] font-medium uppercase tracking-wide text-zinc-500">
                Ports
              </div>
              <div class="font-mono text-[10px] text-zinc-600">
                {Object.keys(n().ports).length}
              </div>
            </div>
            <Show
              when={Object.keys(n().ports).length > 0}
              fallback={<div class="mt-1 text-[11px] text-zinc-500">(no ports)</div>}
            >
              <div class="no-scrollbar mt-2 flex-1 space-y-1.5 overflow-y-auto pr-1">
                <For each={Object.entries(n().ports)}>
                  {([k, v]) => (
                    <div class="rounded-md border border-white/5 bg-surface-2/60 p-2">
                      <div class="font-mono text-[10px] text-zinc-500">{k}</div>
                      <div class="mt-0.5 break-all font-mono text-[11px] text-zinc-100">{v}</div>
                    </div>
                  )}
                </For>
              </div>
            </Show>
          </div>
        )}
      </Show>
    </aside>
  );
};

/** Empty inspector state. Shows tree-wide stats so the panel isn't dead space. */
const EmptyState: Component = () => {
  const counts = createMemo(statusCounts);
  const total = createMemo(() => tree.nodes.length);

  return (
    <div class="flex h-full flex-col">
      <div class="text-xs font-medium uppercase tracking-wide text-zinc-500">Inspector</div>
      <Show
        when={total() > 0}
        fallback={
          <div class="mt-3 text-xs text-zinc-500">
            No tree loaded. Open a file or connect to a running BT.
          </div>
        }
      >
        <div class="mt-1 text-[11px] text-zinc-500">
          Click a node in the canvas to inspect it.
        </div>
        <div class="mt-4 text-[10px] font-medium uppercase tracking-wide text-zinc-500">
          Tree overview
        </div>
        <div class="mt-2 grid grid-cols-2 gap-2 text-[11px]">
          <Stat label="Nodes" value={total()} />
          <Stat label="Running" value={counts().running} accent="text-status-running" />
          <Stat label="Success" value={counts().success} accent="text-status-success" />
          <Stat label="Failure" value={counts().failure} accent="text-status-failure" />
          <Stat label="Skipped" value={counts().skipped} />
          <Stat label="Idle" value={counts().idle} />
        </div>
        <div class="mt-auto pt-3 text-[10px] text-zinc-600">
          Drag canvas to pan · scroll to zoom · press 0 to fit
        </div>
      </Show>
    </div>
  );
};

const Stat: Component<{ label: string; value: number; accent?: string }> = (p) => (
  <div class="rounded-md border border-white/5 bg-surface-2/50 p-2">
    <div class="text-[10px] uppercase tracking-wide text-zinc-500">{p.label}</div>
    <div class={`mt-0.5 font-mono text-[12px] ${p.accent ?? "text-zinc-100"}`}>{p.value}</div>
  </div>
);

export default Sidebar;
