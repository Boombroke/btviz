//! Tauri command surface.
//!
//! Two surfaces:
//!
//! - File mode: [`load_xml`] reads a `.xml` off disk, runs `btparse`, returns
//!   the laid-out tree to the frontend.
//! - Live mode: [`connect_server`] / [`disconnect_server`] start/stop a
//!   background poller that hits `BT::Groot2Publisher` over ZMQ and emits
//!   `btviz://status` (per-frame status diffs) Tauri events.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

/// Layout payload sent to the frontend.
///
/// Mirrors `btparse::Layout` but with names matching `src/stores/tree.ts` so
/// the Solid store can `setTree(payload)` in one go.
#[derive(Serialize, Clone)]
pub struct LayoutNode {
    pub id: u32,
    pub r#type: String,
    pub display_name: Option<String>,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub status: String,
    pub ports: serde_json::Value,
}

#[derive(Serialize, Clone)]
pub struct LayoutEdge {
    pub from: u32,
    pub to: u32,
}

#[derive(Serialize, Clone)]
pub struct LayoutResult {
    pub tree_id: String,
    pub nodes: Vec<LayoutNode>,
    pub edges: Vec<LayoutEdge>,
}

#[derive(Serialize, Clone)]
pub struct StatusPayload {
    /// Map of node uid -> status string ("idle" | "running" | "success" | "failure" | "skipped").
    pub statuses: std::collections::HashMap<u32, String>,
}

fn status_to_str(s: groot2_client::NodeStatus) -> &'static str {
    match s {
        groot2_client::NodeStatus::Idle => "idle",
        groot2_client::NodeStatus::Running => "running",
        groot2_client::NodeStatus::Success => "success",
        groot2_client::NodeStatus::Failure => "failure",
        groot2_client::NodeStatus::Skipped => "skipped",
    }
}

fn build_layout(xml: &str) -> Result<LayoutResult, String> {
    let trees = btparse::parse_xml(xml).map_err(|e| format!("parse: {e}"))?;
    let tree = trees
        .into_iter()
        .next()
        .ok_or_else(|| "no <BehaviorTree> in XML".to_string())?;
    let layout = btparse::layout_reingold_tilford(&tree, btparse::LayoutOptions::default());

    // Walk the parsed tree once to associate registration name + ports with
    // each layout node id. btparse assigns ids in pre-order starting at 0.
    let mut by_id: std::collections::HashMap<u32, &btparse::BtNode> =
        std::collections::HashMap::new();
    fn walk<'a>(
        n: &'a btparse::BtNode,
        m: &mut std::collections::HashMap<u32, &'a btparse::BtNode>,
    ) {
        m.insert(n.id, n);
        for c in &n.children {
            walk(c, m);
        }
    }
    walk(&tree.root, &mut by_id);

    let nodes = layout
        .nodes
        .into_iter()
        .map(|ln| {
            let n = by_id.get(&ln.id).copied();
            LayoutNode {
                id: ln.id,
                r#type: n.map(|n| n.registration_name.clone()).unwrap_or_default(),
                display_name: n.and_then(|n| n.display_name.clone()),
                x: ln.x,
                y: ln.y,
                w: ln.width,
                h: ln.height,
                status: "idle".into(),
                ports: n
                    .map(|n| serde_json::to_value(&n.ports).unwrap_or(serde_json::Value::Null))
                    .unwrap_or(serde_json::Value::Null),
            }
        })
        .collect();
    let edges = layout
        .edges
        .into_iter()
        .map(|e| LayoutEdge {
            from: e.parent_id,
            to: e.child_id,
        })
        .collect();

    Ok(LayoutResult {
        tree_id: tree.id,
        nodes,
        edges,
    })
}

/// Read a BT XML file from disk and return its layout. Used in "file" mode.
#[tauri::command]
pub async fn load_xml(path: String) -> Result<LayoutResult, String> {
    let xml = std::fs::read_to_string(&path).map_err(|e| format!("read {path}: {e}"))?;
    build_layout(&xml)
}

/// Live-mode shared state. Holds a flag the poller thread watches so we can
/// disconnect cleanly. Stored as a Tauri-managed singleton.
#[derive(Default)]
pub struct LiveSession {
    pub running: Arc<AtomicBool>,
}

/// Connect to a `BT::Groot2Publisher`, fetch the full tree, then start a
/// background thread polling status. Returns the initial layout. The frontend
/// receives subsequent updates via the `btviz://status` event.
#[tauri::command]
pub async fn connect_server(
    addr: String,
    poll_hz: Option<f32>,
    app: AppHandle,
    session: State<'_, LiveSession>,
) -> Result<LayoutResult, String> {
    // Stop any existing poller before starting a new one.
    session.running.store(false, Ordering::Relaxed);

    let mut client = groot2_client::Groot2Client::new(addr.clone());
    client.set_timeout(Duration::from_secs(2));

    // 1) Pull the tree XML synchronously so the frontend has something to draw.
    let xml = client
        .request_full_tree()
        .map_err(|e| format!("request_full_tree({addr}): {e}"))?;
    let layout = build_layout(&xml)?;

    // 2) Spawn the status poller. `poll_hz` defaults to 50 — high enough that
    //    every BT tick is visible without unduly stressing ZMQ. Status diffs
    //    are computed against the previous snapshot so we only emit when
    //    something actually changed.
    let hz = poll_hz.unwrap_or(50.0).clamp(1.0, 200.0);
    let period = Duration::from_millis(((1000.0 / hz) as u64).max(1));
    let running = session.running.clone();
    running.store(true, Ordering::Relaxed);
    let app_for_thread = app.clone();
    let addr_for_thread = addr.clone();

    std::thread::spawn(move || {
        let mut client = groot2_client::Groot2Client::new(addr_for_thread);
        client.set_timeout(Duration::from_millis(800));
        let mut prev: std::collections::HashMap<u16, groot2_client::NodeStatus> =
            std::collections::HashMap::new();
        while running.load(Ordering::Relaxed) {
            match client.request_status_map() {
                Ok(map) => {
                    let mut diff = std::collections::HashMap::new();
                    for (uid, st) in &map {
                        if prev.get(uid) != Some(st) {
                            diff.insert(*uid as u32, status_to_str(*st).to_string());
                        }
                    }
                    if !diff.is_empty() {
                        let _ = app_for_thread
                            .emit("btviz://status", StatusPayload { statuses: diff });
                    }
                    prev = map;
                }
                Err(e) => {
                    let _ = app_for_thread.emit(
                        "btviz://error",
                        serde_json::json!({ "where": "status_poll", "error": e.to_string() }),
                    );
                    // Back off briefly so we don't tight-loop while the server is down.
                    std::thread::sleep(Duration::from_millis(500));
                }
            }
            std::thread::sleep(period);
        }
    });

    Ok(layout)
}

/// Stop the status poller. The thread loops on an `AtomicBool` so this is
/// fire-and-forget; the next iteration exits.
#[tauri::command]
pub fn disconnect_server(session: State<'_, LiveSession>) {
    session.running.store(false, Ordering::Relaxed);
}
