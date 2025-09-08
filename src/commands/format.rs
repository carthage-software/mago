use std::borrow::Cow;
use std::io::Read;
use std::path::PathBuf;
use std::process::ExitCode;

use bumpalo::Bump;
use clap::Parser;

use mago_database::change::ChangeLog;
use mago_database::error::DatabaseError;
use mago_database::file::File;
use mago_formatter::Formatter;

use crate::config::Configuration;
use crate::database;
use crate::error::Error;
use crate::pipeline::format::FormatContext;
use crate::pipeline::format::FormatMode;
use crate::pipeline::format::run_format_pipeline;

/// Represents the `format` command, which is responsible for formatting source files
/// according to specified rules in the configuration file.
#[derive(Parser, Debug)]
#[command(
    name = "format",
    aliases = ["fmt"],
    about = "Format source files to match defined style rules",
    long_about = r#"
The `format` command applies consistent formatting to source files based on the rules defined in the configuration file.

This command helps maintain a consistent codebase style, improving readability and collaboration.
"#
)]
pub struct FormatCommand {
    /// Format specific files or directories, overriding the source configuration.
    #[arg(help = "Format specific files or directories, overriding the source configuration")]
    pub path: Vec<PathBuf>,

    /// Perform a dry run, printing a diff without modifying files.
    ///
    /// This will calculate and print a diff of any changes that would be made.
    /// No files will be modified on disk.
    #[arg(
        long,
        short = 'd',
        help = "Print a diff of changes without modifying files",
        conflicts_with_all = ["check", "stdin_input"],
        alias = "diff"
    )]
    pub dry_run: bool,

    /// Check if the source files are formatted.
    ///
    /// This flag is ideal for CI environments. The command will exit with a
    /// success code (`0`) if all files are formatted, and a failure code (`1`)
    /// if any files would be changed. No output is printed to `stdout`.
    #[arg(
        long,
        short = 'c',
        help = "Check if files are formatted, exiting with a non-zero status code on changes",
        conflicts_with_all = ["dry_run", "stdin_input"],
    )]
    pub check: bool,

    #[arg(
        long,
        short = 'i',
        help = "Read input from STDIN, format it, and write to STDOUT",
        conflicts_with_all = ["dry_run", "check", "path"],
    )]
    pub stdin_input: bool,
}

impl FormatCommand {
    /// Executes the format command with the provided configuration and options.
    ///
    /// # Arguments
    ///
    /// * `configuration` - The application configuration loaded from file or defaults.
    /// * `use_colors` - Whether to use colored output for diffs in dry-run mode.
    ///
    /// # Returns
    ///
    /// Exit code: `0` if successful or no changes were needed, `1` if issues were found during the check.
    pub fn execute(self, mut configuration: Configuration, use_colors: bool) -> Result<ExitCode, Error> {
        configuration.source.excludes.extend(std::mem::take(&mut configuration.formatter.excludes));

        if self.stdin_input {
            let arena = Bump::new();
            let file = Self::create_file_from_stdin()?;
            let formatter = Formatter::new(&arena, configuration.php_version, configuration.formatter.settings);
            return Ok(match formatter.format_file(&file) {
                Ok(formatted) => {
                    print!("{formatted}");
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    tracing::error!("Failed to format input: {}", error);
                    ExitCode::FAILURE
                }
            });
        }

        let mut database = if !self.path.is_empty() {
            database::load_from_paths(&mut configuration.source, self.path, None)?
        } else {
            database::load_from_configuration(&mut configuration.source, false, None)?
        };

        // 1. Create the shared ChangeLog and context for the pipeline.
        let change_log = ChangeLog::new();
        let shared_context = FormatContext {
            php_version: configuration.php_version,
            settings: configuration.formatter.settings,
            mode: if self.dry_run {
                FormatMode::DryRun
            } else if self.check {
                FormatMode::Check
            } else {
                FormatMode::Format
            },
            change_log: change_log.clone(),
        };

        let changed_count = run_format_pipeline(database.read_only(), shared_context, use_colors)?;

        if !self.dry_run {
            database.commit(change_log, true)?;
        }

        if changed_count == 0 {
            tracing::info!("All files are already formatted.");
            return Ok(ExitCode::SUCCESS);
        }

        Ok(if self.dry_run || self.check {
            tracing::info!("Found {} file(s) that need formatting.", changed_count);
            ExitCode::FAILURE
        } else {
            tracing::info!("Formatted {} file(s) successfully.", changed_count);
            ExitCode::SUCCESS
        })
    }

    /// Creates an ephemeral file from standard input.
    fn create_file_from_stdin() -> Result<File, Error> {
        let mut content = String::new();
        std::io::stdin().read_to_string(&mut content).map_err(|e| Error::Database(DatabaseError::IOError(e)))?;

        Ok(File::ephemeral(Cow::Borrowed("<stdin>"), Cow::Owned(content)))
    }
}
