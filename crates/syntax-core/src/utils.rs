use mago_allocator::prelude::*;

use crate::input::Input;
use crate::number_separator;

/// Parses a PHP literal string, handling all escape sequences, and allocates the result in an arena.
///
/// # Returns
///
/// An `Option` containing the parsed `&'arena str` or `None` if the input is invalid.
///
/// # Panics
///
/// Panics if internal assumptions about character parsing are violated (e.g., invalid hex or octal digits
/// after validation). This should not occur with valid PHP strings.
pub fn parse_literal_string_in<'arena, A>(
    arena: &'arena A,
    s: &'arena [u8],
    quote_char: Option<u8>,
    has_quote: bool,
) -> Option<&'arena [u8]>
where
    A: Arena,
{
    if s.is_empty() {
        return Some(b"");
    }

    let s = if has_quote
        && (s.starts_with(b"b\"") || s.starts_with(b"b'") || s.starts_with(b"B\"") || s.starts_with(b"B'"))
    {
        &s[1..]
    } else {
        s
    };

    let (quote_char, content) = if let Some(quote_char) = quote_char {
        (Some(quote_char), s)
    } else if !has_quote {
        (None, s)
    } else if s.starts_with(b"\"") && s.ends_with(b"\"") && s.len() >= 2 {
        (Some(b'"'), &s[1..s.len() - 1])
    } else if s.starts_with(b"'") && s.ends_with(b"'") && s.len() >= 2 {
        (Some(b'\''), &s[1..s.len() - 1])
    } else {
        return None;
    };

    let needs_processing = content.contains(&b'\\') || quote_char.is_some_and(|q| content.contains(&q));
    if !needs_processing {
        return Some(content);
    }

    let mut result = Vec::with_capacity_in(content.len(), arena);
    let mut i = 0;

    while i < content.len() {
        let b = content[i];
        if b != b'\\' {
            result.push(b);
            i += 1;
            continue;
        }

        let next_index = i + 1;
        let Some(&next) = content.get(next_index) else {
            result.push(b'\\');
            i += 1;
            continue;
        };

        // Most escapes consume two bytes (`\` + the next byte). The hex and octal
        // forms scan additional digit bytes and update `i` themselves.
        let mut consumed = 2;

        match next {
            b'\\' => result.push(b'\\'),
            b'\'' if quote_char == Some(b'\'') => result.push(b'\''),
            b'"' if quote_char == Some(b'"') => result.push(b'"'),
            b'$' if quote_char == Some(b'"') => result.push(b'$'),
            b'n' if quote_char == Some(b'"') => result.push(b'\n'),
            b't' if quote_char == Some(b'"') => result.push(b'\t'),
            b'r' if quote_char == Some(b'"') => result.push(b'\r'),
            b'v' if quote_char == Some(b'"') => result.push(0x0B),
            b'e' if quote_char == Some(b'"') => result.push(0x1B),
            b'f' if quote_char == Some(b'"') => result.push(0x0C),
            b'x' if quote_char == Some(b'"') => {
                let mut hex_val = 0u8;
                let mut hex_len = 0;
                let mut j = i + 2;
                while hex_len < 2 && j < content.len() {
                    let c = content[j];
                    let digit = if c.is_ascii_digit() {
                        c - b'0'
                    } else if (b'a'..=b'f').contains(&c) {
                        c - b'a' + 10
                    } else if (b'A'..=b'F').contains(&c) {
                        c - b'A' + 10
                    } else {
                        break;
                    };
                    hex_val = hex_val * 16 + digit;
                    hex_len += 1;
                    j += 1;
                }
                if hex_len > 0 {
                    result.push(hex_val);
                    consumed = 2 + hex_len;
                } else {
                    // Invalid `\x` sequence, treat as literal `\x`
                    result.push(b'\\');
                    result.push(b'x');
                }
            }
            c if quote_char == Some(b'"') && c.is_ascii_digit() => {
                let mut octal_val = 0u16;
                let mut octal_len = 0;
                let mut j = i + 1;
                while octal_len < 3 && j < content.len() {
                    let d = content[j];
                    if d.is_ascii_digit() && d <= b'7' {
                        octal_val = octal_val * 8 + u16::from(d - b'0');
                        octal_len += 1;
                        j += 1;
                    } else {
                        break;
                    }
                }
                if octal_len > 0 {
                    // Truncate to u8 (matches PHP behavior for octal sequences > 255)
                    result.push(octal_val as u8);
                    consumed = 1 + octal_len;
                } else {
                    result.push(b'\\');
                    result.push(next);
                }
            }
            _ => {
                // Unrecognized escape sequence
                result.push(b'\\');
                result.push(next);
            }
        }

        i += consumed;
    }

    Some(result.leak())
}

