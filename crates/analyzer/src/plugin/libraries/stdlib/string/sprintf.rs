//! `sprintf()` / format-string return type provider.
//!
//! When all arguments are known literals, resolves the exact result string.
//! When the format string is known but arguments are not all literals,
//! infers `non-empty-string` or `truthy-string` when the output is guaranteed
//! to be non-empty or longer than one character.
//!
//! The core logic is exposed via [`resolve_sprintf`] so it can be reused by
//! other providers (e.g. `Psl\Str\format`).

use std::fmt::Write;

use mago_atom::atom;
use mago_codex::ttype::get_literal_string;
use mago_codex::ttype::get_non_empty_string;
use mago_codex::ttype::get_truthy_string;
use mago_codex::ttype::union::TUnion;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::function::FunctionReturnTypeProvider;
use crate::plugin::provider::function::FunctionTarget;

static META: ProviderMeta =
    ProviderMeta::new("php::string::sprintf", "sprintf", "Resolves literal string for sprintf with literal args");

#[derive(Default)]
pub struct SprintfProvider;

impl Provider for SprintfProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl FunctionReturnTypeProvider for SprintfProvider {
    fn targets() -> FunctionTarget {
        FunctionTarget::Exact("sprintf")
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        resolve_sprintf(context, invocation)
    }
}

/// Resolve the return type of a sprintf-like call.
///
/// Expects the first argument to be the format string and subsequent arguments
/// to be the format values (standard `sprintf` / `Psl\Str\format` signature).
pub fn resolve_sprintf(
    context: &ProviderContext<'_, '_, '_>,
    invocation: &InvocationInfo<'_, '_, '_>,
) -> Option<TUnion> {
    let format_argument = invocation.get_argument(0, &["format"])?;
    let format_type = context.get_expression_type(format_argument)?;
    let format_str = format_type.get_single_literal_string_value()?;

    if let Some(result) = resolve_literal(format_str, context, invocation) {
        return Some(get_literal_string(atom(&result)));
    }

    let min_len = analyze_min_length(format_str);
    if min_len >= 2 {
        Some(get_truthy_string())
    } else if min_len >= 1 {
        Some(get_non_empty_string())
    } else {
        None
    }
}

/// Parse flags from a format specifier. Advances `i` past all flag characters.
/// Returns `None` if the format string is malformed (unexpected end).
fn parse_flags(bytes: &[u8], i: &mut usize) -> Option<(char, bool, bool)> {
    let len = bytes.len();
    let mut pad_char = ' ';
    let mut left_align = false;
    let mut show_sign = false;

    loop {
        if *i >= len {
            return None;
        }
        match bytes[*i] {
            b'-' => {
                left_align = true;
                *i += 1;
            }
            b'+' => {
                show_sign = true;
                *i += 1;
            }
            b' ' => *i += 1,
            b'0' => {
                pad_char = '0';
                *i += 1;
            }
            b'\'' => {
                *i += 1;
                if *i >= len {
                    return None;
                }

                pad_char = bytes[*i] as char;
                *i += 1;
            }
            _ => break,
        }
    }

    Some((pad_char, left_align, show_sign))
}

/// Parse a decimal number from `bytes` starting at `i`. Advances `i` past all digits.
fn parse_number(bytes: &[u8], i: &mut usize) -> usize {
    let len = bytes.len();
    let mut n: usize = 0;
    while *i < len && bytes[*i].is_ascii_digit() {
        n = n * 10 + (bytes[*i] - b'0') as usize;
        *i += 1;
    }

    n
}

/// Parse optional precision (`.N`). Advances `i` past the precision if present.
fn parse_precision(bytes: &[u8], i: &mut usize) -> Option<usize> {
    if *i < bytes.len() && bytes[*i] == b'.' {
        *i += 1;
        Some(parse_number(bytes, i))
    } else {
        None
    }
}

