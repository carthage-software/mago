use std::cmp::Ordering;

use ahash::HashMap;
use termcolor::Color;
use termcolor::ColorSpec;
use termcolor::WriteColor;

use mago_database::ReadDatabase;

use crate::IssueCollection;
use crate::Level;
use crate::error::ReportingError;

pub fn count_format(
    writer: &mut dyn WriteColor,
    _database: &ReadDatabase,
    issues: IssueCollection,
) -> Result<(), ReportingError> {
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

    // Write counts to the writer
    for (level, count) in counts_vec {
        let color = level_color(&level);
        let mut spec = ColorSpec::new();

        writer.set_color(spec.set_fg(Some(color)).set_bold(true))?;
        write!(writer, "{level}: ")?;
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
