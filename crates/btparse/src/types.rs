//! Shared data structures for the BT XML AST.

use std::collections::HashMap;

/// A single `<BehaviorTree ID="...">` parsed from an XML document.
#[derive(Debug, Clone, PartialEq)]
pub struct BtTree {
    /// Value of the `ID` attribute on `<BehaviorTree>`.
    pub id: String,
    /// Root node of the tree.
    pub root: BtNode,
}

/// One BT node in the in-memory tree.
#[derive(Debug, Clone, PartialEq)]
pub struct BtNode {
    /// Stable id assigned during parsing (unique within a tree, root = 0,
    /// allocated in pre-order).
    pub id: u32,
    /// Tag name on the wire — i.e. `Sequence`, `Fallback`, or a user
    /// registered name like `PubGoal`.
    ///
    /// For BT.CPP 4 `<Action ID="Foo">` style tags we surface `Foo` here so
    /// downstream code does not need to special-case the `name="ID"`
    /// attribute.
    pub registration_name: String,
    /// `name="..."` attribute, if present. BT.CPP uses this as the human
    /// readable label distinct from the registration.
    pub display_name: Option<String>,
    /// All other attributes as-is (key port mappings, literals, etc).
    /// `name`, `ID` and `_description` style scaffolding attributes are not
    /// stored here.
    pub ports: HashMap<String, String>,
    /// Children, in document order.
    pub children: Vec<BtNode>,
}

impl BtNode {
    /// Construct a new leaf node with the given id and registration name.
    pub fn new(id: u32, registration_name: impl Into<String>) -> Self {
        Self {
            id,
            registration_name: registration_name.into(),
            display_name: None,
            ports: HashMap::new(),
            children: Vec::new(),
        }
    }
}
