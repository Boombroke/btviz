//! Wire format definitions matching `behaviortree_cpp/loggers/groot2_protocol.h`.
//!
//! Byte layout of `RequestHeader` (6 bytes):
//! ```text
//! offset 0: protocol  (u8)   == PROTOCOL_ID (2)
//! offset 1: type      (u8)   RequestType ASCII char
//! offset 2: unique_id (u32 LE)
//! ```
//!
//! `ReplyHeader` echoes the request header followed by the tree UUID
//! (16 bytes) for a total of 22 bytes:
//! ```text
//! offset 0..6: RequestHeader echo (protocol/type/unique_id)
//! offset 6..22: tree_uuid [u8; 16]
//! ```

use byteorder::{ByteOrder, LittleEndian};
use thiserror::Error;

/// Wire-level error parsing or producing protocol headers.
#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("buffer too short: need {expected} bytes, got {actual}")]
    TooShort { expected: usize, actual: usize },
    #[error("unknown request type byte: 0x{0:02x}")]
    UnknownType(u8),
    #[error("protocol id mismatch: expected {expected}, got {actual}")]
    BadProtocolId { expected: u8, actual: u8 },
}

/// Magic version byte the publisher expects (`kProtocolID = 2` in BT.CPP 4.9).
pub const PROTOCOL_ID: u8 = 2;

/// Size of [`RequestHeader`] on the wire.
pub const HEADER_SIZE: usize = 6;

/// Size of [`ReplyHeader`] on the wire (RequestHeader + tree UUID).
pub const REPLY_HEADER_SIZE: usize = HEADER_SIZE + 16;

/// Set of opcodes accepted by `BT::Groot2Publisher`.
///
/// Only [`RequestType::FullTree`] and [`RequestType::Status`] are used by the
/// read-only visualizer flow; the remaining variants are surfaced so callers
/// can extend the client without forking this crate.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestType {
    /// `'T'` — request the entire tree definition as XML.
    FullTree = b'T',
    /// `'S'` — request a snapshot of every node's current status.
    Status = b'S',
    /// `'B'` — read named blackboards.
    Blackboard = b'B',

    /// `'I'` — install a Groot debug hook.
    HookInsert = b'I',
    /// `'R'` — remove a single hook.
    HookRemove = b'R',
    /// `'N'` — server tells client a breakpoint was hit.
    BreakpointReached = b'N',
    /// `'U'` — client unlocks a paused breakpoint.
    BreakpointUnlock = b'U',
    /// `'D'` — dump existing hooks as JSON.
    HooksDump = b'D',
    /// `'A'` — clear all hooks (call before disconnect).
    RemoveAllHooks = b'A',
    /// `'X'` — temporarily disable all hooks.
    DisableAllHooks = b'X',

    /// `'r'` — toggle recording mode.
    ToggleRecording = b'r',
    /// `'t'` — fetch buffered transitions.
    GetTransitions = b't',
}

impl RequestType {
    /// Decode an opcode byte. Returns [`ProtocolError::UnknownType`] for
    /// unknown values so a caller can keep the connection alive instead of
    /// crashing on a future protocol extension.
    pub fn from_byte(b: u8) -> Result<Self, ProtocolError> {
        Ok(match b {
            b'T' => Self::FullTree,
            b'S' => Self::Status,
            b'B' => Self::Blackboard,
            b'I' => Self::HookInsert,
            b'R' => Self::HookRemove,
            b'N' => Self::BreakpointReached,
            b'U' => Self::BreakpointUnlock,
            b'D' => Self::HooksDump,
            b'A' => Self::RemoveAllHooks,
            b'X' => Self::DisableAllHooks,
            b'r' => Self::ToggleRecording,
            b't' => Self::GetTransitions,
            other => return Err(ProtocolError::UnknownType(other)),
        })
    }
}

/// Header sent by the client at the start of every request.
#[derive(Debug, Clone, Copy)]
pub struct RequestHeader {
    pub protocol: u8,
    pub r#type: RequestType,
    pub unique_id: u32,
}

