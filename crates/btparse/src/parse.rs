//! BT.CPP 4.x XML -> [`BtTree`] parser.
//!
//! Highlights:
//! * Multiple `<BehaviorTree>` per document (top-level `<root>` wrapper).
//! * `<TreeNodesModel>` is fully ignored — it is metadata for editors.
//! * Decorator/control/leaf distinction is not made here; we just keep the
//!   tag name as `registration_name` and let downstream code interpret.
//! * BT.CPP allows two spellings for action / condition nodes:
//!     1. `<PubGoal ...>`                       (registration as tag)
//!     2. `<Action ID="PubGoal" ...>`            (Groot editor style)
//!   Both collapse to `registration_name = "PubGoal"`.

use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::reader::Reader;
use std::collections::HashMap;
use thiserror::Error;

use crate::types::{BtNode, BtTree};

/// Errors surfaced from [`parse_xml`].
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("xml read error: {0}")]
    Xml(#[from] quick_xml::Error),

    #[error("xml attribute error: {0}")]
    Attr(#[from] quick_xml::events::attributes::AttrError),

    #[error("utf-8 error in xml: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("<BehaviorTree> with id `{0}` has no root node")]
    EmptyTree(String),

    #[error("<BehaviorTree> is missing the required ID attribute")]
    BehaviorTreeMissingId,

    #[error("unexpected EOF inside element <{0}>")]
    UnexpectedEof(String),

    #[error("malformed structure: {0}")]
    Malformed(String),
}

/// Parse an XML string into one or more [`BtTree`]s.
///
/// `<TreeNodesModel>` is silently ignored. The wrapping `<root>` (or a bare
/// `<BehaviorTree>`) is supported.
pub fn parse_xml(xml: &str) -> Result<Vec<BtTree>, ParseError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut trees: Vec<BtTree> = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Eof => break,
            Event::Start(e) => {
                let name = local_name(&e)?;
                match name.as_str() {
                    "root" | "BehaviorTreeRoot" => {
                        // descend
                    }
                    "BehaviorTree" => {
                        let tree = parse_behavior_tree(&mut reader, &e)?;
                        trees.push(tree);
                    }
                    "TreeNodesModel" => {
                        skip_to_end(&mut reader, "TreeNodesModel")?;
                    }
                    _ => {
                        // Unknown top-level wrapper — descend without
                        // consuming so that nested <BehaviorTree> is still
                        // discovered.
                    }
                }
            }
            Event::Empty(_) => { /* nothing useful at root level */ }
            _ => {}
        }
        buf.clear();
    }

    Ok(trees)
}

/// Parse a `<BehaviorTree>` element, assuming the `<BehaviorTree>` start tag
/// has already been read into `start`.
fn parse_behavior_tree(
    reader: &mut Reader<&[u8]>,
    start: &BytesStart<'_>,
) -> Result<BtTree, ParseError> {
    let id = attr_value(start, "ID")?.ok_or(ParseError::BehaviorTreeMissingId)?;
    let mut next_id: u32 = 0;
    let mut root: Option<BtNode> = None;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Eof => return Err(ParseError::UnexpectedEof("BehaviorTree".to_string())),
            Event::End(e) if end_local_name(&e)? == "BehaviorTree" => break,
            Event::Start(e) => {
                if root.is_some() {
                    return Err(ParseError::Malformed(
                        "<BehaviorTree> has more than one root child".to_string(),
                    ));
                }
                let node = parse_element(reader, &e, &mut next_id, false)?;
                root = Some(node);
            }
            Event::Empty(e) => {
                if root.is_some() {
                    return Err(ParseError::Malformed(
                        "<BehaviorTree> has more than one root child".to_string(),
                    ));
                }
                let node = parse_element(reader, &e, &mut next_id, true)?;
                root = Some(node);
            }
            _ => {}
        }
        buf.clear();
    }

    let root = root.ok_or_else(|| ParseError::EmptyTree(id.clone()))?;
    Ok(BtTree { id, root })
}

