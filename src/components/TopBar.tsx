import type { Component } from "solid-js";
import { connection } from "@/stores/tree";

const TopBar: Component = () => {
  return (
    <header class="z-10 flex h-12 shrink-0 items-center gap-3 border-b border-white/5 bg-surface-1/80 px-4 backdrop-blur">
      <div class="flex items-center gap-2">
        <div class="flex h-6 w-6 items-center justify-center rounded-md bg-brand-500/15 text-brand-400">
          <svg viewBox="0 0 16 16" class="h-4 w-4" fill="currentColor" aria-hidden>
            <circle cx="8" cy="3" r="2" />
            <circle cx="3" cy="13" r="2" />
            <circle cx="13" cy="13" r="2" />
            <path
              d="M8 5v3M8 8L4 11M8 8l4 3"
              stroke="currentColor"
              stroke-width="1.4"
              stroke-linecap="round"
              fill="none"
            />
          </svg>
        </div>
        <span class="text-sm font-semibold tracking-wide">btviz</span>
        <span class="font-mono text-[10px] text-zinc-500">v0.1.0</span>
      </div>

      <div class="ml-4 flex items-center gap-2">
        <button
          type="button"
          class="rounded-md border border-white/5 bg-white/5 px-3 py-1 text-xs font-medium text-zinc-200 transition hover:border-white/10 hover:bg-white/10"
          disabled
          title="Wired in Day 1 integration"
        >
          Open File...
        </button>
        <button
          type="button"
          class="rounded-md border border-white/5 bg-white/5 px-3 py-1 text-xs font-medium text-zinc-200 transition hover:border-white/10 hover:bg-white/10"
          disabled
          title="Wired in Day 2 (groot2_client)"
        >
          Connect
        </button>
      </div>

      <div class="ml-auto flex items-center gap-3">
        <ConnectionPill />
        <button
          type="button"
          class="rounded-md border border-white/5 bg-white/5 p-1.5 text-zinc-300 transition hover:border-white/10 hover:bg-white/10"
          title="Toggle theme (Day 3)"
          disabled
          aria-label="Toggle theme"
        >
          <svg viewBox="0 0 24 24" class="h-3.5 w-3.5" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="4" />
            <path d="M12 2v2M12 20v2M2 12h2M20 12h2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M4.93 19.07l1.41-1.41M17.66 6.34l1.41-1.41" />
          </svg>
        </button>
      </div>
    </header>
  );
};

const ConnectionPill: Component = () => {
  const dotClass = () => {
    switch (connection.state) {
      case "connected":
        return "bg-status-running shadow-[0_0_0_3px] shadow-status-running/20";
      case "connecting":
        return "bg-status-running/60 animate-pulse";
      case "error":
        return "bg-status-failure";
      default:
        return "bg-status-idle";
    }
  };
  return (
    <div class="flex items-center gap-1.5 rounded-full border border-white/5 bg-white/5 px-2.5 py-1 text-[11px] text-zinc-300">
      <span class={`h-2 w-2 rounded-full transition-colors duration-200 ${dotClass()}`} />
      <span class="font-mono">{connection.label}</span>
    </div>
  );
};

export default TopBar;
