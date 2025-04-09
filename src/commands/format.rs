use std::io::Read;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

use mago_formatter::Formatter;
use mago_formatter::settings::FormatSettings;
use mago_interner::ThreadedInterner;
use mago_php_version::PHPVersion;
use mago_source::Source;
use mago_source::SourceCategory;
use mago_source::SourceIdentifier;
use mago_source::SourceManager;
use mago_source::error::SourceError;
use mago_syntax::parser::parse_source;

use crate::config::Configuration;
use crate::error::Error;
use crate::source;
use crate::utils;
use crate::utils::progress::ProgressBarTheme;
use crate::utils::progress::create_progress_bar;
use crate::utils::progress::remove_progress_bar;

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

    /// Perform a dry run to check if files are already formatted.
    #[arg(long, short = 'd', help = "Check if the source files are already formatted without making changes")]
    pub dry_run: bool,

    #[arg(long, short = 'i', help = "Read input from STDIN, format it, and write to STDOUT")]
    pub stdin_input: bool,
}

/// Executes the format command with the provided configuration and options.
///
/// # Arguments
///
/// * `command` - The `FormatCommand` structure containing user-specified options.
/// * `configuration` - The application configuration loaded from file or defaults.
///
/// # Returns
///
/// Exit code: `0` if successful or no changes were needed, `1` if issues were found during the check.
pub async fn execute(command: FormatCommand, mut configuration: Configuration) -> Result<ExitCode, Error> {
    // Initialize the interner for managing identifiers.
    let interner = ThreadedInterner::new();

    configuration.source.excludes.extend(std::mem::take(&mut configuration.format.excludes));

    if command.stdin_input {
        let source = create_source_from_stdin(&interner)?;
        let formatter = Formatter::new(&interner, configuration.php_version, configuration.format.settings);

        return Ok(match formatter.format_source(&source) {
            Ok(formatted) => {
                print!("{}", formatted);

                ExitCode::SUCCESS
            }
            Err(error) => {
                tracing::error!("Failed to format source: {}", error);

                ExitCode::FAILURE
            }
        });
    }

    // Load sources
    let source_manager = if !command.path.is_empty() {
        source::from_paths(&interner, &configuration.source, command.path, false).await?
    } else {
        source::load(&interner, &configuration.source, false, false).await?
    };

    // Format all sources and get the count of changed files.
    let changed =
        format_all(interner, source_manager, configuration.php_version, configuration.format.settings, command.dry_run)
            .await?;

    // Provide feedback and return appropriate exit code.
    if changed == 0 {
        tracing::info!("All source files are already formatted.");

        return Ok(ExitCode::SUCCESS);
    }

    Ok(if command.dry_run {
        tracing::info!("Found {} source files that need formatting.", changed);

        ExitCode::FAILURE
    } else {
        tracing::info!("Formatted {} source files successfully.", changed);

        ExitCode::SUCCESS
    })
}

/// Formats all source files using the provided settings.
///
/// # Arguments
///
/// * `interner` - The interner to manage source identifiers.
/// * `source_manager` - The manager responsible for handling source files.
/// * `settings` - Formatting settings to apply.
/// * `check` - A flag to determine whether to check or apply formatting.
///
/// # Returns
///
/// A result containing the number of changed files or a source error.
#[inline]
async fn format_all(
    interner: ThreadedInterner,
    source_manager: SourceManager,
    php_version: PHPVersion,
    settings: FormatSettings,
    dry_run: bool,
) -> Result<usize, Error> {
    // Collect all user-defined sources.
    let sources: Vec<_> = source_manager.source_ids_for_category(SourceCategory::UserDefined);

    let length = sources.len();
    let progress_bar = create_progress_bar(length, "✨ Formatting", ProgressBarTheme::Green);
    let mut handles = Vec::with_capacity(length);

    // Spawn async tasks to format each source concurrently.
    for source in sources.into_iter() {
        handles.push(tokio::spawn({
            let interner = interner.clone();
            let manager = source_manager.clone();
            let progress_bar = progress_bar.clone();

            async move {
                let result = format_source(&interner, &manager, &source, php_version, settings, dry_run);

                progress_bar.inc(1);

                result
            }
        }));
    }

    let mut changed = 0;

    // Process each formatting task and update progress bar.
    for handle in handles {
        if handle.await?? {
            changed += 1;
        }
    }

    remove_progress_bar(progress_bar);

    Ok(changed)
}

/// Formats a single source file.
///
/// # Arguments
///
/// * `interner` - Reference to the interner for identifier management.
/// * `manager` - Reference to the source manager.
/// * `source` - Identifier of the source file to format.
/// * `settings` - Formatting settings to apply.
/// * `check` - A flag to determine whether to check or apply formatting.
///
/// # Returns
///
/// A result indicating whether the file was changed or an error occurred.
#[inline]
fn format_source(
    interner: &ThreadedInterner,
    manager: &SourceManager,
    source: &SourceIdentifier,
    php_version: PHPVersion,
    settings: FormatSettings,
    dry_run: bool,
) -> Result<bool, Error> {
    // Load the source file.
    let source = manager.load(source)?;

    // Parse the source file to generate an AST.
    let (program, error) = parse_source(interner, &source);

    // Handle parsing errors and perform formatting.
    let changed = match error {
        Some(error) => {
            let source_name = interner.lookup(&source.identifier.0);

            tracing::error!("Skipping formatting for source '{}': {}.", source_name, error);

            false
        }
        None => {
            let formatter = Formatter::new(interner, php_version, settings);
            let formatted = formatter.format(&source, &program);

            utils::apply_changes(interner, manager, &source, formatted, dry_run)?
        }
    };

    Ok(changed)
}

/// Creates a standalone source from standard input.
///
/// # Arguments
///
/// * `interner` - The interner to manage source identifiers.
///
/// # Returns
///
/// A result containing the standalone source or an error.
fn create_source_from_stdin(interner: &ThreadedInterner) -> Result<Source, Error> {
    let mut content = String::new();
    std::io::stdin().read_to_string(&mut content).map_err(|e| Error::Source(SourceError::IOError(e)))?;

    Ok(Source::standalone(interner, "<stdin>", &content))
}
