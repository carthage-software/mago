//! Helpers for inspecting comment trivia.

use crate::ast::Trivia;

/// Split a comment's body into newline-delimited lines, retaining the
/// byte-offset of each line relative to the start of the trivia.
///
/// The offsets are useful when a consumer wants to recover precise spans
/// for pragmas or annotations embedded inside a comment body.
#[must_use]
pub fn comment_lines<'arena>(trivia: &Trivia<'arena>) -> Vec<(u32, &'arena str)> {
    if !trivia.kind.is_comment() {
        return Vec::new();
    }

    let body = trivia.value;
    let mut lines = Vec::new();
    let mut offset: u32 = 0;
    for line in body.split_inclusive('\n') {
        let trimmed = line.strip_suffix('\n').unwrap_or(line);
        lines.push((offset, trimmed));
        offset += line.len() as u32;
    }

    lines
}
