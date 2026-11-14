//! Helpers for inspecting comment trivia.

use crate::cst::Trivia;
use crate::cst::TriviaKind;

pub use crate::comments::scope::CommentScopes;

mod scope;
mod tags;

/// Split a comment's body into newline-delimited lines, retaining the
/// byte-offset of each line relative to the start of the trivia.
///
/// The offsets are useful when a consumer wants to recover precise spans
/// for pragmas or annotations embedded inside a comment body.
#[must_use]
pub fn comment_lines<'arena>(trivia: &Trivia<'arena>) -> Vec<(u32, &'arena [u8])> {
    if !trivia.kind.is_comment() {
        return Vec::new();
    }

    let full = trivia.value;
    let (start, end) = match trivia.kind {
        TriviaKind::Comment => {
            let start = 2 + usize::from(full.get(2).is_some_and(|byte| matches!(byte, b'-' | b'~')));
            let end = full.len().saturating_sub(2);
            let end = end - usize::from(full.get(end.wrapping_sub(1)).is_some_and(|byte| matches!(byte, b'-' | b'~')));
            (start, end)
        }
        TriviaKind::InlineComment => (1, full.len()),
        TriviaKind::Whitespace => return Vec::new(),
    };

    let Some(body) = full.get(start..end) else {
        return Vec::new();
    };

    let mut lines = Vec::new();
    let mut cursor = 0;
    while cursor < body.len() {
        let length =
            body[cursor..].iter().position(|byte| matches!(byte, b'\r' | b'\n')).unwrap_or(body.len() - cursor);
        lines.push(((start + cursor) as u32, &body[cursor..cursor + length]));
        cursor += length;
        if body.get(cursor..cursor + 2) == Some(b"\r\n") {
            cursor += 2;
        } else {
            cursor += 1;
        }
    }

    lines
}

#[cfg(test)]
mod tests {
    use mago_database::file::FileId;
    use mago_span::Span;

    use super::*;

    #[test]
    fn keeps_offsets_without_delimiters_or_trim_markers() {
        for source in [b"{#- first\r\nsecond\rthird\nfourth -#}".as_slice(), b"{#~ first\r\nsecond\rthird\nfourth ~#}"]
        {
            let trivia = Trivia {
                kind: TriviaKind::Comment,
                span: Span::new(FileId::zero(), 0.into(), (source.len() as u32).into()),
                value: source,
            };

            let lines = comment_lines(&trivia);
            assert_eq!(
                lines.iter().map(|(_, line)| *line).collect::<Vec<_>>(),
                [b" first".as_slice(), b"second", b"third", b"fourth "]
            );
            for (offset, line) in lines {
                assert_eq!(&source[offset as usize..offset as usize + line.len()], line);
            }
        }
    }
}
