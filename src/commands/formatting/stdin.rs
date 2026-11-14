use std::borrow::Cow;
use std::io::Read;
use std::path::Path;
use std::process::ExitCode;

use mago_database::DatabaseReader;
use mago_database::ReadDatabase;
use mago_database::error::DatabaseError;
use mago_database::file::File;
use mago_orchestrator::Orchestrator;
use mago_orchestrator::service::format::FileFormatStatus;

use crate::config::Configuration;
use crate::error::Error;

use super::FormatArgs;
use super::FormatTool;
use super::output::emit_stdin;

impl FormatArgs {
    pub(super) fn execute_stdin(
        self,
        tool: FormatTool,
        mut orchestrator: Orchestrator<'_>,
        configuration: &Configuration,
    ) -> Result<ExitCode, Error> {
        let content = read_stdin()?;
        let Some(filepath) = self.stdin_filepath.as_deref() else {
            let file = File::ephemeral(Cow::Borrowed(b"<stdin>"), Cow::Owned(content));
            let service = tool.service(&orchestrator, ReadDatabase::empty(), configuration);
            return Ok(emit_stdin(rayon::scope(|_| service.format_file(&file))?, &file));
        };

        let name = logical_name(filepath, &configuration.source.workspace);
        orchestrator.set_source_paths([filepath.to_string_lossy().into_owned()]);
        let database = orchestrator.load_database(
            &configuration.source.workspace,
            false,
            None,
            Some((name.clone(), content.clone())),
        )?;
        let Ok(file) = database.get_by_name(name.as_bytes()) else {
            let file = File::ephemeral(Cow::Owned(name.into_bytes()), Cow::Owned(content));
            return Ok(emit_stdin(FileFormatStatus::Unchanged, &file));
        };
        let service = tool.service(&orchestrator, database.read_only(), configuration);
        Ok(emit_stdin(rayon::scope(|_| service.format_file(&file))?, &file))
    }
}

pub(super) fn unchanged() -> Result<ExitCode, Error> {
    let file = File::ephemeral(Cow::Borrowed(b"<stdin>"), Cow::Owned(read_stdin()?));
    Ok(emit_stdin(FileFormatStatus::Unchanged, &file))
}

fn read_stdin() -> Result<Vec<u8>, Error> {
    let mut content = Vec::new();
    std::io::stdin().read_to_end(&mut content).map_err(|error| Error::Database(DatabaseError::IOError(error)))?;
    Ok(content)
}

fn logical_name(filepath: &Path, workspace: &Path) -> String {
    let canonical_filepath = filepath.canonicalize();
    let canonical_workspace = workspace.canonicalize();
    let stripped = match (canonical_filepath.as_ref(), canonical_workspace.as_ref()) {
        (Ok(filepath), Ok(workspace)) => filepath.trim_prefix(workspace),
        _ => filepath.trim_prefix(workspace),
    };

    #[cfg(windows)]
    let mut name = stripped.to_string_lossy().replace('\\', "/");
    #[cfg(not(windows))]
    let mut name = stripped.to_string_lossy().into_owned();

    while let Some(rest) = name.strip_prefix("./") {
        name = rest.to_owned();
    }
    name
}
