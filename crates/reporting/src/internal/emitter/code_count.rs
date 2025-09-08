use std::cmp::Ordering;

use ahash::HashMap;
use termcolor::Color;
use termcolor::ColorSpec;
use termcolor::WriteColor;

use mago_database::ReadDatabase;

use crate::IssueCollection;
use crate::Level;
use crate::error::ReportingError;

pub fn code_count_format(
    writer: &mut dyn WriteColor,
    _database: &ReadDatabase,
    issues: IssueCollection,
) -> Result<(), ReportingError> {
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

    // Write results
    for (code, (count, level)) in counts_vec {
        let color = level_color(&level);
        let mut spec = ColorSpec::new();

        writer.set_color(spec.set_fg(Some(color)).set_bold(true))?;
        write!(writer, "{code}: ")?;
        writer.set_color(spec.set_bold(false))?;
        writeln!(writer, "{count}")?;
        writer.reset()?;
    }

    Ok(())
}

fn level_color(level: &Level) -> Color {
    match level {
        Level::Error => Color::Red,
        Level::Warning => Color::Yellow,
        Level::Note => Color::Blue,
        Level::Help => Color::Green,
    }
}
