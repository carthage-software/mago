#![allow(clippy::big_endian_bytes, reason = "network byte order is part of the stable extension frame format")]

use std::io::Read;
use std::io::Write;

use crate::error::ProtocolError;

/// Magic bytes at the beginning of every extension protocol frame.
pub const FRAME_MAGIC: [u8; 4] = *b"MAGO";

/// Number of bytes in the fixed extension frame header.
pub const FRAME_HEADER_LENGTH: usize = 32;

const RESERVED: u16 = 0;

/// The extension wire protocol version implemented by this Mago build.
pub const PROTOCOL_VERSION: ProtocolVersion = ProtocolVersion { major: 1, minor: 0 };

/// A version of the stable extension wire protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProtocolVersion {
    /// Incompatible protocol revision.
    pub major: u16,
    /// Backward-compatible protocol revision.
    pub minor: u16,
}

/// The role of an extension protocol frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FrameKind {
    Request = 1,
    Response = 2,
    Notification = 3,
    Cancel = 4,
    Shutdown = 5,
}

impl TryFrom<u8> for FrameKind {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Request),
            2 => Ok(Self::Response),
            3 => Ok(Self::Notification),
            4 => Ok(Self::Cancel),
            5 => Ok(Self::Shutdown),
            unknown => Err(ProtocolError::UnknownFrameKind(unknown)),
        }
    }
}

/// Flags attached to a protocol frame.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FrameFlags(u8);

impl FrameFlags {
    /// The response contains a remote error payload instead of a successful result.
    pub const ERROR: Self = Self(1 << 0);

    #[must_use]
    pub const fn empty() -> Self {
        Self(0)
    }

    #[must_use]
    pub const fn from_bits(bits: u8) -> Self {
        Self(bits)
    }

    #[must_use]
    pub const fn bits(self) -> u8 {
        self.0
    }

    #[must_use]
    pub const fn contains(self, flag: Self) -> bool {
        self.0 & flag.0 == flag.0
    }
}

/// One framed protocol message.
///
/// Payloads are opaque here. Linter and analyzer protocol layers define their
/// own stable binary payloads without coupling process management to either.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    /// Protocol version used to encode this frame.
    pub version: ProtocolVersion,
    /// The role of this frame.
    pub kind: FrameKind,
    /// Frame-level status and feature flags.
    pub flags: FrameFlags,
    /// Request identifier, or the identifier being answered for a response.
    pub id: u64,
    /// The containing host request for a nested worker request, otherwise zero.
    pub parent_id: u64,
    /// Domain-specific binary data.
    pub payload: Vec<u8>,
}

