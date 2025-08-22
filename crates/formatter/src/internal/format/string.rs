use bumpalo::Bump;
use bumpalo::collections::Vec;

use mago_syntax::ast::LiteralStringKind;

use crate::internal::FormatterState;

pub(super) fn print_lowercase_keyword<'arena>(f: &FormatterState<'_, 'arena>, keyword: &'arena str) -> &'arena str {
    if keyword.chars().all(|c| c.is_ascii_lowercase()) {
        return keyword;
    }

    let mut lowercase_bytes = Vec::with_capacity_in(keyword.len(), f.arena);
    for c in keyword.chars() {
        for lower_c in c.to_lowercase() {
            let mut buf = [0; 4];
            lowercase_bytes.extend_from_slice(lower_c.encode_utf8(&mut buf).as_bytes());
        }
    }

    (unsafe { std::str::from_utf8_unchecked(lowercase_bytes.into_bump_slice()) }) as _
}

pub(super) fn print_string<'arena>(
    f: &FormatterState<'_, 'arena>,
    kind: Option<LiteralStringKind>,
    text: &'arena str,
) -> &'arena str {
    let quote = unsafe { text.chars().next().unwrap_unchecked() };
    let raw_text = &text[1..text.len() - 1];
    let enclosing_quote = get_preferred_quote(raw_text, quote, f.settings.single_quote);

    match kind {
        None => text,
        Some(LiteralStringKind::SingleQuoted) if enclosing_quote == '\'' => text,
        Some(LiteralStringKind::DoubleQuoted) if enclosing_quote == '"' => text,
        _ => make_string_in(f.arena, raw_text, enclosing_quote),
    }
}

fn get_preferred_quote(raw: &str, enclosing_quote: char, prefer_single_quote: bool) -> char {
    let (preferred_quote_char, alternate_quote_char) = if prefer_single_quote { ('\'', '"') } else { ('"', '\'') };

    let mut preferred_quote_count = 0;
    let mut alternate_quote_count = 0;

    for character in raw.chars() {
        if character == preferred_quote_char {
            preferred_quote_count += 1;
        } else if character == alternate_quote_char {
            alternate_quote_count += 1;
        } else if character == '\\' && !matches!(raw.chars().next(), Some(c) if c == enclosing_quote) {
            // If the string contains a backslash followed by the other quote character, we should
            // prefer the existing quote character.
            return enclosing_quote;
        }
    }

    if preferred_quote_count > alternate_quote_count { alternate_quote_char } else { preferred_quote_char }
}

/// Escapes a raw string and encloses it in quotes, allocating the result in an arena.
///
/// # Arguments
///
/// * `arena`: The `Bump` arena to allocate the new string in.
/// * `raw_text`: The raw string content to process.
/// * `enclosing_quote`: The quote character (' or ") to use for the output.
pub fn make_string_in<'arena>(arena: &'arena Bump, raw_text: &'arena str, enclosing_quote: char) -> &'arena str {
    // Pre-allocate with a reasonable guess to avoid reallocations within the arena.
    let mut result = Vec::with_capacity_in(raw_text.len() + 2, arena);
    result.push(enclosing_quote as u8);

    let other_quote = if enclosing_quote == '"' { '\'' } else { '"' };
    let mut chars = raw_text.chars().peekable();

    while let Some(c) = chars.next() {
        let mut buf = [0; 4];
        match c {
            '\\' => {
                if let Some(&next_char) = chars.peek() {
                    if next_char != other_quote {
                        result.push(b'\\');
                    }

                    chars.next();
                    result.extend_from_slice(next_char.encode_utf8(&mut buf).as_bytes());
                } else {
                    result.push(b'\\');
                }
            }
            _ if c == enclosing_quote => {
                result.push(b'\\');
                result.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
            }
            _ => {
                result.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
            }
        }
    }

    result.push(enclosing_quote as u8);

    // Convert the byte vec into a slice and then to a string slice.
    // SAFETY: The logic ensures only valid UTF-8 characters are pushed.
    unsafe { std::str::from_utf8_unchecked(result.into_bump_slice()) }
}
