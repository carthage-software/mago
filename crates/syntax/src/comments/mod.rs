use crate::ast::*;

pub mod docblock;

/// Splits a comment into lines, preserving the offset of each line from the start of the trivia.
///
/// This is crucial for calculating the precise `Span` of pragmas within a comment.
///
/// # Returns
///
/// A `Vec` of `(usize, &str)` tuples, where the `usize` is the byte offset of the
/// line from the start of the entire trivia text (including `/**`, `//`, etc.),
/// and the `&str` is the cleaned line content.
#[inline]
pub fn comment_lines<'arena>(trivia: &Trivia<'arena>) -> Vec<(u32, &'arena str)> {
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

    for line in content.lines() {
        // Calculate the offset of the line relative to the start of the `content` slice.
        let relative_line_offset = (line.as_ptr() as u32) - (content.as_ptr() as u32);
        // Add the initial offset to get the position from the start of the entire trivia string.
        let offset_in_trivia = content_start_offset + relative_line_offset;

        let cleaned_line = if trivia.kind.is_block_comment() {
            if let Some(stripped) = line.trim_start().strip_prefix('*') { stripped.trim_start() } else { line }
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
