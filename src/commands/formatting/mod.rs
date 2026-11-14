use std::process::ExitCode;

use clap::ColorChoice;
use mago_database::DatabaseReader;

use crate::commands::outcome::CommandOutcome;
use crate::config::Configuration;
use crate::error::Error;
use crate::utils::create_orchestrator;

mod args;
mod output;
mod staged;
mod stdin;
mod tool;

pub(crate) use args::FormatArgs;
pub(crate) use tool::FormatTool;

impl FormatArgs {
    pub(crate) fn execute(
        self,
        tool: FormatTool,
        configuration: Configuration,
        color: ColorChoice,
    ) -> Result<CommandOutcome, Error> {
        if !tool.enabled(&configuration) {
            return if self.stdin_input {
                stdin::unchanged().map(CommandOutcome::from)
            } else {
                Ok(ExitCode::SUCCESS.into())
            };
        }
        if self.staged {
            return self.execute_staged(tool, configuration, color).map(CommandOutcome::from);
        }

        let mut orchestrator = create_orchestrator(&configuration, color, false, true, false);
        tool.configure(&mut orchestrator, &configuration);
        if !self.path.is_empty() {
            orchestrator.set_source_paths(self.path.iter().map(|path| path.to_string_lossy().into_owned()));
        }
        if self.stdin_input {
            return self.execute_stdin(tool, orchestrator, &configuration).map(CommandOutcome::from);
        }

        let mut database = orchestrator.load_database(&configuration.source.workspace, false, None, None)?;
        let result = tool.service(&orchestrator, database.read_only(), &configuration).run()?;
        for (file, error) in result.parse_errors() {
            let file = database.get_ref(file)?;
            tracing::error!("Failed to parse file '{}': {error}", mago_bytes::BytesDisplay(&file.name));
        }

        let changed = result.changed_files_count();
        if changed == 0 {
            if result.is_failed() {
                return Ok(ExitCode::FAILURE.into());
            }
            tracing::info!("All files are already formatted.");
            return Ok(ExitCode::SUCCESS.into());
        }
        if self.check {
            tracing::info!(
                "Found {changed} file(s) need formatting. Run the command without '--check' to format them."
            );
            return Ok(ExitCode::FAILURE.into());
        }

        let changes = output::to_change_log(&database, &result, self.dry_run, color)?;
        let changed_files = changes.changed_file_ids()?;
        database.commit(changes, true)?;
        let exit = if self.dry_run {
            tracing::info!("Found {changed} file(s) that need formatting.");
            ExitCode::FAILURE
        } else {
            tracing::info!("Formatted {changed} file(s) successfully.");
            ExitCode::SUCCESS
        };

        CommandOutcome::with_changes(
            if result.is_failed() { ExitCode::FAILURE } else { exit },
            &database,
            changed_files,
        )
    }
}