/// Parses a PHP literal float, handling underscore separators.
#[inline]
#[must_use]
pub fn parse_literal_float(value: &[u8]) -> Option<f64> {
    if memchr::memchr(b'_', value).is_none() {
        return std::str::from_utf8(value).ok()?.parse::<f64>().ok();
    }

    let mut buf = [0u8; 64];
    let mut len = 0;

    for &b in value {
        if b != b'_' {
            if len < 64 {
                buf[len] = b;
                len += 1;
            } else {
                let source: std::vec::Vec<u8> = value.iter().copied().filter(|&b| b != b'_').collect();
                return std::str::from_utf8(&source).ok()?.parse::<f64>().ok();
            }
        }
    }

    std::str::from_utf8(&buf[..len]).ok()?.parse::<f64>().ok()
}

/// Parses a PHP literal integer with support for binary, octal, decimal, and hex.
///
/// Optimized to use byte-level iteration instead of Unicode chars.
#[inline]
#[must_use]
pub fn parse_literal_integer(bytes: &[u8]) -> Option<u64> {
    if bytes.is_empty() {
        return None;
    }

    let (radix, start) = match bytes {
        [b'0', b'x' | b'X', ..] => (16u128, 2),
        [b'0', b'o' | b'O', ..] => (8u128, 2),
        [b'0', b'b' | b'B', ..] => (2u128, 2),
        [b'0', _, ..] if bytes[1..].iter().all(|&b| b == b'_' || (b'0'..=b'7').contains(&b)) => (8u128, 1), // Legacy octal
        [b'0', _, ..] => (10u128, 0), // Invalid octal (contains 8/9), treat as decimal
        _ => (10u128, 0),
    };

    let mut result: u128 = 0;
    let mut has_digits = false;

    for &b in &bytes[start..] {
        if b == b'_' {
            continue;
        }

        let digit = if b.is_ascii_digit() {
            (b - b'0') as u128
        } else if (b'a'..=b'f').contains(&b) {
            (b - b'a' + 10) as u128
        } else if (b'A'..=b'F').contains(&b) {
            (b - b'A' + 10) as u128
        } else {
            return None;
        };

        if digit >= radix {
            return None;
        }

        has_digits = true;

        result = match result.checked_mul(radix) {
            Some(r) => r,
            None => return Some(u64::MAX),
        };

        result = match result.checked_add(digit) {
            Some(r) => r,
            None => return Some(u64::MAX),
        };
    }

    if !has_digits {
        return None;
    }

    Some(result.min(u64::MAX as u128) as u64)
}

/// Lookup table for identifier start characters (a-z, A-Z, _)
/// Index by byte value, true if valid start of identifier
static IS_IDENT_START: [bool; 256] = {
    let mut table = [false; 256];
    let mut i = 0u8;
    loop {
        table[i as usize] = matches!(i, b'a'..=b'z' | b'A'..=b'Z' | b'_');
        if i == 255 {
            break;
        }
        i += 1;
    }

    table
};

/// Lookup table for identifier continuation characters (a-z, A-Z, 0-9, _, or >= 0x80)
/// Index by byte value, true if valid part of identifier
static IS_IDENT_PART: [bool; 256] = {
    let mut table = [false; 256];
    let mut i = 0u8;
    loop {
        table[i as usize] = matches!(i, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | 0x80..=0xFF);
        if i == 255 {
            break;
        }
        i += 1;
    }
    table
};