/// Parse one element (start or empty) into a [`BtNode`], recursing into its
/// children if it is a non-empty start element.
fn parse_element(
    reader: &mut Reader<&[u8]>,
    start: &BytesStart<'_>,
    next_id: &mut u32,
    is_empty: bool,
) -> Result<BtNode, ParseError> {
    let raw_tag = local_name(start)?;
    let mut ports: HashMap<String, String> = HashMap::new();
    let mut display_name: Option<String> = None;
    let mut id_override: Option<String> = None;

    for attr_res in start.attributes() {
        let attr = attr_res?;
        let key = std::str::from_utf8(attr.key.as_ref())?.to_string();
        let value = attr.unescape_value()?.into_owned();
        match key.as_str() {
            "ID" => {
                // Groot-style: <Action ID="PubGoal" ...> — the registration
                // name lives on the ID attribute.
                id_override = Some(value);
            }
            "name" => {
                display_name = Some(value);
            }
            _ => {
                ports.insert(key, value);
            }
        }
    }

    // Promote ID-on-builtin-tag to registration name. Heuristic: BT.CPP's
    // categorical wrappers `Action`/`Condition`/`Decorator`/`Control`/`SubTree`
    // use `ID="..."` to carry the real registration. Other tags (`Sequence`,
    // `PubGoal`, ...) keep their own tag name as registration.
    let registration_name = match (id_override.as_deref(), raw_tag.as_str()) {
        (Some(id), "Action" | "Condition" | "Decorator" | "Control" | "SubTree") => id.to_string(),
        _ => raw_tag.clone(),
    };

    // If id_override applied but tag was not a wrapper, we still drop the ID
    // attribute (it's never a port). Already excluded above by the explicit
    // match arm; for non-wrapper tags ID is also discarded.
    let _ = id_override;

    let my_id = *next_id;
    *next_id += 1;

    let mut node = BtNode {
        id: my_id,
        registration_name,
        display_name,
        ports,
        children: Vec::new(),
    };

    if is_empty {
        return Ok(node);
    }

    // Recurse into children.
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Eof => return Err(ParseError::UnexpectedEof(raw_tag.clone())),
            Event::End(e) => {
                let end_name = end_local_name(&e)?;
                if end_name == raw_tag {
                    break;
                }
                // Tolerate stray mismatched end tag — but log via Malformed.
                return Err(ParseError::Malformed(format!(
                    "expected </{raw_tag}>, got </{end_name}>"
                )));
            }
            Event::Start(child) => {
                let c = parse_element(reader, &child, next_id, false)?;
                node.children.push(c);
            }
            Event::Empty(child) => {
                let c = parse_element(reader, &child, next_id, true)?;
                node.children.push(c);
            }
            Event::Comment(_) | Event::Text(_) | Event::CData(_) | Event::Decl(_)
            | Event::PI(_) | Event::DocType(_) => {
                // ignore
            }
        }
        buf.clear();
    }

    Ok(node)
}

/// Skip events until we see `</tag>`. Used to discard `<TreeNodesModel>`.
fn skip_to_end(reader: &mut Reader<&[u8]>, tag: &str) -> Result<(), ParseError> {
    let mut depth: i32 = 1;
    let mut buf = Vec::new();
    while depth > 0 {
        match reader.read_event_into(&mut buf)? {
            Event::Eof => return Err(ParseError::UnexpectedEof(tag.to_string())),
            Event::Start(e) => {
                if local_name(&e)? == tag {
                    depth += 1;
                }
            }
            Event::End(e) => {
                if end_local_name(&e)? == tag {
                    depth -= 1;
                }
            }
            _ => {}
        }
        buf.clear();
    }
    Ok(())
}

/// Local element name on a start tag (ignoring xml namespaces).
fn local_name(start: &BytesStart<'_>) -> Result<String, ParseError> {
    name_bytes_to_local(start.name().as_ref())
}

/// Local element name on an end tag.
fn end_local_name(end: &BytesEnd<'_>) -> Result<String, ParseError> {
    name_bytes_to_local(end.name().as_ref())
}

fn name_bytes_to_local(bytes: &[u8]) -> Result<String, ParseError> {
    let s = std::str::from_utf8(bytes)?;
    let local = s.split(':').next_back().unwrap_or(s);
    Ok(local.to_string())
}

/// Lookup a single attribute by key on a start tag.
fn attr_value(start: &BytesStart<'_>, key: &str) -> Result<Option<String>, ParseError> {
    for attr_res in start.attributes() {
        let attr = attr_res?;
        let k = std::str::from_utf8(attr.key.as_ref())?;
        if k == key {
            let v = attr.unescape_value()?.into_owned();
            return Ok(Some(v));
        }
    }
    Ok(None)
}
