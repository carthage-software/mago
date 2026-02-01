use bumpalo::Bump;
use bumpalo::collections::Vec;

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
pub fn parse_literal_string_in<'arena>(
    arena: &'arena Bump,
    s: &'arena str,
    quote_char: Option<char>,
    has_quote: bool,
) -> Option<&'arena str> {
    if s.is_empty() {
        return Some("");
    }

    let (quote_char, content) = if let Some(quote_char) = quote_char {
        (Some(quote_char), s)
    } else if !has_quote {
        (None, s)
    } else if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        (Some('"'), &s[1..s.len() - 1])
    } else if s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2 {
        (Some('\''), &s[1..s.len() - 1])
    } else {
        return None;
    };

    let needs_processing = content.contains('\\') || quote_char.is_some_and(|q| content.contains(q));
    if !needs_processing {
        return Some(content);
    }

    let mut result = Vec::with_capacity_in(content.len(), arena);
    let mut chars = content.chars().peekable();
    let mut buf = [0; 4];

    while let Some(c) = chars.next() {
        if c != '\\' {
            result.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
            continue;
        }

        let Some(&next_char) = chars.peek() else {
            result.push(b'\\');
            continue;
        };

        let mut consumed = true;

        match next_char {
            '\\' => result.push(b'\\'),
            '\'' if quote_char == Some('\'') => result.push(b'\''),
            '"' if quote_char == Some('"') => result.push(b'"'),
            '$' if quote_char == Some('"') => result.push(b'$'),
            'n' if quote_char == Some('"') => result.push(b'\n'),
            't' if quote_char == Some('"') => result.push(b'\t'),
            'r' if quote_char == Some('"') => result.push(b'\r'),
            'v' if quote_char == Some('"') => result.push(0x0B),
            'e' if quote_char == Some('"') => result.push(0x1B),
            'f' if quote_char == Some('"') => result.push(0x0C),
            '0' if quote_char == Some('"') => result.push(0x00),
            'x' if quote_char == Some('"') => {
                chars.next(); // Consume 'x'
                let mut hex_val = 0u8;
                let mut hex_len = 0;
                // Peek up to 2 hex digits
                while let Some(peeked) = chars.peek() {
                    if hex_len < 2 && peeked.is_ascii_hexdigit() {
                        hex_val = hex_val * 16 + peeked.to_digit(16).unwrap() as u8;
                        hex_len += 1;
                        chars.next(); // Consume the digit
                    } else {
                        break;
                    }
                }
                if hex_len > 0 {
                    result.push(hex_val);
                } else {
                    // Invalid `\x` sequence, treat as literal `\x`
                    result.push(b'\\');
                    result.push(b'x');
                }

                consumed = false;
            }
            c if quote_char == Some('"') && c.is_ascii_digit() => {
                let mut octal_val = 0u16;
                let mut octal_len = 0;

                while let Some(peeked) = chars.peek() {
                    if octal_len < 3 && peeked.is_ascii_digit() && *peeked <= '7' {
                        octal_val = octal_val * 8 + peeked.to_digit(8).unwrap() as u16;
                        octal_len += 1;
                        chars.next(); // Consume the digit
                    } else {
                        break;
                    }
                }
                if octal_len > 0 {
                    // Truncate to u8 (matches PHP behavior for octal sequences > 255)
                    result.push(octal_val as u8);
                } else {
                    result.push(b'\\');
                    result.push(b'0');
                }

                consumed = false;
            }
            _ => {
                // Unrecognized escape sequence
                if quote_char == Some('\'') {
                    // In single quotes, only \' and \\ are special.
                    result.push(b'\\');
                    result.extend_from_slice(next_char.encode_utf8(&mut buf).as_bytes());
                } else {
                    // In double quotes, an invalid escape is just the character.
                    result.extend_from_slice(next_char.encode_utf8(&mut buf).as_bytes());
                }
            }
        }

        if consumed {
            chars.next(); // Consume the character after the backslash
        }
    }

    std::str::from_utf8(result.into_bump_slice()).ok()
}

