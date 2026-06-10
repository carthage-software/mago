//! `get_formatting`: reformat a whole file with the workspace's formatter.
//!
//! Runs the file through [`mago_formatter::Formatter`] with the workspace's PHP
//! version and [`FormatSettings`]. Returns `None` when formatting changes
//! nothing. Range formatting isn't supported; the formatter works whole-file.

use mago_allocator::LocalArena;
use mago_database::DatabaseReader;
use mago_database::file::FileId;
use mago_formatter::Formatter;

use crate::Server;
use crate::domain::FormattedDocument;

impl Server {
    /// Reformat `file_id`, or `None` if it's already formatted (or unknown).
    #[must_use]
    pub fn get_formatting(&self, file_id: FileId) -> Option<FormattedDocument> {
        let file = self.database().get(&file_id).ok()?;
        let arena = LocalArena::new();
        let formatter = Formatter::new(&arena, self.php_version, self.formatter);

        let output = formatter.format_file(&file).ok()?;
        if output == file.contents.as_ref() {
            return None;
        }

        Some(FormattedDocument { new_text: String::from_utf8_lossy(output).into_owned() })
    }
}
