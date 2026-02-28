use std::io::Read;
use std::path::Path;

use mago_database::error::DatabaseError;
use mago_orchestrator::Orchestrator;

use crate::error::Error;

/// When `--stdin-input` is used: validates exactly one path, reads stdin,
/// computes workspace-relative logical name (with `./` normalization),
/// sets the orchestrator source path to that single path, and returns
/// `Some((logical_name, content))` for `load_database(..., stdin_override)`.
/// Otherwise returns `None`.
pub fn resolve_stdin_override(
    stdin_input: bool,
    path: &[std::path::PathBuf],
    workspace: &Path,
    orchestrator: &mut Orchestrator,
) -> Result<Option<(String, String)>, Error> {
    if !stdin_input {
        return Ok(None);
    }
    if path.len() != 1 {
        return Err(Error::Database(DatabaseError::IOError(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "When using --stdin-input, exactly one file path must be provided.",
        ))));
    }
    let path = &path[0];
    let mut content = String::new();
    std::io::stdin().read_to_string(&mut content).map_err(|e| Error::Database(DatabaseError::IOError(e)))?;

    #[cfg(windows)]
    let mut logical_name = path.strip_prefix(workspace).unwrap_or(path.as_path()).to_string_lossy().replace('\\', "/");
    #[cfg(not(windows))]
    let mut logical_name = path.strip_prefix(workspace).unwrap_or(path.as_path()).to_string_lossy().into_owned();
    if logical_name.starts_with("./") {
        logical_name = logical_name.split_off(2);
    }

    orchestrator.set_source_paths([path.to_string_lossy().to_string()]);
    Ok(Some((logical_name, content)))
}

/// Sets the orchestrator source paths from the given path list.
/// Call when not using stdin and the path list is non-empty (e.g. after
/// handling a possible `--staged` branch in analyze/lint).
pub fn set_source_paths_from_paths(orchestrator: &mut Orchestrator, paths: &[std::path::PathBuf]) {
    if paths.is_empty() {
        return;
    }
    orchestrator.set_source_paths(paths.iter().map(|p| p.to_string_lossy().to_string()));
}
