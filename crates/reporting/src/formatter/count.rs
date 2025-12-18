use std::cmp::Ordering;
use std::io::Write;

use ahash::HashMap;

use mago_database::ReadDatabase;

use crate::IssueCollection;
use crate::Level;
use crate::error::ReportingError;
use crate::formatter::Formatter;
use crate::formatter::FormatterConfig;

/// Formatter that outputs issue counts by severity level.
pub(crate) struct CountFormatter;

impl Formatter for CountFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        issues: &IssueCollection,
        _database: &ReadDatabase,
        config: &FormatterConfig,
    ) -> Result<(), ReportingError> {
        // Apply filters
        let issues = apply_filters(issues, config);

        // Count occurrences of each issue level
        let mut counts = HashMap::default();
        issues.iter().for_each(|issue| {
            *counts.entry(issue.level).or_insert(0) += 1;
        });

        let mut counts_vec: Vec<_> = counts.into_iter().collect();
        counts_vec.sort_by(|(level_a, count_a), (level_b, count_b)| match count_b.cmp(count_a) {
            Ordering::Equal => level_a.cmp(level_b),
            other => other,
        });

        // Determine if we should use colors
        let use_colors = config.color_choice.should_use_colors(atty::is(atty::Stream::Stdout));

        // Write counts to the writer
        for (level, count) in counts_vec {
            if use_colors {
                let ansi_code = level_ansi_code(level);
                writeln!(writer, "\x1b[{ansi_code}m\x1b[1m{level}:\x1b[0m {count}")?;
            } else {
                writeln!(writer, "{level}: {count}")?;
            }
        }

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

fn level_ansi_code(level: Level) -> &'static str {
    match level {
        Level::Error => "31",   // Red
        Level::Warning => "33", // Yellow
        Level::Note => "34",    // Blue
        Level::Help => "32",    // Green
    }
}
