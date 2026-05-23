import type { Component } from "solid-js";
import { createSignal, Show } from "solid-js";
import {
  connection,
  connectServer,
  disconnectServer,
  openXmlDialog,
} from "@/stores/tree";

const TopBar: Component = () => {
  const [showConnect, setShowConnect] = createSignal(false);
  const [addr, setAddr] = createSignal("tcp://127.0.0.1:1667");

  const onConnect = async () => {
    setShowConnect(false);
    await connectServer(addr());
  };
  const onDisconnect = async () => {
    await disconnectServer();
  };

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
          onClick={openXmlDialog}
          class="rounded-md border border-white/5 bg-white/5 px-3 py-1 text-xs font-medium text-zinc-200 transition hover:border-white/10 hover:bg-white/10"
        >
          Open File...
        </button>
        <Show
          when={connection.state === "connected"}
          fallback={
            <button
              type="button"
              onClick={() => setShowConnect((v) => !v)}
              class="rounded-md border border-brand-500/30 bg-brand-500/10 px-3 py-1 text-xs font-medium text-brand-300 transition hover:border-brand-500/50 hover:bg-brand-500/20"
            >
              Connect
            </button>
          }
        >
          <button
            type="button"
            onClick={onDisconnect}
            class="rounded-md border border-white/5 bg-white/5 px-3 py-1 text-xs font-medium text-zinc-200 transition hover:border-white/10 hover:bg-white/10"
          >
            Disconnect
          </button>
        </Show>

        <Show when={showConnect()}>
          <div class="flex items-center gap-1">
            <input
              type="text"
              value={addr()}
              onInput={(e) => setAddr(e.currentTarget.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") onConnect();
                if (e.key === "Escape") setShowConnect(false);
              }}
              class="w-48 rounded-md border border-white/10 bg-surface-2 px-2 py-1 text-xs font-mono text-zinc-200 placeholder:text-zinc-500 focus:border-brand-500 focus:outline-none"
              placeholder="tcp://127.0.0.1:1667"
              autofocus
            />
            <button
              type="button"
              onClick={onConnect}
              class="rounded-md bg-brand-500 px-2 py-1 text-xs font-semibold text-zinc-950 transition hover:bg-brand-400"
            >
              Go
            </button>
          </div>
        </Show>
      </div>

      <div class="ml-auto flex items-center gap-3">
        <ConnectionPill />
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
      <span class="font-mono max-w-[200px] truncate">{connection.label}</span>
    </div>
  );
};

export default TopBar;
