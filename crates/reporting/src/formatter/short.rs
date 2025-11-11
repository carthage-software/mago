use std::io::Write;

use codespan_reporting::term::Config;
use codespan_reporting::term::DisplayStyle;

use mago_database::ReadDatabase;

use crate::IssueCollection;
use crate::error::ReportingError;
use crate::formatter::Formatter;
use crate::formatter::FormatterConfig;

/// Formatter that outputs issues in short diagnostic format with minimal context.
pub(crate) struct ShortFormatter;

impl Formatter for ShortFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        issues: &IssueCollection,
        database: &ReadDatabase,
        config: &FormatterConfig,
    ) -> Result<(), ReportingError> {
        // Delegate to rich formatter with short display style
        super::rich::codespan_format_with_config(
            writer,
            issues,
            database,
            config,
            Config { display_style: DisplayStyle::Short, ..Default::default() },
        )
    }
}