/// Try to fully resolve sprintf to a literal string when all arguments are known literals.
fn resolve_literal(
    format_str: &str,
    context: &ProviderContext<'_, '_, '_>,
    invocation: &InvocationInfo<'_, '_, '_>,
) -> Option<String> {
    let mut result = String::with_capacity(format_str.len());
    let mut buf = String::new();
    let bytes = format_str.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    let mut arg_index: usize = 1;

    while i < len {
        // Batch consecutive literal characters.
        if bytes[i] != b'%' {
            let start = i;
            i += 1;
            while i < len && bytes[i] != b'%' {
                i += 1;
            }

            result.push_str(&format_str[start..i]);
            continue;
        }

        i += 1;
        if i >= len {
            return None;
        }

        if bytes[i] == b'%' {
            result.push('%');
            i += 1;
            continue;
        }

        let (pad_char, left_align, show_sign) = parse_flags(bytes, &mut i)?;
        let width = parse_number(bytes, &mut i);
        let precision = parse_precision(bytes, &mut i);

        if i >= len {
            return None;
        }

        let specifier = bytes[i];
        let arg = invocation.get_argument(arg_index, &[])?;
        let arg_type = context.get_expression_type(arg)?;

        i += 1;
        arg_index += 1;

        let needs_buf = width > 0 || specifier == b'e' || specifier == b'E';
        let target = if needs_buf {
            buf.clear();
            &mut buf
        } else {
            &mut result
        };

        match specifier {
            b's' => {
                let value = arg_type.get_single_literal_string_value()?;
                if let Some(prec) = precision {
                    target.push_str(&value[..value.len().min(prec)]);
                } else {
                    target.push_str(value);
                }
            }
            b'd' => {
                let value = arg_type.get_single_literal_int_value()?;
                if show_sign && value >= 0 {
                    target.push('+');
                }

                let _ = write!(target, "{value}");
            }
            b'u' => {
                let value = arg_type.get_single_literal_int_value()?;
                let _ = write!(target, "{}", value as u64);
            }
            b'f' | b'F' => {
                let value = get_float_value(arg_type)?;
                let prec = precision.unwrap_or(6);
                if show_sign && value >= 0.0 {
                    target.push('+');
                }

                let _ = write!(target, "{value:.prec$}");
            }
            b'e' | b'E' => {
                let value = get_float_value(arg_type)?;
                let prec = precision.unwrap_or(6);
                if show_sign && value >= 0.0 {
                    target.push('+');
                }

                let mark = target.len();
                if specifier == b'e' {
                    let _ = write!(target, "{value:.prec$e}");
                } else {
                    let _ = write!(target, "{value:.prec$E}");
                }

                // Rust writes e.g. `1e0`, PHP writes `1e+0`. Insert `+` if needed.
                normalize_scientific_in_place(target, mark);
            }
            b'x' => {
                let value = arg_type.get_single_literal_int_value()?;
                let _ = write!(target, "{:x}", value as u64);
            }
            b'X' => {
                let value = arg_type.get_single_literal_int_value()?;
                let _ = write!(target, "{:X}", value as u64);
            }
            b'o' => {
                let value = arg_type.get_single_literal_int_value()?;
                let _ = write!(target, "{:o}", value as u64);
            }
            b'b' => {
                let value = arg_type.get_single_literal_int_value()?;
                let _ = write!(target, "{:b}", value as u64);
            }
            b'c' => {
                let value = arg_type.get_single_literal_int_value()?;
                target.push(char::from_u32(value as u32)?);
            }
            _ => return None,
        }

        if needs_buf {
            if width > 0 && buf.len() < width {
                let padding = width - buf.len();
                if left_align {
                    result.push_str(&buf);
                    for _ in 0..padding {
                        result.push(' ');
                    }
                } else {
                    for _ in 0..padding {
                        result.push(pad_char);
                    }
                    result.push_str(&buf);
                }
            } else {
                result.push_str(&buf);
            }
        }
    }

    Some(result)
}

/// Extract a float value from a type union, accepting either a literal float or literal int.
fn get_float_value(t: &TUnion) -> Option<f64> {
    if let Some(v) = t.get_single_literal_float_value() {
        Some(v)
    } else {
        t.get_single_literal_int_value().map(|v| v as f64)
    }
}

/// Insert a `+` sign after `e`/`E` in scientific notation if Rust omitted it.
/// Only scans bytes from `start` onward.
fn normalize_scientific_in_place(s: &mut String, start: usize) {
    let bytes = s.as_bytes();
    for j in start..bytes.len() {
        if bytes[j] == b'e' || bytes[j] == b'E' {
            if j + 1 < bytes.len() && bytes[j + 1] != b'+' && bytes[j + 1] != b'-' {
                s.insert(j + 1, '+');
            }
            return;
        }
    }
}

/// Analyze a format string to determine the minimum number of characters the output
/// will always contain, regardless of argument values.
fn analyze_min_length(format_str: &str) -> usize {
    let bytes = format_str.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    let mut min_len: usize = 0;

    while i < len {
        if bytes[i] != b'%' {
            let start = i;
            i += 1;
            while i < len && bytes[i] != b'%' {
                i += 1;
            }

            min_len += i - start;
            continue;
        }

        i += 1;
        if i >= len {
            return min_len;
        }

        if bytes[i] == b'%' {
            min_len += 1;
            i += 1;
            continue;
        }

        // Skip flags.
        loop {
            if i >= len {
                return min_len;
            }

            match bytes[i] {
                b'-' | b'+' | b' ' | b'0' => i += 1,
                b'\'' => {
                    i += 2;
                    if i > len {
                        return min_len;
                    }
                }
                _ => break,
            }
        }

        let width = parse_number(bytes, &mut i);

        // Skip precision.
        if i < len && bytes[i] == b'.' {
            i += 1;
            parse_number(bytes, &mut i);
        }

        if i >= len {
            return min_len;
        }

        let specifier = bytes[i];
        i += 1;

        let specifier_min = match specifier {
            b's' => 0,
            b'd' | b'u' | b'f' | b'F' | b'e' | b'E' | b'x' | b'X' | b'o' | b'b' | b'c' => 1,
            _ => 0,
        };

        min_len += specifier_min.max(width);
    }

    min_len
}
