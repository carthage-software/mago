use std::io::Write;

use mago_database::ReadDatabase;

use crate::IssueCollection;
use crate::error::ReportingError;
use crate::formatter::Formatter;
use crate::formatter::FormatterConfig;
use crate::internal::Expandable;
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
        let mut expanded_issues = Vec::new();
        for issue in crate::formatter::utils::filter_issues(issues, config, true) {
            expanded_issues.push(issue.expand(database)?);
        }

        let expanded = ExpandedIssueCollection::from_iter(expanded_issues);

        serde_json::to_writer_pretty(writer, &expanded)?;

        Ok(())
    }
}
