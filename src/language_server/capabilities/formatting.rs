//! `textDocument/formatting`.
//!
//! Runs the file through [`mago_formatter::Formatter`] and returns a single
//! whole-document [`TextEdit`] when the formatter changes anything. Range
//! formatting is intentionally not implemented yet; the formatter operates
//! on whole files.

use bumpalo::Bump;

use mago_database::file::File as MagoFile;
use mago_formatter::Formatter;
use mago_formatter::settings::FormatSettings;
use mago_php_version::PHPVersion;
use tower_lsp::lsp_types::Position;
use tower_lsp::lsp_types::Range;
use tower_lsp::lsp_types::TextEdit;

pub fn compute(file: &MagoFile) -> Option<TextEdit> {
    let arena = Bump::new();
    let formatter = Formatter::new(&arena, PHPVersion::LATEST, FormatSettings::default());

    let formatted = formatter.format_file(file).ok()?;
    if formatted == file.contents.as_ref() {
        return None;
    }

    let end_line = if file.lines.is_empty() { 0 } else { (file.lines.len() - 1) as u32 };
    let end_character = file.size - file.get_line_start_offset(end_line).unwrap_or(0);

    Some(TextEdit {
        range: Range {
            start: Position { line: 0, character: 0 },
            end: Position { line: end_line, character: end_character },
        },
        new_text: formatted.to_string(),
    })
}
