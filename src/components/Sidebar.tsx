import type { Component } from "solid-js";
import { Show } from "solid-js";
import { selection } from "@/stores/tree";

const Sidebar: Component = () => {
  return (
    <aside class="glass shadow-floating absolute right-3 top-3 bottom-3 w-80 rounded-xl p-4 text-sm">
      <Show
        when={selection.node()}
        fallback={
          <div class="flex h-full flex-col items-center justify-center text-center">
            <div class="text-xs font-medium uppercase tracking-wide text-zinc-500">
              Inspector
            </div>
            <div class="mt-2 text-xs text-zinc-500">No node selected</div>
          </div>
        }
      >
        {(node) => (
          <div class="flex h-full flex-col">
            <div class="text-xs font-medium uppercase tracking-wide text-zinc-500">
              Selected
            </div>
            <div class="mt-1 font-semibold">{node().type}</div>
            <div class="mt-0.5 font-mono text-[11px] text-zinc-500">id #{node().id}</div>
          </div>
        )}
      </Show>
    </aside>
  );
};

export default Sidebar;
