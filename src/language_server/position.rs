//! LSP position ↔ mago byte offset conversion.
//!
//! LSP uses zero-based `(line, character)` coordinates with UTF-16 code units
//! by default. Mago tracks positions as zero-based byte offsets into the file
//! contents, with a precomputed line-start table on `mago_database::file::File`.
//!
//! Positions inside a character round down to its start. Columns past the
//! line's text stop before its line ending.

use mago_database::file::File as MagoFile;
use tower_lsp_server::gen_lsp_types::Position;
use tower_lsp_server::gen_lsp_types::Range;

/// Convert a byte offset in `file` to an LSP [`Position`].
#[must_use]
pub fn position_at_offset(file: &MagoFile, offset: u32) -> Position {
    let offset = offset.min(file.size);
    let line = file.line_number(offset);
    let start = file.get_line_start_offset(line).unwrap_or(0);
    let end = line_end_offset(file, line);
    let character = utf16_offset_at_byte(&file.contents[start as usize..end as usize], offset.min(end) - start);
    Position { line, character }
}

/// Convert an LSP [`Position`] in `file` to a byte offset.
///
/// Returns the file's end-of-content offset if the position is past the end.
#[must_use]
pub fn offset_at_position(file: &MagoFile, position: Position) -> u32 {
    let Some(start) = file.get_line_start_offset(position.line) else {
        return file.size;
    };
    let end = line_end_offset(file, position.line);
    let bytes = &file.contents[start as usize..end as usize];
    if bytes.is_ascii() {
        return start + position.character.min(end - start);
    }

    let mut offset = start;
    let mut units = 0;
    for (byte_width, utf16_width) in character_widths(bytes) {
        if units + utf16_width > position.character {
            break;
        }
        offset += byte_width as u32;
        units += utf16_width;
    }
    offset
}

/// Return the end of a line's text, before LF, CRLF, or CR.
#[must_use]
pub(super) fn line_end_offset(file: &MagoFile, line: u32) -> u32 {
    let start = file.get_line_start_offset(line).unwrap_or(file.size);
    let end = file.get_line_end_offset(line).unwrap_or(file.size);
    if end > start && file.contents[end as usize - 1] == b'\r' { end - 1 } else { end }
}

/// Convert a byte offset to UTF-16 units, rounding down inside a character.
#[must_use]
pub(super) fn utf16_offset_at_byte(bytes: &[u8], offset: u32) -> u32 {
    let end = (offset as usize).min(bytes.len());
    if bytes[..end].is_ascii() {
        return end as u32;
    }

    let mut offset = 0;
    let mut units = 0;
    for (byte_width, utf16_width) in character_widths(bytes) {
        offset += byte_width;
        if offset > end {
            break;
        }
        units += utf16_width;
    }
    units
}

// Invalid UTF-8 counts as replacement characters, as in `from_utf8_lossy`.
fn character_widths(bytes: &[u8]) -> impl Iterator<Item = (usize, u32)> {
    bytes.utf8_chunks().flat_map(|chunk| {
        chunk
            .valid()
            .chars()
            .map(|character| (character.len_utf8(), character.len_utf16() as u32))
            .chain((!chunk.invalid().is_empty()).then_some((chunk.invalid().len(), 1)))
    })
}

/// Build an LSP [`Range`] from a `[start, end)` byte half-range.
#[must_use]
pub fn range_at_offsets(file: &MagoFile, start: u32, end: u32) -> Range {
    Range { start: position_at_offset(file, start), end: position_at_offset(file, end) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(contents: &'static [u8]) -> MagoFile {
        MagoFile::ephemeral(b"test.php".as_slice().into(), contents.into())
    }

    #[test]
    fn utf16_positions_round_trip_at_character_boundaries() {
        let file = file("aé😀z\r\nΩ".as_bytes());
        for (offset, line, character) in [(0, 0, 0), (1, 0, 1), (3, 0, 2), (7, 0, 4), (8, 0, 5), (10, 1, 0), (12, 1, 1)]
        {
            let position = Position { line, character };
            assert_eq!(position_at_offset(&file, offset), position);
            assert_eq!(offset_at_position(&file, position), offset);
        }
    }

    #[test]
    fn positions_clamp_at_characters_line_endings_and_eof() {
        let file = file("aé😀z\r\nΩ".as_bytes());
        for (offset, character) in [(2, 1), (4, 2), (5, 2), (6, 2), (9, 5)] {
            assert_eq!(position_at_offset(&file, offset), Position::new(0, character));
        }
        assert_eq!(offset_at_position(&file, Position::new(0, 3)), 3);
        assert_eq!(offset_at_position(&file, Position::new(0, u32::MAX)), 8);
        assert_eq!(offset_at_position(&file, Position::new(u32::MAX, u32::MAX)), 12);
        assert_eq!(position_at_offset(&file, u32::MAX), Position::new(1, 1));
    }

    #[test]
    fn ascii_empty_and_trailing_lines_have_valid_positions() {
        for contents in [b"".as_slice(), b"ab", b"ab\n", b"ab\r\n", b"ab\r"] {
            let file = file(contents);
            let end = if contents.is_empty() { 0 } else { 2 };
            assert_eq!(offset_at_position(&file, Position::new(0, u32::MAX)), end);
            let eof = position_at_offset(&file, file.size);
            assert_eq!(offset_at_position(&file, eof), file.size);
        }
    }

    #[test]
    fn invalid_utf8_uses_replacement_character_widths() {
        let file = file(b"a\xe2\x82z");
        assert_eq!(position_at_offset(&file, 2), Position::new(0, 1));
        assert_eq!(position_at_offset(&file, 3), Position::new(0, 2));
        assert_eq!(offset_at_position(&file, Position::new(0, 2)), 3);
    }
}
