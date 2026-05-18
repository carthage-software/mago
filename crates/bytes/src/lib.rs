//! Shared byte-slice utilities for the Mago toolchain.
//!
//! PHP source code is binary-safe, so the toolchain handles identifiers, comments, and
//! string literals as `&[u8]` end-to-end. Diagnostic messages and human-facing logs are
//! UTF-8 strings, so a tiny adapter layer is needed at the display boundary. This crate
//! is that adapter, plus a few SIMD-accelerated byte-trimming primitives.

#![deny(unsafe_op_in_unsafe_fn)]

use std::fmt;

#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::vceqq_u8;
#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::vdupq_n_u8;
#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::vld1q_u8;
#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::vminvq_u8;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::__m128i;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::_mm_cmpeq_epi8;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::_mm_loadu_si128;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::_mm_movemask_epi8;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::_mm_set1_epi8;

/// Writes `bytes` to `f`, rendering valid UTF-8 verbatim and every byte that is not part
/// of a valid UTF-8 sequence as a `\xHH` hex escape.
///
/// PHP source is binary-safe, so identifiers, comments, and string literals can carry
/// arbitrary bytes. When such content reaches a diagnostic message, lossy decoding would
/// collapse every stray byte to `U+FFFD` — unreadable, and ambiguous (two distinct byte
/// sequences look identical). Escaping instead keeps the output readable *and* lossless:
/// `Caf` followed by the bytes `C9 E9 FF` renders as `Caf\xC9\xE9\xFF`.
///
/// # Errors
///
/// Returns any error produced by the underlying [`fmt::Formatter`] while writing.
pub fn write_escaped(f: &mut fmt::Formatter<'_>, bytes: &[u8]) -> fmt::Result {
    let mut rest = bytes;
    while !rest.is_empty() {
        match std::str::from_utf8(rest) {
            Ok(valid) => {
                f.write_str(valid)?;
                break;
            }
            Err(error) => {
                let valid_up_to = error.valid_up_to();
                if valid_up_to > 0 {
                    // SAFETY: `valid_up_to` is the length of the longest valid UTF-8 prefix,
                    // per the `Utf8Error` contract.
                    f.write_str(unsafe { std::str::from_utf8_unchecked(&rest[..valid_up_to]) })?;
                }

                // `error_len()` is `None` when the input ends mid-sequence; in that case the
                // whole remaining tail is unconvertible.
                let invalid_len = error.error_len().unwrap_or(rest.len() - valid_up_to);
                for &byte in &rest[valid_up_to..valid_up_to + invalid_len] {
                    write!(f, "\\x{byte:02X}")?;
                }

                rest = &rest[valid_up_to + invalid_len..];
            }
        }
    }

    Ok(())
}

/// Renders a byte slice as text for diagnostic messages.
///
/// Valid UTF-8 is shown verbatim; bytes that are not valid UTF-8 are escaped as `\xHH`.
/// Use this in `format!`/`write!`/`println!` whenever a `&[u8]` needs to surface in
/// user-facing output (issue messages, log lines, error reports).
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct BytesDisplay<'src>(pub &'src [u8]);

impl fmt::Display for BytesDisplay<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_escaped(f, self.0)
    }
}

/// Strips all leading occurrences of `byte` from `s`.
///
/// SIMD-accelerated on x86_64 (SSE2, 16-byte chunks) and aarch64 (NEON, 16-byte chunks);
/// scalar elsewhere. SSE2 and NEON are baseline on their respective targets, so no
/// runtime feature detection is required.
#[inline]
#[must_use]
pub fn trim_start_byte(s: &[u8], byte: u8) -> &[u8] {
    let mut i = simd_skip_leading(s, byte);
    while i < s.len() && s[i] == byte {
        i += 1;
    }
    &s[i..]
}

/// Strips all trailing occurrences of `byte` from `s`.
///
/// SIMD-accelerated on x86_64 (SSE2, 16-byte chunks) and aarch64 (NEON, 16-byte chunks);
/// scalar elsewhere.
#[inline]
#[must_use]
pub fn trim_end_byte(s: &[u8], byte: u8) -> &[u8] {
    let mut end = simd_skip_trailing(s, byte);
    while end > 0 && s[end - 1] == byte {
        end -= 1;
    }
    &s[..end]
}

/// Strips all leading and trailing occurrences of `byte` from `s`.
#[inline]
#[must_use]
pub fn trim_byte(s: &[u8], byte: u8) -> &[u8] {
    trim_end_byte(trim_start_byte(s, byte), byte)
}

