//! Code formatting command implementation.
//!
//! This module implements the `mago format` (aliased as `mago fmt`) command, which
//! automatically formats PHP code according to configured style rules. The formatter
//! ensures consistent code style across the entire codebase.
//!
//! # Formatting Modes
//!
//! The formatter supports three distinct modes:
//!
//! - **In-place Formatting** (default): Modifies files directly on disk
//! - **Check Mode** (`--check`): Validates formatting without making changes (CI-friendly)
//! - **Dry Run** (`--dry-run`): Shows what would change via diff without modifying files
//! - **STDIN Mode** (`--stdin-input`): Reads from stdin, writes formatted code to stdout
//!
//! # Configuration
//!
//! Formatting style is configured in `mago.toml` under the `[formatter]` section:
//!
//! - **print-width**: Maximum line length
//! - **tab-width**: Number of spaces per indentation level
//! - **use-tabs**: Whether to use tabs instead of spaces
//! - Additional style preferences for braces, spacing, etc.
//!
//! # Output and Reporting
//!
//! - **In-place**: Prints summary of formatted files
//! - **Check**: Exits with failure code if files need formatting
//! - **Dry run**: Displays colorized diffs of proposed changes
//! - **STDIN**: Outputs formatted code directly
//!
//! # Use Cases
//!
//! - Pre-commit hooks to ensure consistent formatting
//! - CI checks to validate code style compliance
//! - Editor integration for format-on-save
//! - Bulk formatting after style guide changes

use std::borrow::Cow;
use std::io::Read;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::ColorChoice;
use clap::Parser;

use mago_database::DatabaseReader;
use mago_database::change::ChangeLog;
use mago_database::error::DatabaseError;
use mago_database::file::File;
use mago_orchestrator::service::format::FileFormatStatus;

use crate::config::Configuration;
use crate::error::Error;
use crate::utils;
use crate::utils::create_orchestrator;

/// Command for formatting PHP source files according to style rules.
///
/// This command applies consistent formatting to PHP code based on the configured
/// style preferences. It supports multiple modes including in-place formatting,
/// check mode for CI, and dry-run mode for previewing changes.
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
    #[arg()]
    pub path: Vec<PathBuf>,

    /// Perform a dry run, printing a diff without modifying files.
    ///
    /// This will calculate and print a diff of any changes that would be made.
    /// No files will be modified on disk.
    #[arg(long, short = 'd', conflicts_with_all = ["check", "stdin_input"], alias = "diff")]
    pub dry_run: bool,

    /// Check if the source files are formatted.
    ///
    /// This flag is ideal for CI environments. The command will exit with a
    /// success code (`0`) if all files are formatted, and a failure code (`1`)
    /// if any files would be changed. No output is printed to `stdout`.
    #[arg(long, short = 'c', conflicts_with_all = ["dry_run", "stdin_input"])]
    pub check: bool,

    /// Read input from STDIN, format it, and write to STDOUT.
    ///
    /// This flag allows you to pipe PHP code directly into the formatter.
    ///
    /// When using this option, the formatter reads from standard input,
    /// formats the code according to the configuration, and outputs the
    /// formatted code to standard output. This is useful for integrating
    /// with other tools or for quick formatting tasks without modifying files.
    #[arg(long, short = 'i', conflicts_with_all = ["dry_run", "check", "path"])]
    pub stdin_input: bool,
}

impl FormatCommand {
    /// Executes the formatting command.
    ///
    /// This method handles all formatting modes (in-place, check, dry-run, stdin)
    /// and orchestrates the complete formatting workflow:
    ///
    /// 1. **Mode Selection**: Determines which mode to use based on flags
    /// 2. **STDIN Handling**: If `--stdin-input`, reads from stdin and formats immediately
    /// 3. **Database Loading**: Scans workspace for PHP files (unless stdin mode)
    /// 4. **Service Creation**: Creates formatting service with configuration
    /// 5. **Formatting**: Processes each file according to the selected mode
    /// 6. **Reporting**: Outputs results (summary, diffs, or exit code)
    ///
    /// # Arguments
    ///
    /// * `configuration` - The configuration containing formatter settings
    /// * `color_choice` - Whether to use colored output for diffs
    ///
    /// # Returns
    ///
    /// - `Ok(ExitCode::SUCCESS)` if formatting succeeded or no changes needed
    /// - `Ok(ExitCode::FAILURE)` in check mode if files need formatting
    /// - `Err(Error)` if database loading or formatting failed
    ///
    /// # Modes
    ///
    /// - **In-place** (default): Writes formatted code back to files
    /// - **Check** (`--check`): Returns failure if any file needs formatting
    /// - **Dry run** (`--dry-run`): Prints diffs without modifying files
    /// - **STDIN** (`--stdin-input`): Formats input from stdin to stdout
    pub fn execute(self, configuration: Configuration, color_choice: ColorChoice) -> Result<ExitCode, Error> {
        let mut orchestrator = create_orchestrator(&configuration, color_choice, false);
        orchestrator.add_exclude_patterns(configuration.formatter.excludes.iter());
        if !self.path.is_empty() {
            orchestrator.set_source_paths(self.path.iter());
        }

        let mut database = orchestrator.load_database(&configuration.source.workspace, false, None)?;
        let service = orchestrator.get_format_service(database.read_only());

        if self.stdin_input {
            let file = Self::create_file_from_stdin()?;
            let status = service.format_file(&file)?;

            let exit_code = match status {
                FileFormatStatus::Unchanged => {
                    print!("{}", file.contents);

                    ExitCode::SUCCESS
                }
                FileFormatStatus::Changed(new_content) => {
                    print!("{new_content}");

                    ExitCode::SUCCESS
                }
                FileFormatStatus::FailedToParse(parse_error) => {
                    tracing::error!("Failed to parse input: {}", parse_error);
                    ExitCode::FAILURE
                }
            };

            return Ok(exit_code);
        }

        let result = service.run()?;

        for (file_id, parse_error) in result.parse_errors() {
            let file = database.get_ref(file_id)?;

            tracing::error!("Failed to parse file '{}': {parse_error}", file.name);
        }

        let changed_files_count = result.changed_files_count();

        if changed_files_count == 0 {
            tracing::info!("All files are already formatted.");

            return Ok(ExitCode::SUCCESS);
        }

        if self.check {
            tracing::info!(
                "Found {changed_files_count} file(s) need formatting. Run the command without '--check' to format them.",
            );

            return Ok(ExitCode::FAILURE);
        }

        let change_log = ChangeLog::new();
        for (file_id, new_content) in result.changed_files() {
            let file = database.get_ref(file_id)?;

            utils::apply_update(&change_log, file, new_content, self.dry_run, color_choice)?;
        }

        database.commit(change_log, true)?;

        let exit_code = if self.dry_run {
            tracing::info!("Found {changed_files_count} file(s) that need formatting.");

            ExitCode::FAILURE
        } else {
            tracing::info!("Formatted {changed_files_count} file(s) successfully.");

            ExitCode::SUCCESS
        };

        Ok(exit_code)
    }

    /// Creates an ephemeral file from standard input.
    fn create_file_from_stdin() -> Result<File, Error> {
        let mut content = String::new();
        std::io::stdin().read_to_string(&mut content).map_err(|e| Error::Database(DatabaseError::IOError(e)))?;

        Ok(File::ephemeral(Cow::Borrowed("<stdin>"), Cow::Owned(content)))
    }
}
