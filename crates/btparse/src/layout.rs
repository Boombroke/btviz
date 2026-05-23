//! Top-down Reingold-Tilford-style tree layout.
//!
//! This is the simplified variant: each subtree gets a horizontal slot equal
//! to the sum of its children's subtree widths (with `h_spacing` between
//! siblings). The parent is centered above the bounding range of its
//! children. This guarantees no in-tree overlap and reads cleanly for the
//! moderately-deep BTs we care about. We skip the cross-subtree
//! tightening of the original paper — good enough for a visualizer.

use crate::types::{BtNode, BtTree};

/// Geometry of a single node after layout.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutNode {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Parent → child edge for the renderer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutEdge {
    pub parent_id: u32,
    pub child_id: u32,
}

/// Layout output.
#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    pub nodes: Vec<LayoutNode>,
    pub edges: Vec<LayoutEdge>,
    /// (min_x, min_y, max_x, max_y) bounding rect of the laid-out tree.
    pub bounds: (f32, f32, f32, f32),
}

/// Tunable spacing parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutOptions {
    /// Width of every node box.
    pub node_width: f32,
    /// Height of every node box.
    pub node_height: f32,
    /// Horizontal gap between sibling subtrees.
    pub h_spacing: f32,
    /// Vertical gap between a parent's bottom edge and its child's top edge.
    pub v_spacing: f32,
}

impl Default for LayoutOptions {
    fn default() -> Self {
        Self {
            node_width: 160.0,
            node_height: 60.0,
            h_spacing: 24.0,
            v_spacing: 80.0,
        }
    }
}

/// Compute a top-down layout for `tree`.
///
/// The root sits at `y = 0` (top of its box) and grows downward. Final
/// `bounds` is shifted so that `min_x = 0` (i.e. the tree is left-aligned).
pub fn layout_reingold_tilford(tree: &BtTree, opts: LayoutOptions) -> Layout {
    let mut nodes: Vec<LayoutNode> = Vec::new();
    let mut edges: Vec<LayoutEdge> = Vec::new();

    // Pass 1: compute subtree width per node.
    let root_width = subtree_width(&tree.root, &opts);

    // Pass 2: assign coordinates. We give the root a horizontal slot of
    // `root_width`, starting at left=0. Each child gets a sub-slot proportional
    // to its own subtree width.
    assign_positions(
        &tree.root,
        0.0,
        root_width,
        0,
        &opts,
        &mut nodes,
        &mut edges,
        None,
    );

    let (min_x, min_y, max_x, max_y) = compute_bounds(&nodes);
    Layout {
        nodes,
        edges,
        bounds: (min_x, min_y, max_x, max_y),
    }
}

fn subtree_width(node: &BtNode, opts: &LayoutOptions) -> f32 {
    if node.children.is_empty() {
        return opts.node_width;
    }
    let mut total = 0.0;
    for (i, c) in node.children.iter().enumerate() {
        if i > 0 {
            total += opts.h_spacing;
        }
        total += subtree_width(c, opts);
    }
    // Parent must be at least as wide as its own box, otherwise a 1-child
    // chain would shrink below `node_width`.
    total.max(opts.node_width)
}

#[allow(clippy::too_many_arguments)]
fn assign_positions(
    node: &BtNode,
    slot_left: f32,
    slot_width: f32,
    depth: u32,
    opts: &LayoutOptions,
    nodes: &mut Vec<LayoutNode>,
    edges: &mut Vec<LayoutEdge>,
    parent_id: Option<u32>,
) {
    let center_x = slot_left + slot_width / 2.0;
    let x = center_x - opts.node_width / 2.0;
    let y = depth as f32 * (opts.node_height + opts.v_spacing);

    nodes.push(LayoutNode {
        id: node.id,
        x,
        y,
        width: opts.node_width,
        height: opts.node_height,
    });
    if let Some(pid) = parent_id {
        edges.push(LayoutEdge {
            parent_id: pid,
            child_id: node.id,
        });
    }

    if node.children.is_empty() {
        return;
    }

    // Distribute children across the parent's slot. We center the children's
    // total width inside the slot so the parent stays above the centroid of
    // its kids even when one child's subtree is much wider than the parent
    // itself.
    let children_widths: Vec<f32> = node.children.iter().map(|c| subtree_width(c, opts)).collect();
    let children_total: f32 = children_widths.iter().sum::<f32>()
        + opts.h_spacing * (node.children.len().saturating_sub(1) as f32);
    let mut cursor = slot_left + (slot_width - children_total) / 2.0;
    for (i, child) in node.children.iter().enumerate() {
        let cw = children_widths[i];
        assign_positions(
            child,
            cursor,
            cw,
            depth + 1,
            opts,
            nodes,
            edges,
            Some(node.id),
        );
        cursor += cw + opts.h_spacing;
    }
}

fn compute_bounds(nodes: &[LayoutNode]) -> (f32, f32, f32, f32) {
    if nodes.is_empty() {
        return (0.0, 0.0, 0.0, 0.0);
    }
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for n in nodes {
        if n.x < min_x {
            min_x = n.x;
        }
        if n.y < min_y {
            min_y = n.y;
        }
        if n.x + n.width > max_x {
            max_x = n.x + n.width;
        }
        if n.y + n.height > max_y {
            max_y = n.y + n.height;
        }
    }
    (min_x, min_y, max_x, max_y)
}
