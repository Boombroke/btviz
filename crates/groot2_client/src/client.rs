//! High-level client wrapping the ZMQ REQ socket.
//!
//! Each call opens a fresh `ZMQ_REQ` socket on the shared context, sends
//! the request, and reads the multipart reply. This keeps things stateless
//! at the cost of one extra `connect()` per call, which is fine for a UI
//! polling at <100 Hz against a local publisher.
//!
//! ## STATUS payload format
//!
//! The publisher serializes the snapshot as a packed byte stream:
//! ```text
//! repeated:
//!   u16 LE  uid
//!   u8      status_code
//! ```
//! Status codes:
//! - `0` IDLE
//! - `1` RUNNING
//! - `2` SUCCESS
//! - `3` FAILURE
//! - `4` SKIPPED
//!
//! When a node transitions back to IDLE, the publisher encodes the prior
//! status by adding `10` to that status (`12 = IDLE_FROM_SUCCESS`,
//! `13 = IDLE_FROM_FAILURE`, `11 = IDLE_FROM_RUNNING`). [`NodeStatus::decode`]
//! normalizes these back to [`NodeStatus::Idle`]; callers that care about
//! the prior status can use [`NodeStatus::decode_with_prior`].

use std::collections::HashMap;
use std::time::Duration;

use byteorder::{ByteOrder, LittleEndian};
use thiserror::Error;

use crate::protocol::{ProtocolError, ReplyHeader, RequestHeader, RequestType, REPLY_HEADER_SIZE};

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("zmq error: {0}")]
    Zmq(#[from] zmq::Error),
    #[error("protocol error: {0}")]
    Protocol(#[from] ProtocolError),
    #[error("server returned an error reply: {0}")]
    ServerError(String),
    #[error("incomplete reply (missing payload frame)")]
    IncompleteReply,
    #[error("status payload not a multiple of 3 bytes: got {0}")]
    BadStatusPayload(usize),
    #[error("invalid utf-8 in tree XML payload")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),
}

/// Node lifecycle status as exposed on the wire by `BT::Groot2Publisher`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeStatus {
    Idle,
    Running,
    Success,
    Failure,
    Skipped,
}

impl NodeStatus {
    /// Decode a wire byte. IDLE-with-prior codes (`>= 10`) collapse to
    /// [`NodeStatus::Idle`]; unknown bytes also map to Idle so a future
    /// protocol extension does not crash the visualizer.
    pub fn decode(byte: u8) -> Self {
        match byte {
            0 => Self::Idle,
            1 => Self::Running,
            2 => Self::Success,
            3 => Self::Failure,
            4 => Self::Skipped,
            // 10/11/12/13/14 = IDLE-after-X (BT.CPP encoding for transition tracking)
            10..=14 => Self::Idle,
            _ => Self::Idle,
        }
    }

    /// Decode a wire byte and additionally surface the prior status when the
    /// node has just transitioned to IDLE. Returns `(current, prior)`.
    pub fn decode_with_prior(byte: u8) -> (Self, Option<Self>) {
        match byte {
            10 => (Self::Idle, Some(Self::Idle)),
            11 => (Self::Idle, Some(Self::Running)),
            12 => (Self::Idle, Some(Self::Success)),
            13 => (Self::Idle, Some(Self::Failure)),
            14 => (Self::Idle, Some(Self::Skipped)),
            _ => (Self::decode(byte), None),
        }
    }
}

/// Read-only client for `BT::Groot2Publisher`.
///
/// The client does not maintain a persistent socket; every call connects,
/// REQ/REP-s, and disconnects. This keeps usage simple and trades a
/// negligible amount of throughput for resilience against publisher restarts
/// (no reconnect logic to maintain).
pub struct Groot2Client {
    ctx: zmq::Context,
    addr: String,
    timeout_ms: i32,
}

impl Groot2Client {
    pub fn new(addr: impl Into<String>) -> Self {
        Self {
            ctx: zmq::Context::new(),
            addr: addr.into(),
            timeout_ms: 1000,
        }
    }

    /// Convenience: connect to `tcp://127.0.0.1:{port}` (default Groot2 port
    /// is 1667).
    pub fn local(port: u16) -> Self {
        Self::new(format!("tcp://127.0.0.1:{port}"))
    }

