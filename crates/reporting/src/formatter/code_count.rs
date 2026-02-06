use std::cmp::Ordering;
use std::io::IsTerminal;
use std::io::Write;

use foldhash::HashMap;

use mago_database::ReadDatabase;

use crate::IssueCollection;
use crate::Level;
use crate::error::ReportingError;
use crate::formatter::Formatter;
use crate::formatter::FormatterConfig;

/// Formatter that outputs issue counts by issue code.
pub(crate) struct CodeCountFormatter;

impl Formatter for CodeCountFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        issues: &IssueCollection,
        _database: &ReadDatabase,
        config: &FormatterConfig,
    ) -> Result<(), ReportingError> {
        // Apply filters
        let issues = apply_filters(issues, config);

        // Count occurrences per issue code
        let mut counts: HashMap<String, (usize, Level)> = HashMap::default();

        for issue in issues.iter() {
            let code = issue.code.clone().unwrap_or_else(|| "<unknown>".to_string());

            let entry = counts.entry(code).or_insert((0, issue.level));
            entry.0 += 1;

            // update to highest level if needed
            if issue.level > entry.1 {
                entry.1 = issue.level;
            }
        }

        // Sort by descending count, then by code
        let mut counts_vec: Vec<_> = counts.into_iter().collect();
        counts_vec.sort_by(|(code_a, (count_a, _)), (code_b, (count_b, _))| match count_b.cmp(count_a) {
            Ordering::Equal => code_a.cmp(code_b),
            other => other,
        });

        // Determine if we should use colors
        let use_colors = config.color_choice.should_use_colors(std::io::stdout().is_terminal());

        // Write results
        for (code, (count, level)) in counts_vec {
            if use_colors {
                let ansi_code = level_ansi_code(level);
                writeln!(writer, "\x1b[{ansi_code}m\x1b[1m{level}[{code}]:\x1b[0m {count}")?;
            } else {
                writeln!(writer, "{level}[{code}]: {count}")?;
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
