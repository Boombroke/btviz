import { createSignal } from "solid-js";
import { createStore } from "solid-js/store";

/** Status mirrors `groot2_client::NodeStatus`. */
export type NodeStatus = "idle" | "running" | "success" | "failure" | "skipped";

export interface BtNode {
  id: number;
  type: string;
  x: number;
  y: number;
  w: number;
  h: number;
  status: NodeStatus;
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

/** Tree topology + per-node status. Populated by the Tauri backend in
 *  Day 1 (parser) / Day 2 (live status). */
export const [tree, setTree] = createStore<TreeStore>({
  treeId: null,
  nodes: [],
  edges: [],
});

/** Connection state for the top bar pill. */
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