impl RequestHeader {
    /// Build a request header with a fresh `unique_id` derived from the
    /// system clock. The publisher does not validate uniqueness; any non-zero
    /// id is fine, but echoing a unique value lets a multiplexed client
    /// pair replies with requests.
    pub fn new(t: RequestType) -> Self {
        let id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos() ^ d.as_secs() as u32)
            .unwrap_or(1);
        Self::with_id(t, id.max(1))
    }

    pub fn with_id(t: RequestType, unique_id: u32) -> Self {
        Self {
            protocol: PROTOCOL_ID,
            r#type: t,
            unique_id,
        }
    }

    pub fn to_bytes(&self) -> [u8; HEADER_SIZE] {
        let mut buf = [0u8; HEADER_SIZE];
        buf[0] = self.protocol;
        buf[1] = self.r#type as u8;
        LittleEndian::write_u32(&mut buf[2..6], self.unique_id);
        buf
    }

    pub fn from_bytes(b: &[u8]) -> Result<Self, ProtocolError> {
        if b.len() < HEADER_SIZE {
            return Err(ProtocolError::TooShort {
                expected: HEADER_SIZE,
                actual: b.len(),
            });
        }
        let protocol = b[0];
        if protocol != PROTOCOL_ID {
            return Err(ProtocolError::BadProtocolId {
                expected: PROTOCOL_ID,
                actual: protocol,
            });
        }
        let r#type = RequestType::from_byte(b[1])?;
        let unique_id = LittleEndian::read_u32(&b[2..6]);
        Ok(Self {
            protocol,
            r#type,
            unique_id,
        })
    }
}

/// First frame of a successful reply.
///
/// The publisher prepends [`HEADER_SIZE`] bytes of request echo plus a
/// 16-byte `tree_uuid`. Visualizers may safely ignore `tree_uuid` for a
/// single-tree session, but it is exposed for completeness.
#[derive(Debug, Clone, Copy)]
pub struct ReplyHeader {
    pub request: RequestHeader,
    pub tree_uuid: [u8; 16],
}

impl ReplyHeader {
    pub fn from_bytes(b: &[u8]) -> Result<Self, ProtocolError> {
        if b.len() < REPLY_HEADER_SIZE {
            return Err(ProtocolError::TooShort {
                expected: REPLY_HEADER_SIZE,
                actual: b.len(),
            });
        }
        let request = RequestHeader::from_bytes(&b[..HEADER_SIZE])?;
        let mut tree_uuid = [0u8; 16];
        tree_uuid.copy_from_slice(&b[HEADER_SIZE..REPLY_HEADER_SIZE]);
        Ok(Self {
            request,
            tree_uuid,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_request_header() {
        let h = RequestHeader::with_id(RequestType::Status, 0xDEADBEEF);
        let bytes = h.to_bytes();
        // Wire layout: [protocol, type, id_le_0..3]
        assert_eq!(bytes[0], PROTOCOL_ID);
        assert_eq!(bytes[1], b'S');
        assert_eq!(&bytes[2..6], &0xDEADBEEFu32.to_le_bytes());

        let parsed = RequestHeader::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.protocol, PROTOCOL_ID);
        assert_eq!(parsed.r#type, RequestType::Status);
        assert_eq!(parsed.unique_id, 0xDEADBEEF);
    }

    #[test]
    fn rejects_short_buffer() {
        let err = RequestHeader::from_bytes(&[2, b'T']).unwrap_err();
        match err {
            ProtocolError::TooShort { expected, actual } => {
                assert_eq!(expected, HEADER_SIZE);
                assert_eq!(actual, 2);
            }
            _ => panic!("unexpected error: {err:?}"),
        }
    }

    #[test]
    fn rejects_wrong_protocol() {
        let bytes = [99, b'T', 1, 0, 0, 0];
        let err = RequestHeader::from_bytes(&bytes).unwrap_err();
        assert!(matches!(err, ProtocolError::BadProtocolId { actual: 99, .. }));
    }

    #[test]
    fn rejects_unknown_type() {
        let bytes = [PROTOCOL_ID, b'?', 1, 0, 0, 0];
        let err = RequestHeader::from_bytes(&bytes).unwrap_err();
        assert!(matches!(err, ProtocolError::UnknownType(b'?')));
    }

    #[test]
    fn parses_reply_header() {
        let mut buf = [0u8; REPLY_HEADER_SIZE];
        let req = RequestHeader::with_id(RequestType::FullTree, 7);
        buf[..HEADER_SIZE].copy_from_slice(&req.to_bytes());
        // tree uuid: a recognizable pattern
        for i in 0..16 {
            buf[HEADER_SIZE + i] = i as u8;
        }
        let r = ReplyHeader::from_bytes(&buf).unwrap();
        assert_eq!(r.request.unique_id, 7);
        assert_eq!(r.request.r#type, RequestType::FullTree);
        assert_eq!(r.tree_uuid[0], 0);
        assert_eq!(r.tree_uuid[15], 15);
    }
}
