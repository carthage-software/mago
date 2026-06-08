use mago_allocator::Arena;
use mago_allocator::vec::Vec;

use mago_syntax::ast::LiteralStringKind;

use crate::internal::FormatterState;

pub(super) fn print_lowercase_keyword<'arena, A>(
    f: &FormatterState<'_, 'arena, A>,
    keyword: &'arena [u8],
) -> &'arena [u8]
where
    A: Arena,
{
    if keyword.iter().all(u8::is_ascii_lowercase) {
        return keyword;
    }

    let mut lowercase_bytes = Vec::with_capacity_in(keyword.len(), f.arena);
    for &byte in keyword {
        lowercase_bytes.push(byte.to_ascii_lowercase());
    }

    lowercase_bytes.leak()
}

pub(super) fn print_uppercase_keyword<'arena, A>(
    f: &FormatterState<'_, 'arena, A>,
    keyword: &'arena [u8],
) -> &'arena [u8]
where
    A: Arena,
{
    if keyword.iter().all(u8::is_ascii_uppercase) {
        return keyword;
    }

    let mut uppercase_bytes = Vec::with_capacity_in(keyword.len(), f.arena);
    for &byte in keyword {
        uppercase_bytes.push(byte.to_ascii_uppercase());
    }

    uppercase_bytes.leak()
}

pub(super) fn print_string<'arena, A>(
    f: &FormatterState<'_, 'arena, A>,
    kind: LiteralStringKind,
    text: &'arena [u8],
) -> &'arena [u8]
where
    A: Arena,
{
    // Strip binary string prefix (b/B) if present
    let (prefix, text_without_prefix): (&[u8], &[u8]) =
        if text.starts_with(b"b") || text.starts_with(b"B") { (&text[..1], &text[1..]) } else { (&[], text) };

    // SAFETY: callers always pass a non-empty string-literal token starting with a quote,
    // so the slice has at least one byte.
    let quote = unsafe { *text_without_prefix.first().unwrap_unchecked() };
    let raw_text = &text_without_prefix[1..text_without_prefix.len() - 1];
    let enclosing_quote = get_preferred_quote(raw_text, quote, f.settings.single_quote);

    match kind {
        LiteralStringKind::SingleQuoted if enclosing_quote == b'\'' => text,
        LiteralStringKind::DoubleQuoted if enclosing_quote == b'"' => text,
        _ if prefix.is_empty() => make_string_in(f.arena, raw_text, enclosing_quote),
        _ => {
            let inner = make_string_in(f.arena, raw_text, enclosing_quote);
            let mut result = Vec::with_capacity_in(inner.len() + 1, f.arena);
            result.extend_from_slice(prefix);
            result.extend_from_slice(inner);
            result.leak()
        }
    }
}

fn get_preferred_quote(raw: &[u8], enclosing_quote: u8, prefer_single_quote: bool) -> u8 {
    let (preferred_quote_char, alternate_quote_char) = if prefer_single_quote { (b'\'', b'"') } else { (b'"', b'\'') };

    let mut preferred_quote_count = 0;
    let mut alternate_quote_count = 0;

    let mut i = 0;
    while i < raw.len() {
        let byte = raw[i];
        if byte == preferred_quote_char {
            preferred_quote_count += 1;
        } else if byte == alternate_quote_char {
            alternate_quote_count += 1;
        } else if byte == b'\\'
            && let Some(&next_byte) = raw.get(i + 1)
        {
            if next_byte != enclosing_quote {
                return enclosing_quote;
            }
            i += 1;
        }
        i += 1;
    }

    if preferred_quote_count > alternate_quote_count { alternate_quote_char } else { preferred_quote_char }
}

/// Escapes a raw byte slice and encloses it in quotes, allocating the result in an arena.
///
/// # Arguments
///
/// * `arena`: The arena to allocate the new bytes in.
/// * `raw_text`: The raw byte content to process.
/// * `enclosing_quote`: The quote byte (`b'\''` or `b'"'`) to use for the output.
pub fn make_string_in<'arena, A>(arena: &'arena A, raw_text: &'arena [u8], enclosing_quote: u8) -> &'arena [u8]
where
    A: Arena,
{
    let mut result = Vec::with_capacity_in(raw_text.len() + 2, arena);
    result.push(enclosing_quote);

    let other_quote = if enclosing_quote == b'"' { b'\'' } else { b'"' };
    let mut i = 0;
    while i < raw_text.len() {
        let byte = raw_text[i];
        match byte {
            b'\\' => {
                if let Some(&next_byte) = raw_text.get(i + 1) {
                    if next_byte != other_quote {
                        result.push(b'\\');
                    }
                    result.push(next_byte);
                    i += 1;
                } else {
                    result.push(b'\\');
                }
            }
            _ if byte == enclosing_quote => {
                result.push(b'\\');
                result.push(byte);
            }
            _ => {
                result.push(byte);
            }
        }
        i += 1;
    }

    result.push(enclosing_quote);

    result.leak()
}