    pub fn set_timeout(&mut self, dur: Duration) {
        self.timeout_ms = dur.as_millis().min(i32::MAX as u128) as i32;
    }

    /// Address as passed to `zmq::Socket::connect`.
    pub fn addr(&self) -> &str {
        &self.addr
    }

    fn round_trip(
        &self,
        req: RequestHeader,
    ) -> Result<(ReplyHeader, Option<Vec<u8>>), ClientError> {
        let sock = self.ctx.socket(zmq::REQ)?;
        // Use the configured timeout for both directions; LINGER=0 prevents the
        // socket from blocking on close if the server has gone away.
        sock.set_sndtimeo(self.timeout_ms)?;
        sock.set_rcvtimeo(self.timeout_ms)?;
        sock.set_linger(0)?;
        sock.connect(&self.addr)?;

        sock.send(&req.to_bytes()[..], 0)?;

        let frame1 = sock.recv_bytes(0)?;

        // The publisher emits an explicit error reply as two frames: `b"error"`
        // followed by a UTF-8 message. Detect that early so callers see a
        // useful error instead of a TooShort header decode.
        if frame1 == b"error" {
            let msg = if sock.get_rcvmore()? {
                let m = sock.recv_bytes(0)?;
                String::from_utf8_lossy(&m).into_owned()
            } else {
                String::new()
            };
            return Err(ClientError::ServerError(msg));
        }

        let header = ReplyHeader::from_bytes(&frame1)?;
        let payload = if sock.get_rcvmore()? {
            Some(sock.recv_bytes(0)?)
        } else {
            None
        };
        Ok((header, payload))
    }

    /// Wire `'T'`: ask the publisher for the full tree XML.
    pub fn request_full_tree(&self) -> Result<String, ClientError> {
        let req = RequestHeader::new(RequestType::FullTree);
        let (_header, payload) = self.round_trip(req)?;
        let bytes = payload.ok_or(ClientError::IncompleteReply)?;
        Ok(String::from_utf8(bytes)?)
    }

    /// Wire `'S'`: raw status snapshot bytes (3 bytes per node).
    pub fn request_status_raw(&self) -> Result<Vec<u8>, ClientError> {
        let req = RequestHeader::new(RequestType::Status);
        let (_header, payload) = self.round_trip(req)?;
        let bytes = payload.ok_or(ClientError::IncompleteReply)?;
        if bytes.len() % 3 != 0 {
            return Err(ClientError::BadStatusPayload(bytes.len()));
        }
        Ok(bytes)
    }

    /// Wire `'S'` decoded into a `node_uid -> status` map. Convenient for
    /// per-frame diffs against the previous snapshot.
    pub fn request_status_map(&self) -> Result<HashMap<u16, NodeStatus>, ClientError> {
        let bytes = self.request_status_raw()?;
        let mut map = HashMap::with_capacity(bytes.len() / 3);
        for chunk in bytes.chunks_exact(3) {
            let uid = LittleEndian::read_u16(&chunk[..2]);
            let status = NodeStatus::decode(chunk[2]);
            map.insert(uid, status);
        }
        Ok(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_helper_builds_loopback_addr() {
        let c = Groot2Client::local(1667);
        assert_eq!(c.addr(), "tcp://127.0.0.1:1667");
    }

    #[test]
    fn decode_idle_transition_codes() {
        assert_eq!(NodeStatus::decode(0), NodeStatus::Idle);
        assert_eq!(NodeStatus::decode(1), NodeStatus::Running);
        assert_eq!(NodeStatus::decode(2), NodeStatus::Success);
        assert_eq!(NodeStatus::decode(3), NodeStatus::Failure);
        assert_eq!(NodeStatus::decode(4), NodeStatus::Skipped);
        // IDLE-after-X — current always Idle, prior surfaces the transition
        for code in 10..=14 {
            assert_eq!(NodeStatus::decode(code), NodeStatus::Idle);
        }
        let (cur, prior) = NodeStatus::decode_with_prior(12);
        assert_eq!(cur, NodeStatus::Idle);
        assert_eq!(prior, Some(NodeStatus::Success));
    }
}
