//! Git utilities for staged file operations.
//!
//! This module provides helper functions for interacting with git repositories,
//! specifically for the `--staged` formatting feature that allows formatting
//! staged files in pre-commit hooks.

use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use mago_database::Database;
use mago_database::DatabaseReader;
use mago_database::error::DatabaseError;
use mago_database::file::FileId;

use crate::error::Error;

/// Get staged file paths relative to the workspace.
///
/// This function is used by `--staged` flags in lint and analyze commands
/// to filter analysis to only staged files.
///
/// # Arguments
///
/// * `workspace` - The git repository root directory
///
/// # Returns
///
/// A vector of staged file paths (relative to workspace), or an error if
/// not in a git repository.
pub fn get_staged_file_paths(workspace: &Path) -> Result<Vec<PathBuf>, Error> {
    if !is_git_repository(workspace) {
        return Err(Error::NotAGitRepository);
    }

    get_staged_files(workspace)
}

/// Get staged files that are clean (no unstaged changes) as file IDs.
///
/// This function combines multiple git operations and database lookups into
/// a single convenient function for the `--staged` formatting feature:
///
/// 1. Checks if we're in a git repository
/// 2. Gets all staged files
/// 3. Gets files with unstaged changes
/// 4. For each staged file, checks it doesn't have unstaged changes
/// 5. Resolves each staged file path to a database FileId
///
/// # Arguments
///
/// * `workspace` - The git repository root directory
/// * `database` - The loaded database to look up file IDs
///
/// # Returns
///
/// A vector of FileIds for staged files that have no unstaged changes,
/// or an error if:
/// - Not in a git repository
/// - A staged file has unstaged changes (partial staging)
///
/// # Errors
///
/// Returns `Error::NotAGitRepository` if not in a git repository.
/// Returns `Error::StagedFileHasUnstagedChanges` if any staged file
/// has unstaged modifications (which would cause data loss).
pub fn get_staged_clean_files(workspace: &Path, database: &Database) -> Result<Vec<FileId>, Error> {
    if !is_git_repository(workspace) {
        return Err(Error::NotAGitRepository);
    }

    let staged_files = get_staged_files(workspace)?;
    if staged_files.is_empty() {
        return Ok(Vec::new());
    }

    let files_with_unstaged = get_files_with_unstaged_changes(workspace)?;

    let mut file_ids = Vec::with_capacity(staged_files.len());
    for staged_file in staged_files {
        if files_with_unstaged.contains(&staged_file) {
            return Err(Error::StagedFileHasUnstagedChanges(staged_file.display().to_string()));
        }

        let absolute_path = workspace.join(&staged_file);
        let canonical_path = absolute_path.canonicalize().unwrap_or(absolute_path);

        if let Ok(file) = database.get_by_path(&canonical_path) {
            file_ids.push(file.id);
        }
    }

    Ok(file_ids)
}

/// Stage multiple files at once by their file IDs.
///
/// This function looks up file paths from the database and runs
/// `git add -- <files...>` to stage all specified files in a single git invocation.
///
/// # Arguments
///
/// * `workspace` - The git repository root directory
/// * `database` - The database to look up file paths from
/// * `file_ids` - Iterator of file IDs to stage
///
/// # Returns
///
/// `Ok(())` on success, or an error if the git command fails.
pub fn stage_files<I>(workspace: &Path, database: &Database, file_ids: I) -> Result<(), Error>
where
    I: IntoIterator<Item = FileId>,
{
    let paths: Vec<PathBuf> = file_ids
        .into_iter()
        .filter_map(|id| database.get_ref(&id).ok())
        .map(|file| PathBuf::from(&*file.name))
        .collect();

    if paths.is_empty() {
        return Ok(());
    }

    let mut cmd = Command::new("git");
    cmd.args(["add", "--"]);
    for path in &paths {
        cmd.arg(path);
    }

    let status = cmd.current_dir(workspace).status().map_err(|e| Error::Database(DatabaseError::IOError(e)))?;

    if !status.success() {
        return Err(Error::Database(DatabaseError::IOError(std::io::Error::other("git add failed"))));
    }

    Ok(())
}

/// Check if we're inside a git repository.
///
/// This function runs `git rev-parse --git-dir` to determine if the given
/// workspace is inside a git repository.
///
/// # Arguments
///
/// * `workspace` - The directory to check
///
/// # Returns
///
/// `true` if the workspace is inside a git repository, `false` otherwise.
fn is_git_repository(workspace: &Path) -> bool {
    Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(workspace)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get list of staged files (returns paths relative to workspace).
///
/// This function runs `git diff --cached --name-only --diff-filter=ACMR` to get
/// the list of files that are staged for commit. The filter excludes deleted files.
///
/// # Arguments
///
/// * `workspace` - The git repository root directory
///
/// # Returns
///
/// A vector of paths relative to the workspace, or an error if git command fails.
fn get_staged_files(workspace: &Path) -> Result<Vec<PathBuf>, Error> {
    let output = Command::new("git")
        .args(["diff", "--cached", "--name-only", "--diff-filter=ACMR"])
        .current_dir(workspace)
        .output()
        .map_err(|e| Error::Database(DatabaseError::IOError(e)))?;

    if !output.status.success() {
        return Err(Error::NotAGitRepository);
    }

    Ok(String::from_utf8_lossy(&output.stdout).lines().filter(|l| !l.is_empty()).map(PathBuf::from).collect())
}

/// Get set of all files with unstaged changes.
///
/// This function runs `git diff --name-only` once to get all files with unstaged
/// modifications, returning them as a HashSet for O(1) lookup.
///
/// # Arguments
///
/// * `workspace` - The git repository root directory
///
/// # Returns
///
/// A HashSet of paths (relative to workspace) that have unstaged changes.
fn get_files_with_unstaged_changes(workspace: &Path) -> Result<HashSet<PathBuf>, Error> {
    let output = Command::new("git")
        .args(["diff", "--name-only"])
        .current_dir(workspace)
        .output()
        .map_err(|e| Error::Database(DatabaseError::IOError(e)))?;

    Ok(String::from_utf8_lossy(&output.stdout).lines().filter(|l| !l.is_empty()).map(PathBuf::from).collect())
}
