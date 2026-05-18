use crate::ast::Trivia;
use crate::ast::TriviaKind;

pub mod docblock;

/// Splits a byte slice into lines on `\n`, returning each line without its trailing
/// `\n` (and trailing `\r` if present). Empty trailing line after a final newline is
/// dropped to mirror `str::lines` behaviour.
fn byte_lines(input: &[u8]) -> impl Iterator<Item = &[u8]> + '_ {
    let mut rest = input;
    std::iter::from_fn(move || {
        if rest.is_empty() {
            return None;
        }
        match memchr::memchr(b'\n', rest) {
            Some(pos) => {
                let mut line = &rest[..pos];
                if line.last() == Some(&b'\r') {
                    line = &line[..line.len() - 1];
                }

                rest = &rest[pos + 1..];
                Some(line)
            }
            None => {
                let line = rest;
                rest = &[];
                Some(line)
            }
        }
    })
}

fn trim_ascii_whitespace_start(input: &[u8]) -> &[u8] {
    let start = input.iter().position(|b| !b.is_ascii_whitespace()).unwrap_or(input.len());
    &input[start..]
}

/// Splits a comment into lines, preserving the offset of each line from the start of the trivia.
///
/// This is crucial for calculating the precise `Span` of pragmas within a comment.
///
/// # Returns
///
/// A `Vec` of `(u32, &[u8])` tuples, where the `u32` is the byte offset of the
/// line from the start of the entire trivia text (including `/**`, `//`, etc.),
/// and the `&[u8]` is the cleaned line content.
#[inline]
#[must_use]
pub fn comment_lines<'arena>(trivia: &Trivia<'arena>) -> Vec<(u32, &'arena [u8])> {
    let full_text = trivia.value;
    let (content_start_offset, content_end_offset) = match trivia.kind {
        TriviaKind::MultiLineComment => (2u32, full_text.len() as u32 - 2),
        TriviaKind::DocBlockComment => (3u32, full_text.len() as u32 - 2),
        TriviaKind::SingleLineComment => (2u32, full_text.len() as u32),
        TriviaKind::HashComment => (1u32, full_text.len() as u32),
        TriviaKind::WhiteSpace => return vec![],
    };

    // Handle empty comments like `/**/` to prevent slicing panics.
    if content_start_offset >= content_end_offset {
        return vec![];
    }

    let content = &full_text[content_start_offset as usize..content_end_offset as usize];

    let mut lines = Vec::new();

    for line in byte_lines(content) {
        // Calculate the offset of the line relative to the start of the `content` slice.
        let relative_line_offset = (line.as_ptr() as u32) - (content.as_ptr() as u32);
        // Add the initial offset to get the position from the start of the entire trivia string.
        let offset_in_trivia = content_start_offset + relative_line_offset;

        let cleaned_line = if trivia.kind.is_block_comment() {
            let trimmed = trim_ascii_whitespace_start(line);
            if let Some(stripped) = trimmed.strip_prefix(b"*") { trim_ascii_whitespace_start(stripped) } else { line }
        } else {
            line
        };

        // Calculate how many bytes were trimmed from the start of the original line slice.
        let trimmed_bytes = (cleaned_line.as_ptr() as u32) - (line.as_ptr() as u32);
        let final_offset = offset_in_trivia + trimmed_bytes;

        lines.push((final_offset, cleaned_line));
    }

    lines
}
