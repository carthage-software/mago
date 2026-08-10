//! Domain-neutral primitives for stable extension payloads.
//!
//! Integers use network byte order. Byte strings and UTF-8 strings are
//! prefixed by their length as a `u32`. Capability-specific crates are
//! responsible for defining message headers, tags, limits, and validation.

#![allow(clippy::big_endian_bytes, reason = "network byte order is part of the stable extension wire format")]

use crate::error::PayloadError;

/// A writer for stable binary extension payloads.
#[derive(Debug, Default)]
pub struct PayloadWriter {
    bytes: Vec<u8>,
}

impl PayloadWriter {
    /// Creates an empty payload writer.
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    /// Creates an empty payload writer with space for at least `capacity` bytes.
    #[must_use]
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self { bytes: Vec::with_capacity(capacity) }
    }

    /// Appends bytes without a length prefix.
    #[inline]
    pub fn write_raw(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
    }

    #[inline]
    pub fn write_u8(&mut self, value: u8) {
        self.bytes.push(value);
    }

    #[inline]
    pub fn write_u16(&mut self, value: u16) {
        self.bytes.extend_from_slice(&value.to_be_bytes());
    }

    #[inline]
    pub fn write_u32(&mut self, value: u32) {
        self.bytes.extend_from_slice(&value.to_be_bytes());
    }

    #[inline]
    pub fn write_u64(&mut self, value: u64) {
        self.bytes.extend_from_slice(&value.to_be_bytes());
    }

    #[inline]
    pub fn write_bool(&mut self, value: bool) {
        self.write_u8(u8::from(value));
    }

    /// Writes a collection length as a `u32`.
    ///
    /// # Errors
    ///
    /// Returns an error when `length` does not fit in a `u32`.
    #[inline]
    pub fn write_length(&mut self, length: usize) -> Result<(), PayloadError> {
        let length = u32::try_from(length).map_err(|_| PayloadError::LengthOverflow { length })?;
        self.write_u32(length);
        Ok(())
    }

    /// Writes length-prefixed bytes.
    ///
    /// # Errors
    ///
    /// Returns an error when the byte length does not fit in a `u32`.
    #[inline]
    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), PayloadError> {
        self.write_length(bytes.len())?;
        self.write_raw(bytes);
        Ok(())
    }

    /// Writes a length-prefixed UTF-8 string.
    ///
    /// # Errors
    ///
    /// Returns an error when the encoded byte length does not fit in a `u32`.
    #[inline]
    pub fn write_string(&mut self, value: &str) -> Result<(), PayloadError> {
        self.write_bytes(value.as_bytes())
    }

    /// Writes an optional length-prefixed UTF-8 string.
    ///
    /// # Errors
    ///
    /// Returns an error when the encoded byte length does not fit in a `u32`.
    #[inline]
    pub fn write_optional_string(&mut self, value: Option<&str>) -> Result<(), PayloadError> {
        self.write_bool(value.is_some());
        if let Some(value) = value {
            self.write_string(value)?;
        }

        Ok(())
    }

    /// Returns the completed payload.
    #[must_use]
    #[inline]
    pub fn finish(self) -> Vec<u8> {
        self.bytes
    }
}

/// A zero-copy reader for stable binary extension payloads.
#[derive(Debug)]
pub struct PayloadReader<'payload> {
    payload: &'payload [u8],
    offset: usize,
}

impl<'payload> PayloadReader<'payload> {
    #[must_use]
    #[inline]
    pub const fn new(payload: &'payload [u8]) -> Self {
        Self { payload, offset: 0 }
    }

