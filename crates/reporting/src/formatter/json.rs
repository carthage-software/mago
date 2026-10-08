use std::io::Write;

use mago_database::ReadDatabase;

use crate::IssueCollection;
use crate::error::ReportingError;
use crate::formatter::Formatter;
use crate::formatter::FormatterConfig;
use crate::internal::ExpandedIssueCollection;

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
        let issues = crate::formatter::utils::filter_issues(issues, config, true).collect();
        let expanded = ExpandedIssueCollection::new(issues, database)?;

        crate::formatter::utils::write_pretty_json(writer, &expanded)?;

        Ok(())
    }
}
