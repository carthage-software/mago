use std::process::ExitCode;

use clap::Parser;
use clap::ValueEnum;
use mago_database::DatabaseReader;

use crate::config::Configuration;
use crate::database;
use crate::error::Error;

#[derive(ValueEnum, Debug, Clone, Copy)]
#[value(rename_all = "kebab-case")]
enum Command {
    Linter,
    Formatter,
    Analyzer,
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
    pub fn execute(self, mut configuration: Configuration) -> Result<ExitCode, Error> {
        if let Some(command) = self.command {
            configuration.source.excludes.extend(std::mem::take(match command {
                Command::Linter => &mut configuration.linter.excludes,
                Command::Formatter => &mut configuration.formatter.excludes,
                Command::Analyzer => &mut configuration.analyzer.excludes,
            }));
        }

        let database = database::load_from_configuration(&mut configuration.source, false, None)?;
        for file in database.files() {
            print!("{}{}", file.name, if self.zero_terminate { '\0' } else { '\n' });
        }

        Ok(ExitCode::SUCCESS)
    }
}
