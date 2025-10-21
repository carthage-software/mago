use std::borrow::Cow;
use std::io::IsTerminal;

use clap::ColorChoice;
use diffy::PatchFormatter;

use mago_database::change::ChangeLog;
use mago_database::file::File;
use mago_linter::integration::IntegrationSet;
use mago_linter::settings::RulesSettings;
use mago_linter::settings::Settings;
use mago_orchestrator::Orchestrator;
use mago_orchestrator::OrchestratorConfiguration;

use crate::config::Configuration;
use crate::error::Error;

pub mod logger;
pub mod progress;
pub mod version;

pub(crate) fn create_orchestrator<'a>(
    configuration: &'a Configuration,
    color_choice: ColorChoice,
    pedantic_linter: bool,
) -> Orchestrator<'a> {
    let linter_settings = if pedantic_linter {
        Settings {
            php_version: configuration.php_version,
            integrations: IntegrationSet::all(),
            rules: RulesSettings::default(),
        }
    } else {
        Settings {
            php_version: configuration.php_version,
            integrations: IntegrationSet::from_slice(&configuration.linter.integrations),
            rules: configuration.linter.rules.clone(),
        }
    };

    let orchestrator_config = OrchestratorConfiguration {
        php_version: configuration.php_version,
        analyzer_settings: configuration.analyzer.to_settings(configuration.php_version, color_choice),
        linter_settings,
        guard_settings: configuration.guard.settings.clone(),
        formatter_settings: configuration.formatter.settings,
        use_progress_bars: true,
        use_colors: color_choice != ColorChoice::Never,
        paths: configuration.source.paths.iter().map(|p| p.as_ref()).collect(),
        excludes: configuration.source.excludes.iter().map(|p| p.as_ref()).collect(),
        extensions: configuration.source.extensions.iter().map(|e| e.as_ref()).collect(),
        includes: configuration.source.includes.iter().map(|p| p.as_ref()).collect(),
    };

    Orchestrator::new(orchestrator_config)
}

/// Processes the result of a modifying a single file.
///
/// This function compares the original file content with the newly modified content.
/// If there's a difference, it either prints a colorized diff to the console (if in
/// `dry_run` mode) or records an update operation in the provided [`ChangeLog`].
///
/// # Arguments
///
/// * `change_log`: The log where file updates are recorded when not in dry-run mode.
/// * `file`: The original file, used for comparison and context.
/// * `modified_contents`: The newly modified content.
/// * `dry_run`: If `true`, a diff is printed to standard output; otherwise, the
///   change is recorded in the `change_log`.
///
/// # Returns
///
/// Returns `true` if the file content was changed, `false` otherwise.
pub fn apply_update(
    change_log: &ChangeLog,
    file: &File,
    modified_contents: &str,
    dry_run: bool,
    color_choice: ColorChoice,
) -> Result<bool, Error> {
    if file.contents == modified_contents {
        return Ok(false);
    }

    if dry_run {
        let patch = diffy::create_patch(&file.contents, modified_contents);
        let mut formatter = PatchFormatter::new();

        let should_use_colors = match color_choice {
            ColorChoice::Always => true,
            ColorChoice::Never => false,
            ColorChoice::Auto => std::io::stdout().is_terminal(),
        };

        if should_use_colors {
            formatter = formatter.with_color();
        };

        println!("diff of '{}':", file.name);
        println!("{}", formatter.fmt_patch(&patch));
    } else {
        change_log.update(file.id, Cow::Owned(modified_contents.to_owned()))?;
    }

    Ok(true)
}
