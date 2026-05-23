//! Tauri command surface.
//!
//! These are placeholders for Day 1 integration: the real implementations
//! will delegate to `btparse` (XML parsing + Reingold-Tilford layout) and
//! `groot2_client` (ZMQ live status), both of which live in sibling git
//! worktrees and will be added as workspace dependencies during merge.

use serde::Serialize;

#[derive(Serialize)]
pub struct LayoutNode {
    pub id: u32,
    pub r#type: String,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Serialize)]
pub struct LayoutEdge {
    pub from: u32,
    pub to: u32,
}

#[derive(Serialize)]
pub struct LayoutResult {
    pub tree_id: String,
    pub nodes: Vec<LayoutNode>,
    pub edges: Vec<LayoutEdge>,
}

/// Parse a BT XML file off disk and return its layout. Wired up to btparse
/// during the merge step on Day 1.
#[tauri::command]
pub async fn load_xml(path: String) -> Result<LayoutResult, String> {
    // TODO(day-1-merge): replace placeholder once btparse is added as a workspace dep.
    let _ = path;
    Ok(LayoutResult {
        tree_id: "placeholder".into(),
        nodes: vec![],
        edges: vec![],
    })
}
