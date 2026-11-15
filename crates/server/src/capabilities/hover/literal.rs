//! Primitive values at exact expression spans, without another analysis or parse.

use mago_bytes::BytesDisplay;
use mago_database::file::File;
use mago_syntax::token::TokenKind;
use mago_syntax_core::utils::parse_literal_integer;

use crate::{FileAnalysis, HoverEntry, HoverInfo, Range, Server, SourceDocumentation};

const VALUE_BYTES: usize = 512;
const VALUE_LINES: usize = 8;

pub(super) fn at(server: &Server, file: &File, analysis: &FileAnalysis, offset: u32) -> Option<HoverInfo> {
    let token = server.tokens.at_offset(file, offset)?;
    let ty = match token.kind {
        TokenKind::LiteralString => "string",
        TokenKind::LiteralInteger => match parse_literal_integer(token.value) {
            Some(value) if i64::try_from(value).is_ok() => "int",
            Some(_) => "float",
            None => "int|float",
        },
        TokenKind::LiteralFloat => "float",
        TokenKind::True | TokenKind::False => "bool",
        TokenKind::Null => "null",
        _ => return None,
    };
    let span = (token.start.offset, token.start.offset + token.value.len() as u32);
    // Literal keywords also occur in type hints and member names. Only expressions have values.
    analysis.expression_spans.binary_search(&span).ok()?;
    let preview = value_prefix(token.value);
    let value = BytesDisplay(preview);
    let declaration = if token.kind == TokenKind::Null { value.to_string() } else { format!("{ty}\n{value}") };
    Some(HoverInfo {
        entries: vec![HoverEntry {
            scope: None,
            declaration,
            documentation: (preview.len() < token.value.len()).then(|| SourceDocumentation {
                text: "The value exceeds the preview limit.".to_owned(),
                tags: Vec::new(),
            }),
        }],
        range: Range::new(span.0, span.1),
    })
}

fn value_prefix(value: &[u8]) -> &[u8] {
    let mut end = value.len().min(VALUE_BYTES);
    if let Some((line_end, _)) =
        value[..end].iter().enumerate().filter(|(_, byte)| matches!(byte, b'\n' | b'\r')).nth(VALUE_LINES - 1)
    {
        end = line_end;
    }
    if let Some(chunk) = value[..end].utf8_chunks().last()
        && let Err(error) = std::str::from_utf8(chunk.invalid())
        && error.error_len().is_none()
    {
        end -= chunk.invalid().len();
    }
    &value[..end]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_keeps_whole_utf8_characters_after_invalid_php_bytes() {
        let mut value = b"'\xFF".to_vec();
        value.extend(std::iter::repeat_n(b'a', VALUE_BYTES - 3));
        value.extend_from_slice("😀'".as_bytes());
        let preview = value_prefix(&value);
        assert_eq!(preview, &value[..VALUE_BYTES - 1]);
        assert!(BytesDisplay(preview).to_string().starts_with("'\\xFF"));
    }
}
