//! BT.CPP 4.x XML parser + Reingold-Tilford layout.
//!
//! See [`parse_xml`] to convert XML into a list of [`BtTree`]s, and
//! [`layout_reingold_tilford`] to compute geometric positions for each
//! node in a tree.

pub mod layout;
pub mod parse;
pub mod types;

pub use layout::{layout_reingold_tilford, Layout, LayoutEdge, LayoutNode, LayoutOptions};
pub use parse::{parse_xml, ParseError};
pub use types::{BtNode, BtTree};
