//! Helpers for inspecting comment trivia.

use crate::cst::Trivia;

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

    let body = trivia.value;
    let mut lines = Vec::new();
    let mut offset: u32 = 0;
    for line in body.split_inclusive(|&b| b == b'\n') {
        let trimmed = line.strip_suffix(b"\n").unwrap_or(line);
        lines.push((offset, trimmed));
        offset += line.len() as u32;
    }

    lines
}
