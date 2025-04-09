use crate::input::Input;
use crate::number_separator;

#[inline]
pub fn parse_literal_float(value: &str) -> Option<f64> {
    let source = value.replace("_", "");

    source.parse::<f64>().ok()
}

#[inline]
pub fn parse_literal_integer(value: &str) -> Option<u64> {
    let source = value.replace("_", "");

    let value = match source.as_bytes() {
        [b'0', b'x' | b'X', ..] => u128::from_str_radix(&source.as_str()[2..], 16).ok(),
        [b'0', b'o' | b'O', ..] => u128::from_str_radix(&source.as_str()[2..], 8).ok(),
        [b'0', b'b' | b'B', ..] => u128::from_str_radix(&source.as_str()[2..], 2).ok(),
        _ => source.parse::<u128>().ok(),
    };

    value.map(|value| if value > u64::MAX as u128 { u64::MAX } else { value as u64 })
}

#[inline]
pub fn is_start_of_identifier(byte: &u8) -> bool {
    byte.is_ascii_lowercase() || byte.is_ascii_uppercase() || (*byte == b'_')
}

#[inline]
pub fn is_part_of_identifier(byte: &u8) -> bool {
    byte.is_ascii_digit()
        || byte.is_ascii_lowercase()
        || byte.is_ascii_uppercase()
        || (*byte == b'_')
        || (*byte >= 0x80)
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
