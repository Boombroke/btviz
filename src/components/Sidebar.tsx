import type { Component } from "solid-js";
import { For, Show } from "solid-js";
import { selection, type NodeStatus } from "@/stores/tree";

const statusBg: Record<NodeStatus, string> = {
  idle: "bg-status-idle/30 text-zinc-300",
  running: "bg-status-running/20 text-status-running",
  success: "bg-status-success/20 text-status-success",
  failure: "bg-status-failure/20 text-status-failure",
  skipped: "bg-status-skipped/20 text-zinc-300",
};

const Sidebar: Component = () => {
  return (
    <aside class="glass shadow-floating absolute right-3 top-3 bottom-3 w-80 rounded-xl p-4 text-sm overflow-hidden">
      <Show
        when={selection.node()}
        fallback={
          <div class="flex h-full flex-col items-center justify-center text-center">
            <div class="text-xs font-medium uppercase tracking-wide text-zinc-500">
              Inspector
            </div>
            <div class="mt-2 text-xs text-zinc-500">No node selected</div>
            <div class="mt-1 text-[11px] text-zinc-600">Click a node in the canvas</div>
          </div>
        }
      >
        {(node) => (
          <div class="flex h-full flex-col">
            <div class="flex items-baseline justify-between">
              <div class="text-xs font-medium uppercase tracking-wide text-zinc-500">
                Selected
              </div>
              <div
                class={`rounded-full px-2 py-0.5 text-[10px] font-mono uppercase tracking-wider transition ${statusBg[node().status]}`}
              >
                {node().status}
              </div>
            </div>
            <div class="mt-1 truncate font-semibold">{node().displayName ?? node().type}</div>
            <Show when={node().displayName}>
              <div class="font-mono text-[11px] text-zinc-400">{node().type}</div>
            </Show>
            <div class="mt-0.5 font-mono text-[11px] text-zinc-500">id #{node().id}</div>

            <div class="mt-4 text-[10px] font-medium uppercase tracking-wide text-zinc-500">
              Ports
            </div>
            <Show
              when={Object.keys(node().ports).length > 0}
              fallback={<div class="mt-1 text-[11px] text-zinc-500">(no ports)</div>}
            >
              <div class="no-scrollbar mt-2 flex-1 space-y-1.5 overflow-y-auto pr-1">
                <For each={Object.entries(node().ports)}>
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

export default Sidebar;
