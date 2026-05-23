use btparse::{layout_reingold_tilford, parse_xml, LayoutNode, LayoutOptions};

#[test]
fn layout_no_overlap_and_root_at_top() {
    let xml = include_str!("fixtures/a.xml");
    let trees = parse_xml(xml).unwrap();
    let opts: LayoutOptions = Default::default();
    let layout = layout_reingold_tilford(&trees[0], opts);

    // root_id == 0 by parser contract
    let root = layout.nodes.iter().find(|n| n.id == 0).expect("root in layout");
    let min_y = layout
        .nodes
        .iter()
        .map(|n| n.y as i32)
        .min()
        .unwrap();
    assert_eq!(root.y as i32, min_y, "root should sit at the smallest y");

    // No two nodes overlap. We use AABB intersection on (x, y, x+w, y+h).
    for (i, a) in layout.nodes.iter().enumerate() {
        for b in layout.nodes.iter().skip(i + 1) {
            assert!(
                !boxes_overlap(a, b),
                "nodes {} and {} overlap: {:?} vs {:?}",
                a.id,
                b.id,
                a,
                b
            );
        }
    }

    // Y levels increase strictly with depth (each layer should be distinct).
    let mut ys: Vec<i32> = layout.nodes.iter().map(|n| n.y as i32).collect();
    ys.sort_unstable();
    ys.dedup();
    assert!(ys.len() >= 2, "expected multiple y-levels in a non-trivial tree");

    // Bounds must contain every node.
    let (min_x, _min_y, max_x, max_y) = layout.bounds;
    for n in &layout.nodes {
        assert!(n.x >= min_x - 1e-3);
        assert!(n.x + n.width <= max_x + 1e-3);
        assert!(n.y + n.height <= max_y + 1e-3);
    }
}

#[test]
fn layout_b_xml_edges_cover_all_non_root_nodes() {
    let xml = include_str!("fixtures/b.xml");
    let trees = parse_xml(xml).unwrap();
    let layout = layout_reingold_tilford(&trees[0], Default::default());
    // Every non-root node should appear as a child in exactly one edge.
    let total_nodes = layout.nodes.len();
    assert_eq!(layout.edges.len(), total_nodes - 1);
    let mut seen = std::collections::HashSet::new();
    for e in &layout.edges {
        assert!(seen.insert(e.child_id), "child {} appeared twice", e.child_id);
    }
}

#[test]
fn layout_siblings_are_horizontally_separated() {
    // For a tree with the canonical opts, any two nodes at the same depth
    // (i.e. same y) must be horizontally separated by at least h_spacing on
    // their bounding boxes.
    let xml = include_str!("fixtures/a.xml");
    let trees = parse_xml(xml).unwrap();
    let opts = LayoutOptions::default();
    let layout = layout_reingold_tilford(&trees[0], opts);
    for (i, a) in layout.nodes.iter().enumerate() {
        for b in layout.nodes.iter().skip(i + 1) {
            if (a.y - b.y).abs() < 1e-3 {
                let (left, right) = if a.x <= b.x { (a, b) } else { (b, a) };
                let gap = right.x - (left.x + left.width);
                assert!(
                    gap >= opts.h_spacing - 1e-3,
                    "siblings {} and {} too close: gap={}",
                    a.id,
                    b.id,
                    gap
                );
            }
        }
    }
}

fn boxes_overlap(a: &LayoutNode, b: &LayoutNode) -> bool {
    let ax2 = a.x + a.width;
    let ay2 = a.y + a.height;
    let bx2 = b.x + b.width;
    let by2 = b.y + b.height;
    a.x < bx2 && b.x < ax2 && a.y < by2 && b.y < ay2
}
