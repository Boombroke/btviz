use btparse::BtNode;

#[test]
fn parse_a_xml_returns_one_tree() {
    let xml = include_str!("fixtures/a.xml");
    let trees = btparse::parse_xml(xml).unwrap();
    assert_eq!(trees.len(), 1);
    assert_eq!(trees[0].id, "a");
    // Root is KeepRunningUntilFailure (latest a.xml shape)
    assert_eq!(trees[0].root.registration_name, "KeepRunningUntilFailure");
}

#[test]
fn parse_a_xml_pubgoal_has_topic_name() {
    let xml = include_str!("fixtures/a.xml");
    let trees = btparse::parse_xml(xml).unwrap();
    let mut hits = vec![];
    walk(&trees[0].root, &mut hits);
    assert!(!hits.is_empty(), "expected PubGoal nodes in a.xml");
    let p = hits[0];
    assert_eq!(p.ports.get("topic_name").map(String::as_str), Some("/goal_pose"));
    // first PubGoal in a.xml is at (3.71, -0.61)
    assert_eq!(p.ports.get("goal_pose_x").map(String::as_str), Some("3.71"));
}

#[test]
fn parse_b_xml_has_pubgoal_with_topic_name() {
    let xml = include_str!("fixtures/b.xml");
    let trees = btparse::parse_xml(xml).unwrap();
    let mut hits = vec![];
    walk(&trees[0].root, &mut hits);
    assert!(!hits.is_empty(), "expected PubGoal nodes in b.xml");
    let p1 = hits[0];
    assert_eq!(p1.ports.get("topic_name").map(String::as_str), Some("/goal_pose"));
    // first PubGoal in b.xml goes to point 1 at x=9.17
    assert_eq!(p1.ports.get("goal_pose_x").map(String::as_str), Some("9.17"));
}

#[test]
fn tree_node_ids_are_unique_and_root_is_zero() {
    let xml = include_str!("fixtures/b.xml");
    let trees = btparse::parse_xml(xml).unwrap();
    let mut ids: Vec<u32> = vec![];
    collect_ids(&trees[0].root, &mut ids);
    assert_eq!(trees[0].root.id, 0, "root id should be 0");
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), ids.len(), "node ids must be unique");
}

#[test]
fn tree_nodes_model_is_ignored() {
    let xml = include_str!("fixtures/a.xml");
    let trees = btparse::parse_xml(xml).unwrap();
    // No node should have registration_name in the metadata-only set (these
    // would only appear if we accidentally parsed inside <TreeNodesModel>).
    let mut names = vec![];
    collect_names(&trees[0].root, &mut names);
    for n in &names {
        assert_ne!(n, "input_port", "input_port leaked from TreeNodesModel");
        assert_ne!(n, "TreeNodesModel", "TreeNodesModel itself appeared in tree");
    }
}

fn walk<'a>(n: &'a BtNode, hits: &mut Vec<&'a BtNode>) {
    if n.registration_name == "PubGoal" {
        hits.push(n);
    }
    for c in &n.children {
        walk(c, hits);
    }
}

fn collect_ids(n: &BtNode, ids: &mut Vec<u32>) {
    ids.push(n.id);
    for c in &n.children {
        collect_ids(c, ids);
    }
}

fn collect_names(n: &BtNode, names: &mut Vec<String>) {
    names.push(n.registration_name.clone());
    for c in &n.children {
        collect_names(c, names);
    }
}
