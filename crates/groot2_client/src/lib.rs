//! ZMQ client for [BehaviorTree.CPP](https://github.com/BehaviorTree/BehaviorTree.CPP)
//! `BT::Groot2Publisher`.
//!
//! The publisher exposes a ZMQ REP socket (default port `1667`) speaking a
//! small custom binary protocol described in
//! `behaviortree_cpp/loggers/groot2_protocol.h`. Each request is a single
//! ZMQ frame containing a 6-byte header; the corresponding reply is a
//! multipart message whose first frame is a 22-byte echo header (request
//! header + 16-byte tree UUID) followed by request-specific payloads.
//!
//! This crate covers the read-only parts needed by a visualizer:
//!
//! - [`Groot2Client::request_full_tree`] — wire `'T'` (FULLTREE), returns the
//!   serialized BT XML
//! - [`Groot2Client::request_status_map`] — wire `'S'` (STATUS), returns a
//!   `node_uid -> NodeStatus` snapshot
//!
//! Hooks, breakpoints, blackboard reads, and the recording stream are not
//! implemented; their opcodes are exposed in [`RequestType`] for extension.

pub mod client;
pub mod protocol;

pub use client::{ClientError, Groot2Client, NodeStatus};
pub use protocol::{ProtocolError, ReplyHeader, RequestHeader, RequestType, HEADER_SIZE,
                   PROTOCOL_ID, REPLY_HEADER_SIZE};
