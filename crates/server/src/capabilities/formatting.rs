//! Reformat a file or source range with the workspace's formatter.
//!
//! Runs the file through [`mago_formatter::Formatter`] with the workspace's PHP
//! version and [`FormatSettings`]. Returns `None` when formatting changes
//! nothing.

use std::sync::Arc;

use mago_allocator::LocalArena;
use mago_database::file::File;
use mago_database::file::FileId;
use mago_formatter::Formatter;

use crate::Server;
use crate::domain::FormattedDocument;
use crate::domain::Range;
use crate::domain::TextReplacement;

impl Server {
    /// Format the statements enclosing `range`, or return `None` if no edit is needed.
    #[must_use]
    pub fn get_range_formatting(&self, file_id: FileId, range: Range) -> Option<TextReplacement> {
        let file = self.formattable_file(file_id)?;
        let arena = LocalArena::new();
        let formatter = Formatter::new(&arena, self.php_version, self.formatter);
        let edit = formatter.format_range(&file, range.start..range.end).ok()??;
        Some(TextReplacement {
            range: Range::new(edit.range.start, edit.range.end),
            new_text: String::from_utf8_lossy(edit.new_text).into_owned(),
        })
    }

    /// Reformat `file_id`, or `None` if it's already formatted (or unknown).
    #[must_use]
    pub fn get_formatting(&self, file_id: FileId) -> Option<FormattedDocument> {
        let file = self.formattable_file(file_id)?;
        let arena = LocalArena::new();
        let formatter = Formatter::new(&arena, self.php_version, self.formatter);

        let output = formatter.format_file(&file).ok()?;
        if output == file.contents.as_ref() {
            return None;
        }

        Some(FormattedDocument { new_text: String::from_utf8_lossy(output).into_owned() })
    }

    fn formattable_file(&self, file_id: FileId) -> Option<Arc<File>> {
        self.editable_file(file_id)
            .filter(|file| self.formatter_enabled && !self.formatter_excludes.excludes_file(file))
    }
}