/// Returns the byte index past the last all-matching SIMD chunk at the start of `s`, or
/// the exact byte index of the first non-matching byte when the SIMD path can locate it.
/// The scalar tail in the caller picks up from this position.
#[cfg(target_arch = "x86_64")]
#[inline]
fn simd_skip_leading(s: &[u8], byte: u8) -> usize {
    let mut i = 0;
    // SAFETY: SSE2 is baseline on x86_64; the loop guard `i + 16 <= s.len()` keeps every
    // unaligned 16-byte load inside the slice's allocation.
    #[allow(clippy::multiple_unsafe_ops_per_block)]
    unsafe {
        let target = _mm_set1_epi8(byte as i8);
        while i + 16 <= s.len() {
            let chunk = _mm_loadu_si128(s.as_ptr().add(i).cast::<__m128i>());
            let eq = _mm_cmpeq_epi8(chunk, target);
            let mask = _mm_movemask_epi8(eq) as u32;
            if mask == 0xFFFF {
                i += 16;
                continue;
            }
            return i + mask.trailing_ones() as usize;
        }
    }
    i
}

#[cfg(target_arch = "aarch64")]
#[inline]
fn simd_skip_leading(s: &[u8], byte: u8) -> usize {
    let mut i = 0;
    // SAFETY: NEON is baseline on aarch64; the loop guard `i + 16 <= s.len()` keeps every
    // unaligned 16-byte load inside the slice's allocation.
    #[allow(clippy::multiple_unsafe_ops_per_block)]
    unsafe {
        let target = vdupq_n_u8(byte);
        while i + 16 <= s.len() {
            let chunk = vld1q_u8(s.as_ptr().add(i));
            let eq = vceqq_u8(chunk, target);
            // Reduce-min across lanes: 0xFF iff every lane matched.
            if vminvq_u8(eq) != 0xFF {
                break;
            }
            i += 16;
        }
    }
    i
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
#[inline]
fn simd_skip_leading(_s: &[u8], _byte: u8) -> usize {
    0
}

/// Returns the byte index of the first non-matching byte from the end, or the index past
/// the last all-matching SIMD chunk near the end. The caller's scalar loop trims any
/// remaining matching bytes.
#[cfg(target_arch = "x86_64")]
#[inline]
fn simd_skip_trailing(s: &[u8], byte: u8) -> usize {
    let mut end = s.len();
    // SAFETY: SSE2 is baseline on x86_64; `end >= 16` keeps every load at `end - 16`
    // inside the slice.
    #[allow(clippy::multiple_unsafe_ops_per_block)]
    unsafe {
        let target = _mm_set1_epi8(byte as i8);
        while end >= 16 {
            let start = end - 16;
            let chunk = _mm_loadu_si128(s.as_ptr().add(start).cast::<__m128i>());
            let eq = _mm_cmpeq_epi8(chunk, target);
            let mask = _mm_movemask_epi8(eq) as u32;
            if mask == 0xFFFF {
                end = start;
                continue;
            }
            // Shift the 16-bit mask into the high half of u32 so `leading_ones` measures
            // from the highest lane; the count of leading 1s is the number of trailing
            // matching bytes in this chunk.
            let mask_hi = mask << 16;
            return start + 16 - mask_hi.leading_ones() as usize;
        }
    }
    end
}

#[cfg(target_arch = "aarch64")]
#[inline]
fn simd_skip_trailing(s: &[u8], byte: u8) -> usize {
    let mut end = s.len();
    // SAFETY: NEON is baseline on aarch64; `end >= 16` keeps the load at `end - 16` inside
    // the slice.
    #[allow(clippy::multiple_unsafe_ops_per_block)]
    unsafe {
        let target = vdupq_n_u8(byte);
        while end >= 16 {
            let start = end - 16;
            let chunk = vld1q_u8(s.as_ptr().add(start));
            let eq = vceqq_u8(chunk, target);
            if vminvq_u8(eq) != 0xFF {
                break;
            }
            end = start;
        }
    }
    end
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
#[inline]
fn simd_skip_trailing(s: &[u8], _byte: u8) -> usize {
    s.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn escaped(bytes: &[u8]) -> String {
        BytesDisplay(bytes).to_string()
    }

    #[test]
    fn escape_pure_ascii_is_verbatim() {
        assert_eq!(escaped(b""), "");
        assert_eq!(escaped(b"hello"), "hello");
        assert_eq!(escaped(b"a b\tc"), "a b\tc");
    }

    #[test]
    fn escape_valid_utf8_is_verbatim() {
        // `café` and `日本語` are valid UTF-8 and must render unchanged, not escaped.
        assert_eq!(escaped("café".as_bytes()), "café");
        assert_eq!(escaped("日本語".as_bytes()), "日本語");
    }

    #[test]
    fn escape_invalid_bytes() {
        // 0xC9 is an incomplete 2-byte lead, 0xFF is never valid UTF-8.
        assert_eq!(escaped(b"Caf\xC9\xE9\xFF"), "Caf\\xC9\\xE9\\xFF");
        assert_eq!(escaped(b"\xFF\xFE"), "\\xFF\\xFE");
        // Invalid byte between valid runs.
        assert_eq!(escaped(b"a\xFFb"), "a\\xFFb");
        // Valid multibyte char followed by an invalid byte.
        assert_eq!(escaped(b"\xC3\xA9\xFF"), "é\\xFF");
        // Trailing incomplete sequence (error_len() == None path).
        assert_eq!(escaped(b"ab\xC9"), "ab\\xC9");
    }

    #[test]
    fn trim_start_byte_basic() {
        assert_eq!(trim_start_byte(b"", b'x'), b"");
        assert_eq!(trim_start_byte(b"xxx", b'x'), b"");
        assert_eq!(trim_start_byte(b"xxxa", b'x'), b"a");
        assert_eq!(trim_start_byte(b"axxx", b'x'), b"axxx");
        assert_eq!(trim_start_byte(b"abc", b'x'), b"abc");
    }

    #[test]
    fn trim_start_byte_long() {
        let s: Vec<u8> = b"xxxxxxxxxxxxxxxx".iter().chain(b"abc".iter()).copied().collect();
        assert_eq!(trim_start_byte(&s, b'x'), b"abc");
        let s: Vec<u8> = b"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx".iter().chain(b"yz".iter()).copied().collect();
        assert_eq!(trim_start_byte(&s, b'x'), b"yz");
        let s: Vec<u8> = b"xxxxxxxxxxxxxxxxx".iter().chain(b"q".iter()).copied().collect();
        assert_eq!(trim_start_byte(&s, b'x'), b"q");
        let s = vec![b'x'; 64];
        assert_eq!(trim_start_byte(&s, b'x'), b"");
        let mut s = vec![b'x'; 16];
        s[15] = b'q';
        assert_eq!(trim_start_byte(&s, b'x'), b"q");
    }

    #[test]
    fn trim_end_byte_basic() {
        assert_eq!(trim_end_byte(b"", b'x'), b"");
        assert_eq!(trim_end_byte(b"xxx", b'x'), b"");
        assert_eq!(trim_end_byte(b"axxx", b'x'), b"a");
        assert_eq!(trim_end_byte(b"xxxa", b'x'), b"xxxa");
        assert_eq!(trim_end_byte(b"abc", b'x'), b"abc");
    }

    #[test]
    fn trim_end_byte_long() {
        let s: Vec<u8> = b"abc".iter().chain(b"xxxxxxxxxxxxxxxx".iter()).copied().collect();
        assert_eq!(trim_end_byte(&s, b'x'), b"abc");
        let s: Vec<u8> = b"yz".iter().chain(b"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx".iter()).copied().collect();
        assert_eq!(trim_end_byte(&s, b'x'), b"yz");
        let s: Vec<u8> = b"q".iter().chain(b"xxxxxxxxxxxxxxxxx".iter()).copied().collect();
        assert_eq!(trim_end_byte(&s, b'x'), b"q");
        let s = vec![b'x'; 64];
        assert_eq!(trim_end_byte(&s, b'x'), b"");
        let mut s = vec![b'x'; 16];
        s[0] = b'q';
        assert_eq!(trim_end_byte(&s, b'x'), b"q");
    }

    #[test]
    fn trim_byte_both_sides() {
        assert_eq!(trim_byte(b"xxxabcxxx", b'x'), b"abc");
        assert_eq!(trim_byte(b"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxabcxxxxxxxxxxxxxxxx", b'x'), b"abc");
        assert_eq!(trim_byte(b"abc", b'x'), b"abc");
        assert_eq!(trim_byte(b"", b'x'), b"");
        assert_eq!(trim_byte(b"xxxx", b'x'), b"");
    }
}
