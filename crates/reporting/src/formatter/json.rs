use std::io::Write;

use mago_database::ReadDatabase;

use crate::IssueCollection;
use crate::error::ReportingError;
use crate::formatter::Formatter;
use crate::formatter::FormatterConfig;
use crate::internal::Expandable;

/// Formatter that outputs issues in JSON format.
pub(crate) struct JsonFormatter;

impl Formatter for JsonFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        issues: &IssueCollection,
        database: &ReadDatabase,
        config: &FormatterConfig,
    ) -> Result<(), ReportingError> {
        // Apply filters
        let issues = apply_filters(issues, config);

        // Expand issues with database information
        let expanded = issues.expand(database)?;

        // Write as pretty JSON
        serde_json::to_writer_pretty(writer, &expanded)?;

        Ok(())
    }
}

fn apply_filters(issues: &IssueCollection, config: &FormatterConfig) -> IssueCollection {
    let mut filtered = issues.clone();

    if let Some(min_level) = config.minimum_level {
        filtered = filtered.with_minimum_level(min_level);
    }

    if config.filter_fixable {
        filtered = filtered.with_edits();
    }

    if config.sort {
        filtered = filtered.sorted();
    }

    filtered
}