    /// Reads exactly `length` bytes without a length prefix.
    ///
    /// # Errors
    ///
    /// Returns an error when the requested range overflows or the payload ends.
    #[inline]
    pub fn read_raw(&mut self, length: usize, field: &'static str) -> Result<&'payload [u8], PayloadError> {
        let end = self.offset.checked_add(length).ok_or(PayloadError::OffsetOverflow { field })?;
        let bytes = self.payload.get(self.offset..end).ok_or(PayloadError::UnexpectedEnd {
            field,
            needed: length,
            remaining: self.remaining(),
        })?;
        self.offset = end;
        Ok(bytes)
    }

    /// Reads a fixed-size byte array without a length prefix.
    ///
    /// # Errors
    ///
    /// Returns an error when the payload does not contain `N` more bytes.
    #[inline]
    pub fn read_array<const N: usize>(&mut self, field: &'static str) -> Result<[u8; N], PayloadError> {
        let mut value = [0; N];
        value.copy_from_slice(self.read_raw(N, field)?);
        Ok(value)
    }

    /// Reads an unsigned byte.
    ///
    /// # Errors
    ///
    /// Returns an error when the payload has no byte remaining.
    #[inline]
    pub fn read_u8(&mut self, field: &'static str) -> Result<u8, PayloadError> {
        Ok(self.read_array::<1>(field)?[0])
    }

    /// Reads a big-endian `u16`.
    ///
    /// # Errors
    ///
    /// Returns an error when the payload has fewer than two bytes remaining.
    #[inline]
    pub fn read_u16(&mut self, field: &'static str) -> Result<u16, PayloadError> {
        Ok(u16::from_be_bytes(self.read_array(field)?))
    }

    /// Reads a big-endian `u32`.
    ///
    /// # Errors
    ///
    /// Returns an error when the payload has fewer than four bytes remaining.
    #[inline]
    pub fn read_u32(&mut self, field: &'static str) -> Result<u32, PayloadError> {
        Ok(u32::from_be_bytes(self.read_array(field)?))
    }

    /// Reads a big-endian `u64`.
    ///
    /// # Errors
    ///
    /// Returns an error when the payload has fewer than eight bytes remaining.
    #[inline]
    pub fn read_u64(&mut self, field: &'static str) -> Result<u64, PayloadError> {
        Ok(u64::from_be_bytes(self.read_array(field)?))
    }

    /// Reads a boolean encoded as zero or one.
    ///
    /// # Errors
    ///
    /// Returns an error when the payload ends or contains another value.
    #[inline]
    pub fn read_bool(&mut self, field: &'static str) -> Result<bool, PayloadError> {
        match self.read_u8(field)? {
            0 => Ok(false),
            1 => Ok(true),
            value => Err(PayloadError::InvalidBoolean { field, value }),
        }
    }

    /// Reads a collection length and validates it against `maximum`.
    ///
    /// # Errors
    ///
    /// Returns an error when the payload ends or the decoded count is too large.
    #[inline]
    pub fn read_count(&mut self, field: &'static str, maximum: usize) -> Result<usize, PayloadError> {
        let count = self.read_u32(field)? as usize;
        if count > maximum {
            return Err(PayloadError::CountExceeded { field, count, maximum });
        }

        Ok(count)
    }

    /// Reads length-prefixed bytes without copying them.
    ///
    /// # Errors
    ///
    /// Returns an error when the payload ends before the byte string does.
    #[inline]
    pub fn read_bytes(&mut self, field: &'static str) -> Result<&'payload [u8], PayloadError> {
        let length = self.read_u32(field)? as usize;
        self.read_raw(length, field)
    }

    /// Reads a length-prefixed UTF-8 string without copying it.
    ///
    /// # Errors
    ///
    /// Returns an error when the payload ends or the bytes are not valid UTF-8.
    #[inline]
    pub fn read_str(&mut self, field: &'static str) -> Result<&'payload str, PayloadError> {
        std::str::from_utf8(self.read_bytes(field)?).map_err(|_| PayloadError::InvalidUtf8 { field })
    }

    /// Reads an owned length-prefixed UTF-8 string.
    ///
    /// # Errors
    ///
    /// Returns an error when the payload ends or the bytes are not valid UTF-8.
    #[inline]
    pub fn read_string(&mut self, field: &'static str) -> Result<String, PayloadError> {
        self.read_str(field).map(str::to_owned)
    }

    /// Reads an optional owned length-prefixed UTF-8 string.
    ///
    /// # Errors
    ///
    /// Returns an error when its presence flag or string encoding is invalid.
    #[inline]
    pub fn read_optional_string(&mut self, field: &'static str) -> Result<Option<String>, PayloadError> {
        if self.read_bool(field)? { self.read_string(field).map(Some) } else { Ok(None) }
    }

    /// Validates that the entire payload was consumed.
    ///
    /// # Errors
    ///
    /// Returns an error when unread bytes remain.
    #[inline]
    pub fn finish(self) -> Result<(), PayloadError> {
        let remaining = self.remaining();
        if remaining == 0 { Ok(()) } else { Err(PayloadError::TrailingBytes { remaining }) }
    }

    #[must_use]
    #[inline]
    pub const fn remaining(&self) -> usize {
        self.payload.len() - self.offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_payload_primitives() {
        let mut writer = PayloadWriter::new();
        writer.write_u8(1);
        writer.write_u16(0x0203);
        writer.write_u32(0x0405_0607);
        writer.write_u64(0x0809_0a0b_0c0d_0e0f);
        writer.write_bool(false);
        writer.write_bytes(b"bytes").unwrap();
        writer.write_string("string").unwrap();
        writer.write_optional_string(Some("optional")).unwrap();
        writer.write_optional_string(None).unwrap();

        let payload = writer.finish();
        let mut reader = PayloadReader::new(&payload);
        assert_eq!(reader.read_u8("u8").unwrap(), 1);
        assert_eq!(reader.read_u16("u16").unwrap(), 0x0203);
        assert_eq!(reader.read_u32("u32").unwrap(), 0x0405_0607);
        assert_eq!(reader.read_u64("u64").unwrap(), 0x0809_0a0b_0c0d_0e0f);
        assert!(!reader.read_bool("bool").unwrap());
        assert_eq!(reader.read_bytes("bytes").unwrap(), b"bytes");
        assert_eq!(reader.read_str("string").unwrap(), "string");
        assert_eq!(reader.read_optional_string("optional").unwrap().as_deref(), Some("optional"));
        assert_eq!(reader.read_optional_string("optional").unwrap(), None);
        reader.finish().unwrap();
    }

    #[test]
    fn rejects_invalid_and_truncated_payloads() {
        let mut reader = PayloadReader::new(&[2]);
        assert!(matches!(reader.read_bool("flag"), Err(PayloadError::InvalidBoolean { field: "flag", value: 2 })));

        let mut reader = PayloadReader::new(&[0, 0, 0, 2, 0xff, 0xff]);
        assert!(matches!(reader.read_str("text"), Err(PayloadError::InvalidUtf8 { field: "text" })));

        let mut reader = PayloadReader::new(&[0, 0, 0, 2, 1]);
        assert!(matches!(reader.read_bytes("bytes"), Err(PayloadError::UnexpectedEnd { field: "bytes", .. })));
    }

    #[test]
    fn enforces_counts_and_complete_consumption() {
        let mut reader = PayloadReader::new(&[0, 0, 0, 2]);
        assert!(matches!(
            reader.read_count("items", 1),
            Err(PayloadError::CountExceeded { field: "items", count: 2, maximum: 1 })
        ));

        let reader = PayloadReader::new(&[1]);
        assert!(matches!(reader.finish(), Err(PayloadError::TrailingBytes { remaining: 1 })));
    }
}
