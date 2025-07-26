use mago_interner::StringIdentifier;
use mago_syntax::ast::LiteralStringKind;

use crate::internal::FormatterState;

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

fn make_string(raw_text: &str, enclosing_quote: char) -> String {
    let other_quote = if enclosing_quote == '"' { '\'' } else { '"' };
    let mut result = String::new();
    result.push(enclosing_quote);

    let mut chars = raw_text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                if let Some(&next_char) = chars.peek() {
                    if next_char != other_quote {
                        result.push('\\');
                    }
                    result.push(next_char);
                    chars.next();
                } else {
                    result.push('\\');
                }
            }
            _ if c == enclosing_quote => {
                result.push('\\');
                result.push(c);
            }
            _ => result.push(c),
        }
    }

    result.push(enclosing_quote);
    result
}

pub(super) fn print_string<'a>(
    f: &FormatterState<'a>,
    kind: Option<LiteralStringKind>,
    value: &StringIdentifier,
) -> &'a str {
    let text = f.lookup(value);

    let quote = unsafe { text.chars().next().unwrap_unchecked() };
    let raw_text = &text[1..text.len() - 1];
    let enclosing_quote = get_preferred_quote(raw_text, quote, f.settings.single_quote);

    match kind {
        None => text,
        Some(LiteralStringKind::SingleQuoted) if enclosing_quote == '\'' => text,
        Some(LiteralStringKind::DoubleQuoted) if enclosing_quote == '"' => text,
        _ => f.as_str(make_string(raw_text, enclosing_quote)),
    }
}
