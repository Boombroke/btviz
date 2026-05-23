//! Integration tests against a live `BT::Groot2Publisher`.
//!
//! These are gated behind `#[ignore]` because they need:
//! 1. A BT.CPP server running on `tcp://127.0.0.1:1667`
//! 2. An active behavior tree (the publisher is constructed when a tree
//!    is created via `factory.createTree`)
//!
//! Run with:
//! ```sh
//! cargo test --manifest-path crates/groot2_client/Cargo.toml -- --ignored
//! ```

use groot2_client::{Groot2Client, NodeStatus};
use std::time::Duration;

fn client() -> Groot2Client {
    let mut c = Groot2Client::local(1667);
    c.set_timeout(Duration::from_secs(2));
    c
}

#[test]
#[ignore]
fn fetch_full_tree_xml() {
    let c = client();
    let xml = c.request_full_tree().expect("request_full_tree failed");
    assert!(
        xml.contains("<root") && xml.contains("BehaviorTree"),
        "reply did not look like BT XML: {}",
        &xml.chars().take(80).collect::<String>()
    );
    eprintln!("[live] FULLTREE returned {} bytes", xml.len());
}

#[test]
#[ignore]
fn fetch_status_map() {
    let c = client();
    let map = c.request_status_map().expect("request_status_map failed");
    assert!(!map.is_empty(), "status map should be non-empty");
    let counts: std::collections::HashMap<NodeStatus, usize> = map.values().fold(
        std::collections::HashMap::new(),
        |mut acc, s| {
            *acc.entry(*s).or_insert(0) += 1;
            acc
        },
    );
    eprintln!("[live] STATUS: {} nodes, breakdown {:?}", map.len(), counts);
}
