import { createSignal } from "solid-js";
import { createStore, produce } from "solid-js/store";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { open as openDialog } from "@tauri-apps/plugin-dialog";

/** Status mirrors `groot2_client::NodeStatus` on the backend. */
export type NodeStatus = "idle" | "running" | "success" | "failure" | "skipped";

export interface BtNode {
  id: number;
  type: string;
  displayName?: string;
  x: number;
  y: number;
  w: number;
  h: number;
  status: NodeStatus;
  /** Raw port attributes parsed off the XML (e.g. `goal_pose_x`). */
  ports: Record<string, string>;
}

export interface BtEdge {
  from: number;
  to: number;
}

export interface TreeStore {
  treeId: string | null;
  nodes: BtNode[];
  edges: BtEdge[];
}

/** Wire format from Tauri commands. snake_case + `type` keyword. */
interface WireLayoutNode {
  id: number;
  type: string;
  display_name?: string | null;
  x: number;
  y: number;
  w: number;
  h: number;
  status: NodeStatus;
  ports: Record<string, string> | null;
}
interface WireLayoutEdge {
  from: number;
  to: number;
}
interface WireLayoutResult {
  tree_id: string;
  nodes: WireLayoutNode[];
  edges: WireLayoutEdge[];
}
interface WireStatusPayload {
  statuses: Record<string, NodeStatus>;
}

/** Tree topology + per-node status. */
export const [tree, setTree] = createStore<TreeStore>({
  treeId: null,
  nodes: [],
  edges: [],
});

/** Connection state for the top-bar pill. */
export type ConnectionState = "disconnected" | "connecting" | "connected" | "error";

export interface ConnectionStore {
  state: ConnectionState;
  label: string;
}

export const [connection, setConnection] = createStore<ConnectionStore>({
  state: "disconnected",
  label: "disconnected",
});

/** Selected node id for the inspector sidebar. */
const [selectedId, setSelectedId] = createSignal<number | null>(null);
export const selection = {
  id: selectedId,
  set: setSelectedId,
  node: () => {
    const id = selectedId();
    return id == null ? undefined : tree.nodes.find((n) => n.id === id);
  },
};

function applyLayout(payload: WireLayoutResult) {
  setSelectedId(null);
  setTree({
    treeId: payload.tree_id,
    nodes: payload.nodes.map((n) => ({
      id: n.id,
      type: n.type,
      displayName: n.display_name ?? undefined,
      x: n.x,
      y: n.y,
      w: n.w,
      h: n.h,
      status: n.status,
      ports: n.ports ?? {},
    })),
    edges: payload.edges,
  });
}

/** Open a `.xml` from disk via the Tauri dialog plugin and load it. */
export async function openXmlDialog() {
  const picked = await openDialog({
    multiple: false,
    filters: [
      { name: "BehaviorTree XML", extensions: ["xml"] },
      { name: "All", extensions: ["*"] },
    ],
  });
  if (typeof picked !== "string" || !picked) return;
  setConnection({ state: "connecting", label: "loading..." });
  try {
    const result = await invoke<WireLayoutResult>("load_xml", { path: picked });
    applyLayout(result);
    setConnection({ state: "disconnected", label: `file: ${shortenPath(picked)}` });
  } catch (e) {
    setConnection({ state: "error", label: `load failed: ${e}` });
  }
}

let unlistenStatus: UnlistenFn | null = null;
let unlistenError: UnlistenFn | null = null;

async function ensureListeners() {
  if (!unlistenStatus) {
    unlistenStatus = await listen<WireStatusPayload>("btviz://status", (ev) => {
      // Patch only nodes that changed. Solid's produce keeps reactivity scoped
      // to the touched array slot.
      setTree(
        "nodes",
        produce((nodes: BtNode[]) => {
          for (let i = 0; i < nodes.length; i++) {
            const next = ev.payload.statuses[String(nodes[i].id)];
            if (next && nodes[i].status !== next) nodes[i].status = next;
          }
        }),
      );
    });
  }
  if (!unlistenError) {
    unlistenError = await listen<{ where: string; error: string }>("btviz://error", (ev) => {
      setConnection({ state: "error", label: `server: ${ev.payload.error.slice(0, 40)}` });
    });
  }
}

/** Connect to a `BT::Groot2Publisher` at the given address. */
export async function connectServer(addr = "tcp://127.0.0.1:1667") {
  setConnection({ state: "connecting", label: addr });
  await ensureListeners();
  try {
    const result = await invoke<WireLayoutResult>("connect_server", { addr });
    applyLayout(result);
    setConnection({ state: "connected", label: addr });
  } catch (e) {
    setConnection({ state: "error", label: `connect failed: ${e}` });
  }
}

export async function disconnectServer() {
  await invoke("disconnect_server");
  setConnection({ state: "disconnected", label: "disconnected" });
}

function shortenPath(path: string): string {
  const parts = path.split(/[/\\]/);
  return parts.slice(-2).join("/");
}
