use std::io::Write;
use std::process::ExitCode;

use clap::ColorChoice;
use mago_database::Database;
use mago_database::DatabaseReader;
use mago_database::change::ChangeLog;
use mago_database::file::File;
use mago_orchestrator::service::format::FileFormatStatus;
use mago_orchestrator::service::format::FormatResult;

use crate::EXIT_CODE_ERROR;
use crate::error::Error;
use crate::utils;

pub(super) fn to_change_log(
    database: &Database<'_>,
    result: &FormatResult,
    dry_run: bool,
    color: ColorChoice,
) -> Result<ChangeLog, Error> {
    let changes = ChangeLog::new();
    for (id, content) in result.changed_files() {
        utils::apply_update(&changes, database.get_ref(id)?, content, dry_run, color)?;
    }
    Ok(changes)
}

pub(super) fn emit_stdin(status: FileFormatStatus, file: &File) -> ExitCode {
    match status {
        FileFormatStatus::Unchanged => {
            let _ = std::io::stdout().write_all(&file.contents);
            ExitCode::SUCCESS
        }
        FileFormatStatus::Changed(content) => {
            let _ = std::io::stdout().write_all(&content);
            ExitCode::SUCCESS
        }
        FileFormatStatus::FailedToParse(error) => {
            tracing::error!("Failed to parse {}: {error}", mago_bytes::BytesDisplay(&file.name));
            ExitCode::from(EXIT_CODE_ERROR)
        }
    }
}
