use std::process::ExitCode;

use clap::Parser;

use mago_feedback::create_progress_bar;
use mago_feedback::remove_progress_bar;
use mago_feedback::ProgressBarTheme;
use mago_formatter::format;
use mago_formatter::settings::FormatSettings;
use mago_interner::ThreadedInterner;
use mago_parser::parse_source;
use mago_source::SourceIdentifier;
use mago_source::SourceManager;

use crate::config::Configuration;
use crate::error::Error;
use crate::source;
use crate::utils;

/// Represents the `format` command, which is responsible for formatting source files
/// according to specified rules in the configuration file.
#[derive(Parser, Debug)]
#[command(
    name = "format",
    aliases = ["fmt"],
    about = "Format source files using defined rules.",
    long_about = r#"
The `format` command ensures that source files adhere to the formatting rules defined
in the configuration file. Optionally, you can check if files are formatted correctly
without applying changes.
"#
)]
pub struct FormatCommand {
    /// Perform a dry run to check if files are already formatted.
    #[arg(long, short = 'd', help = "Check if the source files are formatted correctly.")]
    pub dry_run: bool,

    /// Specify the width of the printed source code for formatting purposes.
    #[arg(long, short = 'w', help = "The width of the printed source code.", value_name = "WIDTH")]
    pub print_width: Option<usize>,
}

/// Executes the format command with the provided configuration and options.
///
/// # Arguments
/// * `command` - The `FormatCommand` structure containing user-specified options.
/// * `configuration` - The application configuration loaded from file or defaults.
///
/// # Returns
///
/// Exit code: `0` if successful or no changes were needed, `1` if issues were found during the check.
pub async fn execute(command: FormatCommand, mut configuration: Configuration) -> Result<ExitCode, Error> {
    // Initialize the interner for managing identifiers.
    let interner = ThreadedInterner::new();
    // Load sources
    let source_manager = source::load(&interner, &configuration.source, false).await?;

    // Update the print width in configuration if provided.
    if let Some(width) = command.print_width {
        configuration.format.print_width = Some(width);
    }

    // Extract formatting settings from the configuration.
    let settings = configuration.format.get_settings();

    // Format all sources and get the count of changed files.
    let changed = format_all(interner, source_manager, settings, command.dry_run).await?;

    // Provide feedback and return appropriate exit code.
    if changed == 0 {
        mago_feedback::info!("All source files are already formatted.");

        return Ok(ExitCode::SUCCESS);
    }

    Ok(if command.dry_run {
        mago_feedback::info!("Found {} source files that need formatting.", changed);

        ExitCode::FAILURE
    } else {
        mago_feedback::info!("Formatted {} source files successfully.", changed);

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
    settings: FormatSettings,
    dry_run: bool,
) -> Result<usize, Error> {
    // Collect all user-defined sources.
    let sources: Vec<_> = source_manager.user_defined_source_ids().collect();

    let length = sources.len();
    let mut handles = Vec::with_capacity(length);

    // Spawn async tasks to format each source concurrently.
    for source in sources.into_iter() {
        handles.push(tokio::spawn({
            let interner = interner.clone();
            let manager = source_manager.clone();

            async move { format_source(&interner, &manager, &source, settings, dry_run) }
        }));
    }

    let mut changed = 0;
    let progress_bar = create_progress_bar(length, "✨ Formatting", ProgressBarTheme::Magenta);

    // Process each formatting task and update progress bar.
    for handle in handles {
        if handle.await?? {
            changed += 1;
        }

        progress_bar.inc(1);
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
            let formatted = format(settings, interner, &source, &program);

            utils::apply_changes(interner, manager, &source, formatted, dry_run)?
        }
    };

    Ok(changed)
}