/// Check if a byte can start an identifier (a-z, A-Z, _)
#[inline(always)]
#[must_use]
pub const fn is_start_of_identifier(byte: &u8) -> bool {
    IS_IDENT_START[*byte as usize]
}

/// Check if a byte can be part of an identifier (a-z, A-Z, 0-9, _, or >= 0x80)
#[inline(always)]
#[must_use]
pub const fn is_part_of_identifier(byte: &u8) -> bool {
    IS_IDENT_PART[*byte as usize]
}

/// Scans an identifier starting at `offset` in the byte slice and returns the length.
///
/// Assumes the first byte is already validated as a start of identifier.
/// Returns the total length of the identifier (including the first byte).
/// Stops at the first byte that is not a valid identifier character.
#[inline(always)]
#[must_use]
pub fn scan_identifier_length(bytes: &[u8], offset: usize) -> usize {
    let mut len = 1;
    let remaining = &bytes[offset + 1..];

    for &b in remaining {
        if IS_IDENT_PART[b as usize] {
            len += 1;
        } else {
            break;
        }
    }

    len
}

/// Reads a sequence of bytes representing digits in a specific numerical base.
///
/// This utility function iterates through the input byte slice, consuming bytes
/// as long as they represent valid digits for the given `base`. It handles
/// decimal digits ('0'-'9') and hexadecimal digits ('a'-'f', 'A'-'F').
///
/// It stops consuming at the first byte that is not a valid digit character,
/// or is a digit character whose value is greater than or equal to the specified `base`
/// (e.g., '8' in base 8, or 'A' in base 10).
///
/// This function is primarily intended as a helper for lexer implementations
/// when tokenizing the digit part of number literals (binary, octal, decimal, hexadecimal).
///
/// # Arguments
///
/// * `input` - A byte slice starting at the potential first digit of the number.
/// * `base` - The numerical base (e.g., 2, 8, 10, 16) to use for validating digits.
///   Must be between 2 and 36 (inclusive) for hex characters to be potentially valid.
///
/// # Returns
///
/// The number of bytes (`usize`) consumed from the beginning of the `input` slice
/// that constitute a valid sequence of digits for the specified `base`. Returns 0 if
/// the first byte is not a valid digit for the base.
#[inline]
pub fn read_digits_of_base(input: &Input, offset: usize, base: u8) -> usize {
    if base == 16 {
        read_digits_with(input, offset, u8::is_ascii_hexdigit)
    } else {
        let max = b'0' + base;

        read_digits_with(input, offset, |b| b >= &b'0' && b < &max)
    }
}

#[inline]
fn read_digits_with<F>(input: &Input, offset: usize, is_digit: F) -> usize
where
    F: Fn(&u8) -> bool,
{
    let bytes = input.bytes;
    let total = input.length;
    let start = input.offset;
    let mut pos = start + offset; // Compute the absolute position.

    while pos < total {
        let current = bytes[pos];
        if is_digit(&current) {
            pos += 1;
        } else if pos + 1 < total && bytes[pos] == number_separator!() && is_digit(&bytes[pos + 1]) {
            pos += 2; // Skip the separator and the digit.
        } else {
            break;
        }
    }

    // Return the relative length from the start of the current position.
    pos - start
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! parse_int {
        ($input:expr, $expected:expr) => {
            assert_eq!(parse_literal_integer($input), $expected);
        };
    }

    #[test]
    fn test_parse_literal_integer() {
        parse_int!(b"123", Some(123));
        parse_int!(b"0", Some(0));
        parse_int!(b"0b1010", Some(10));
        parse_int!(b"0o17", Some(15));
        parse_int!(b"0x1A3F", Some(6719));
        parse_int!(b"0XFF", Some(255));
        parse_int!(b"0_1_2_3", Some(83));
        parse_int!(b"0b1_0_1_0", Some(10));
        parse_int!(b"0o1_7", Some(15));
        parse_int!(b"0x1_A_3_F", Some(6719));
        parse_int!(b"", None);
        parse_int!(b"0xGHI", None);
        parse_int!(b"0b102", None);
        parse_int!(b"0o89", None);
    }
}
