//! File listing command implementation.
//!
//! This module implements the `mago list-files` command, which displays all PHP files
//! that would be processed by Mago based on the current configuration. This is a
//! diagnostic tool useful for verifying path configurations and exclusion patterns.
//!
//! # Purpose
//!
//! The command helps users:
//!
//! - **Verify Configuration**: Ensure the correct files are being included/excluded
//! - **Debug Path Issues**: Identify why certain files aren't being processed
//! - **Preview Scope**: See what will be analyzed before running expensive commands
//! - **Integration**: Generate file lists for external tools (with `--zero-terminate`)
//!
//! # Command-Specific Excludes
//!
//! When `--command` is specified, the file list applies that command's specific
//! exclusion patterns in addition to the global source configuration. This shows
//! exactly which files that specific command would process.
//!
//! # Output Formats
//!
//! - **Default**: One file path per line (newline-terminated)
//! - **NUL-terminated** (`-0`): NUL byte separator for safe processing of filenames
//!   with newlines or special characters

use std::process::ExitCode;

use clap::ColorChoice;
use clap::Parser;
use clap::ValueEnum;
use mago_database::DatabaseReader;

use crate::config::Configuration;
use crate::error::Error;
use crate::utils::create_orchestrator;

/// Commands that have specific exclusion patterns.
///
/// Each Mago command can have its own set of files to exclude beyond the global
/// source configuration. This enum represents the commands that support
/// command-specific excludes.
#[derive(ValueEnum, Debug, Clone, Copy)]
#[value(rename_all = "kebab-case")]
enum Command {
    Linter,
    Formatter,
    Analyzer,
    Guard,
}

/// Display all files that will be scanned given the current configuration
///
/// This command is useful for debugging your setup. It prints a full list of
/// all files that would be scanned by Mago given the current configuration.
#[derive(Parser, Debug)]
#[command(
    name = "list-files",
    about = "Display all files that will be scanned.",
    long_about = "Display a list of all files that will be scanned by Mago.\n\n\
                  This can be useful if you want to see which files you might still need\n\
                  to include or exclude in your configuration."
)]
pub struct ListFilesCommand {
    /// Display the list of files for a specific command.
    ///
    /// If given, the exclude rules for that command are taken into
    /// consideration. Otherwise, only the `sources` configuration is used.
    ///
    /// Available commands: linter, formatter, analyzer.
    #[arg(long, value_enum)]
    command: Option<Command>,

    /// Use a NUL byte as the filename terminator.
    ///
    /// If given, instead of a newline a NUL byte will be used to terminate the
    /// filenames. This is useful if filenames might contains newlines themselves.
    #[arg(long, short = '0', default_value_t = false)]
    zero_terminate: bool,
}

impl ListFilesCommand {
    /// Executes the file listing command.
    ///
    /// This method scans the workspace according to the configuration and prints
    /// all discovered PHP files to stdout. The output format depends on the
    /// `--zero-terminate` flag.
    ///
    /// # Process
    ///
    /// 1. Creates an orchestrator with the configuration
    /// 2. Applies command-specific excludes if `--command` was provided
    /// 3. Loads the database by scanning the file system
    /// 4. Prints each file path with the appropriate terminator
    ///
    /// # Arguments
    ///
    /// * `configuration` - The configuration specifying paths and excludes
    /// * `color_choice` - Color settings (not used by this command)
    ///
    /// # Returns
    ///
    /// Always returns `Ok(ExitCode::SUCCESS)` since listing files cannot fail
    /// once the database is loaded successfully.
    ///
    /// # Output Format
    ///
    /// Each file path is printed in the order discovered by the database loader.
    /// Paths are absolute and canonicalized. The terminator character is either
    /// newline (default) or NUL byte (`-0`).
    pub fn execute(self, configuration: Configuration, color_choice: ColorChoice) -> Result<ExitCode, Error> {
        let mut orchestrator = create_orchestrator(&configuration, color_choice, false);
        if let Some(command) = self.command {
            match command {
                Command::Linter => orchestrator.add_exclude_patterns(configuration.linter.excludes.iter()),
                Command::Formatter => orchestrator.add_exclude_patterns(configuration.formatter.excludes.iter()),
                Command::Analyzer => orchestrator.add_exclude_patterns(configuration.analyzer.excludes.iter()),
                Command::Guard => orchestrator.add_exclude_patterns(configuration.guard.excludes.iter()),
            }
        }

        let database = orchestrator.load_database(&configuration.source.workspace, false, None)?;

        for file in database.files() {
            print!("{}{}", file.name, if self.zero_terminate { '\0' } else { '\n' });
        }

        Ok(ExitCode::SUCCESS)
    }
}