/// Parses a PHP literal string, handling all escape sequences, and returns the result as a `String`.
///
/// # Returns
///
/// An `Option<String>` containing the parsed string or `None` if the input is invalid.
///
/// # Notes
///
/// This function is similar to `parse_literal_string_in`, but it allocates the result on the heap instead of in an arena.
/// It is recommended to use `parse_literal_string_in` when possible for better performance in contexts where an arena is available.
///
/// # Panics
///
/// Panics if internal assumptions about character parsing are violated (e.g., invalid hex or octal digits
/// after validation). This should not occur with valid PHP strings.
#[inline]
#[must_use]
pub fn parse_literal_string(s: &str, quote_char: Option<char>, has_quote: bool) -> Option<String> {
    if s.is_empty() {
        return Some(String::new());
    }

    let (quote_char, content) = if let Some(quote_char) = quote_char {
        (Some(quote_char), s)
    } else if !has_quote {
        (None, s)
    } else if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        (Some('"'), &s[1..s.len() - 1])
    } else if s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2 {
        (Some('\''), &s[1..s.len() - 1])
    } else {
        return None;
    };

    let mut result = String::new();
    let mut chars = content.chars().peekable();

    while let Some(c) = chars.next() {
        if c != '\\' {
            result.push(c);

            continue;
        }

        let Some(&next_char) = chars.peek() else {
            result.push(c);

            continue;
        };

        match next_char {
            '\\' => {
                result.push('\\');
                chars.next();
            }
            '\'' if quote_char == Some('\'') => {
                result.push('\'');
                chars.next();
            }
            '"' if quote_char == Some('"') => {
                result.push('"');
                chars.next();
            }
            'n' if quote_char == Some('"') => {
                result.push('\n');
                chars.next();
            }
            't' if quote_char == Some('"') => {
                result.push('\t');
                chars.next();
            }
            'r' if quote_char == Some('"') => {
                result.push('\r');
                chars.next();
            }
            'v' if quote_char == Some('"') => {
                result.push('\x0B');
                chars.next();
            }
            'e' if quote_char == Some('"') => {
                result.push('\x1B');
                chars.next();
            }
            'f' if quote_char == Some('"') => {
                result.push('\x0C');
                chars.next();
            }
            '0' if quote_char == Some('"') => {
                result.push('\0');
                chars.next();
            }
            'x' if quote_char == Some('"') => {
                chars.next();

                let mut hex_chars = String::new();
                for _ in 0..2 {
                    if let Some(&next) = chars.peek() {
                        if next.is_ascii_hexdigit() {
                            hex_chars.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                }

                if hex_chars.is_empty() {
                    return None;
                }
                match u8::from_str_radix(&hex_chars, 16) {
                    Ok(byte_val) => result.push(byte_val as char),
                    Err(_) => {
                        return None;
                    }
                }
            }
            c if quote_char == Some('"') && c.is_ascii_digit() => {
                let mut octal = String::new();
                octal.push(chars.next().unwrap());

                for _ in 0..2 {
                    if let Some(&next) = chars.peek() {
                        if next.is_ascii_digit() && next <= '7' {
                            octal.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                }

                result.push(u8::from_str_radix(&octal, 8).ok()? as char);
            }
            '$' if quote_char == Some('"') => {
                result.push('$');
                chars.next();
            }
            _ => {
                if quote_char == Some('\'') {
                    result.push(c);
                    result.push(next_char);
                    chars.next();
                } else {
                    result.push(c);
                }
            }
        }
    }

    Some(result)
}

/// Parses a PHP literal float, handling underscore separators.
#[inline]
#[must_use]
pub fn parse_literal_float(value: &str) -> Option<f64> {
    if memchr::memchr(b'_', value.as_bytes()).is_none() {
        return value.parse::<f64>().ok();
    }

    let mut buf = [0u8; 64];
    let mut len = 0;

    for &b in value.as_bytes() {
        if b != b'_' {
            if len < 64 {
                buf[len] = b;
                len += 1;
            } else {
                let source = value.replace('_', "");
                return source.parse::<f64>().ok();
            }
        }
    }

    // SAFETY: We only copied ASCII bytes from a valid UTF-8 string
    let s = unsafe { std::str::from_utf8_unchecked(&buf[..len]) };
    s.parse::<f64>().ok()
}

/// Parses a PHP literal integer with support for binary, octal, decimal, and hex.
///
/// Optimized to use byte-level iteration instead of Unicode chars.
#[inline]
#[must_use]
pub fn parse_literal_integer(value: &str) -> Option<u64> {
    let bytes = value.as_bytes();
    if bytes.is_empty() {
        return None;
    }

    let (radix, start) = match bytes {
        [b'0', b'x' | b'X', ..] => (16u128, 2),
        [b'0', b'o' | b'O', ..] => (8u128, 2),
        [b'0', b'b' | b'B', ..] => (2u128, 2),
        [b'0', _, ..] => (8u128, 1), // Legacy octal
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
/// Assumes the first byte is already validated as a start of identifier.
/// Returns the total length of the identifier (including the first byte).
///
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
fn read_digits_with<F: Fn(&u8) -> bool>(input: &Input, offset: usize, is_digit: F) -> usize {
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
        parse_int!("123", Some(123));
        parse_int!("0", Some(0));
        parse_int!("0b1010", Some(10));
        parse_int!("0o17", Some(15));
        parse_int!("0x1A3F", Some(6719));
        parse_int!("0XFF", Some(255));
        parse_int!("0_1_2_3", Some(83));
        parse_int!("0b1_0_1_0", Some(10));
        parse_int!("0o1_7", Some(15));
        parse_int!("0x1_A_3_F", Some(6719));
        parse_int!("", None);
        parse_int!("0xGHI", None);
        parse_int!("0b102", None);
        parse_int!("0o89", None);
    }
}
