use std::process::ExitCode;

use clap::ColorChoice;
use mago_database::DatabaseReader;
use mago_database::ReadDatabase;
use mago_orchestrator::service::format::FileFormatStatus;

use crate::config::Configuration;
use crate::error::Error;
use crate::utils::create_orchestrator;
use crate::utils::git;

use super::FormatArgs;
use super::FormatTool;

impl FormatArgs {
    pub(super) fn execute_staged(
        self,
        tool: FormatTool,
        configuration: Configuration,
        color: ColorChoice,
    ) -> Result<ExitCode, Error> {
        let workspace = &configuration.source.workspace;
        let mut orchestrator = create_orchestrator(&configuration, color, false, true, false);
        tool.configure(&mut orchestrator, &configuration);
        let database = orchestrator.load_database(workspace, false, None, None)?;
        let paths = git::get_staged_file_paths(workspace)?;
        if paths.is_empty() {
            tracing::info!("No staged files to format.");
            return Ok(ExitCode::SUCCESS);
        }

        let mut changed = 0;
        let mut failed = false;
        for path in paths {
            let absolute = workspace.join(&path);
            let canonical = absolute.canonicalize().unwrap_or(absolute);
            if database.get_by_path(&canonical).is_err() {
                continue;
            }

            let file = git::get_staged_file(workspace, &path)?;
            match tool.service(&orchestrator, ReadDatabase::empty(), &configuration).format_file(&file)? {
                FileFormatStatus::Unchanged => {}
                FileFormatStatus::Changed(content) => {
                    git::update_staged_file(workspace, &path, content)?;
                    changed += 1;
                }
                FileFormatStatus::FailedToParse(error) => {
                    failed = true;
                    tracing::error!("Failed to parse staged file '{}': {error}", path.display());
                }
            }
        }

        if changed == 0 {
            if failed {
                return Ok(ExitCode::FAILURE);
            }
            tracing::info!("All staged files are already formatted.");
            return Ok(ExitCode::SUCCESS);
        }
        tracing::info!("Formatted and re-staged {changed} file(s).");
        Ok(if failed { ExitCode::FAILURE } else { ExitCode::SUCCESS })
    }
}
