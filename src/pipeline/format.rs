use mago_database::ReadDatabase;
use mago_database::change::ChangeLog;
use mago_formatter::Formatter;
use mago_formatter::settings::FormatSettings;
use mago_php_version::PHPVersion;

use crate::error::Error;
use crate::pipeline::StatelessParallelPipeline;
use crate::pipeline::StatelessReducer;
use crate::utils;

/// The "reduce" step for the formatting pipeline.
///
/// This struct aggregates the boolean results from each parallel formatting
/// task into a final count of how many files were changed.
#[derive(Debug, Clone)]
pub struct FormatReducer;

impl StatelessReducer<bool, usize> for FormatReducer {
    fn reduce(&self, results: Vec<bool>) -> Result<usize, Error> {
        Ok(results.into_iter().filter(|&changed| changed).count())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatMode {
    /// Apply formatting changes to files.
    Format,
    /// Check if files are formatted without making changes.
    Check,
    /// Print a diff of changes without modifying files.
    DryRun,
}

/// Shared, read-only context provided to each parallel formatting task.
#[derive(Clone)]
pub struct FormatContext {
    /// The target PHP version for formatting rules.
    pub php_version: PHPVersion,
    /// The configured settings for the formatter.
    pub settings: FormatSettings,
    /// The mode of operation: format, check, or dry-run.
    pub mode: FormatMode,
    /// A thread-safe log for recording formatting changes.
    pub change_log: ChangeLog,
}

/// The main entry point for running the parallel formatting pipeline.
///
/// This function orchestrates the formatting of all `Host` files in the database
/// using a stateless parallel pipeline, which is highly efficient for tasks that
/// can process each file in isolation.
///
/// # Arguments
///
/// * `database`: The read-only database containing the files to format.
/// * `context`: The shared [`FormatContext`] for the formatting run.
/// * `use_colors`: Whether to use colorized output for diffs in dry-run mode.
///
/// # Returns
///
/// A `Result` containing the total number of files that were changed, or an [`Error`].
pub fn run_format_pipeline(database: ReadDatabase, context: FormatContext, use_colors: bool) -> Result<usize, Error> {
    StatelessParallelPipeline::new("✨ Formatting", database, context, Box::new(FormatReducer)).run(
        |context, arena, file| {
            let formatter = Formatter::new(arena, context.php_version, context.settings);

            match formatter.format_file(&file) {
                Ok(formatted_content) => utils::apply_update(
                    &context.change_log,
                    &file,
                    formatted_content,
                    matches!(context.mode, FormatMode::DryRun),
                    matches!(context.mode, FormatMode::Check),
                    use_colors,
                ),
                Err(parse_error) => {
                    tracing::error!("Formatting failed for '{}': {}.", file.name, parse_error);

                    Ok(false)
                }
            }
        },
    )
}