impl Frame {
    #[must_use]
    pub fn request(id: u64, payload: Vec<u8>) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            kind: FrameKind::Request,
            flags: FrameFlags::empty(),
            id,
            parent_id: 0,
            payload,
        }
    }

    #[must_use]
    pub fn response(id: u64, parent_id: u64, payload: Vec<u8>) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            kind: FrameKind::Response,
            flags: FrameFlags::empty(),
            id,
            parent_id,
            payload,
        }
    }

    #[must_use]
    pub fn error(id: u64, parent_id: u64, payload: Vec<u8>) -> Self {
        Self { version: PROTOCOL_VERSION, kind: FrameKind::Response, flags: FrameFlags::ERROR, id, parent_id, payload }
    }

    #[must_use]
    pub fn cancel(id: u64) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            kind: FrameKind::Cancel,
            flags: FrameFlags::empty(),
            id,
            parent_id: 0,
            payload: Vec::new(),
        }
    }

    #[must_use]
    pub fn shutdown() -> Self {
        Self {
            version: PROTOCOL_VERSION,
            kind: FrameKind::Shutdown,
            flags: FrameFlags::empty(),
            id: 0,
            parent_id: 0,
            payload: Vec::new(),
        }
    }

    /// Writes this frame to `writer`.
    ///
    /// # Errors
    ///
    /// Returns an error if the payload exceeds the configured limit or the
    /// underlying writer fails.
    pub fn write_to(&self, writer: &mut impl Write, maximum_payload_size: usize) -> Result<(), ProtocolError> {
        let payload_length = self.payload.len();
        if payload_length > maximum_payload_size || payload_length > u32::MAX as usize {
            return Err(ProtocolError::FrameTooLarge { length: payload_length, maximum: maximum_payload_size });
        }

        let mut header = [0u8; FRAME_HEADER_LENGTH];
        header[0..4].copy_from_slice(&FRAME_MAGIC);
        header[4..6].copy_from_slice(&self.version.major.to_be_bytes());
        header[6..8].copy_from_slice(&self.version.minor.to_be_bytes());
        header[8] = self.kind as u8;
        header[9] = self.flags.bits();
        header[10..12].copy_from_slice(&RESERVED.to_be_bytes());
        header[12..20].copy_from_slice(&self.id.to_be_bytes());
        header[20..28].copy_from_slice(&self.parent_id.to_be_bytes());
        header[28..32].copy_from_slice(&(payload_length as u32).to_be_bytes());

        writer.write_all(&header)?;
        writer.write_all(&self.payload)?;

        Ok(())
    }

    /// Reads one frame, returning `None` only for a clean EOF between frames.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid or incompatible headers, oversized
    /// payloads, truncated frames, and failures from the underlying reader.
    pub fn read_from(reader: &mut impl Read, maximum_payload_size: usize) -> Result<Option<Self>, ProtocolError> {
        let mut header = [0u8; FRAME_HEADER_LENGTH];
        let read = reader.read(&mut header[..1])?;
        if read == 0 {
            return Ok(None);
        }

        reader.read_exact(&mut header[1..])?;

        let magic = [header[0], header[1], header[2], header[3]];
        if magic != FRAME_MAGIC {
            return Err(ProtocolError::InvalidMagic(magic));
        }

        let version = ProtocolVersion {
            major: u16::from_be_bytes([header[4], header[5]]),
            minor: u16::from_be_bytes([header[6], header[7]]),
        };
        if version.major != PROTOCOL_VERSION.major {
            return Err(ProtocolError::UnsupportedVersion(version));
        }

        let reserved = u16::from_be_bytes([header[10], header[11]]);
        if reserved != 0 {
            return Err(ProtocolError::ReservedHeaderBits(reserved));
        }

        let kind = FrameKind::try_from(header[8])?;
        let flags = FrameFlags::from_bits(header[9]);
        let id = u64::from_be_bytes([
            header[12], header[13], header[14], header[15], header[16], header[17], header[18], header[19],
        ]);
        let parent_id = u64::from_be_bytes([
            header[20], header[21], header[22], header[23], header[24], header[25], header[26], header[27],
        ]);
        let payload_length = u32::from_be_bytes([header[28], header[29], header[30], header[31]]) as usize;

        if payload_length > maximum_payload_size {
            return Err(ProtocolError::FrameTooLarge { length: payload_length, maximum: maximum_payload_size });
        }

        let mut payload = vec![0u8; payload_length];
        reader.read_exact(&mut payload)?;

        Ok(Some(Self { version, kind, flags, id, parent_id, payload }))
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn round_trips_a_frame() {
        let expected = Frame {
            version: PROTOCOL_VERSION,
            kind: FrameKind::Request,
            flags: FrameFlags::from_bits(0b1010_0001),
            id: 42,
            parent_id: 7,
            payload: vec![0, 1, 2, 0xff],
        };
        let mut bytes = Vec::new();

        expected.write_to(&mut bytes, 1024).expect("frame should encode");
        let actual = Frame::read_from(&mut Cursor::new(bytes), 1024)
            .expect("frame should decode")
            .expect("one frame should be present");

        assert_eq!(actual, expected);
    }

    #[test]
    fn distinguishes_clean_eof_from_a_truncated_header() {
        assert!(Frame::read_from(&mut Cursor::new([]), 1024).expect("clean EOF should decode").is_none());

        let error = Frame::read_from(&mut Cursor::new(*b"M"), 1024).expect_err("truncated frame should fail");
        assert!(matches!(error, ProtocolError::Io(error) if error.kind() == std::io::ErrorKind::UnexpectedEof));
    }

    #[test]
    fn rejects_an_incompatible_major_version() {
        let mut frame = Frame::request(1, Vec::new());
        frame.version.major += 1;
        let mut bytes = Vec::new();
        frame.write_to(&mut bytes, 1024).expect("frame should encode");

        let error = Frame::read_from(&mut Cursor::new(bytes), 1024).expect_err("version should be rejected");
        assert!(matches!(error, ProtocolError::UnsupportedVersion(version) if version == frame.version));
    }

    #[test]
    fn rejects_an_oversized_payload_before_allocating_it() {
        let frame = Frame::request(1, vec![0; 16]);
        let mut bytes = Vec::new();
        frame.write_to(&mut bytes, 16).expect("frame should encode");

        let error = Frame::read_from(&mut Cursor::new(bytes), 8).expect_err("payload should be rejected");
        assert!(matches!(error, ProtocolError::FrameTooLarge { length: 16, maximum: 8 }));
    }
}
