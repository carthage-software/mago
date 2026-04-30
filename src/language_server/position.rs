//! LSP position ↔ mago byte offset conversion.
//!
//! LSP uses zero-based `(line, character)` coordinates with UTF-16 code units
//! by default. Mago tracks positions as zero-based byte offsets into the file
//! contents, with a precomputed line-start table on `mago_database::file::File`.
//!
//! For now we treat the `character` field as bytes; UTF-8 columns rather than
//! UTF-16. This is wrong for files containing characters above the BMP, but
//! correct for the overwhelming majority of PHP code. We'll revisit when we
//! ship the negotiated `general.positionEncodings` capability.

use mago_database::file::File as MagoFile;
use tower_lsp::lsp_types::Position;
use tower_lsp::lsp_types::Range;

/// Convert a byte offset in `file` to an LSP [`Position`].
#[must_use]
pub fn position_at_offset(file: &MagoFile, offset: u32) -> Position {
    Position { line: file.line_number(offset), character: file.column_number(offset) }
}

/// Convert an LSP [`Position`] in `file` to a byte offset.
///
/// Returns the file's end-of-content offset if the position is past the end.
#[must_use]
pub fn offset_at_position(file: &MagoFile, position: Position) -> u32 {
    let line_start = file.get_line_start_offset(position.line).unwrap_or(file.size);
    let column = position.character;
    let line_end = file.get_line_end_offset(position.line).unwrap_or(file.size);
    (line_start + column).min(line_end).min(file.size)
}

/// Build an LSP [`Range`] from a `[start, end)` byte half-range.
#[must_use]
pub fn range_at_offsets(file: &MagoFile, start: u32, end: u32) -> Range {
    Range { start: position_at_offset(file, start), end: position_at_offset(file, end) }
}
