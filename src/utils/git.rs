//! Git utilities for staged file operations.
//!
//! This module provides helper functions for interacting with git repositories,
//! specifically for the `--staged` formatting feature that allows formatting
//! staged files in pre-commit hooks.

use std::borrow::Cow;
use std::collections::HashSet;
use std::ffi::OsString;
use std::io::Write as _;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

use mago_database::Database;
use mago_database::DatabaseReader;
use mago_database::error::DatabaseError;
use mago_database::file::File;
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

/// Creates an ephemeral file with the contents of a staged file
///
/// # Arguments
///
/// * `workspace` - The git repository root directory
/// * `path` - The path for which to get the file
///
/// # Returns
///
/// An ephemeral file with the contents of the staged file
pub fn get_staged_file(workspace: &Path, path: &Path) -> Result<File, Error> {
    let mut index_path = OsString::from(":");
    index_path.push(path);

    let output = Command::new("git")
        .args(["cat-file", "-p"])
        .arg(index_path)
        .current_dir(workspace)
        .output()
        .map_err(|e| Error::Database(DatabaseError::IOError(e)))?;

    Ok(File::ephemeral(Cow::Borrowed("<stdin>"), Cow::Owned(String::from_utf8_lossy(&output.stdout).into_owned())))
}

/// Updates the contents of the staged file
///
/// # Arguments
///
/// * `workspace` - The git repository root directory
/// * `path` - The path for which to get the file
/// * `new_content` - The new content for the staged file
pub fn update_staged_file(workspace: &Path, path: &Path, new_content: String) -> Result<(), Error> {
    let blob_id = create_blob(workspace, path, new_content)?;
    let mode = get_mode(workspace, path)?;

    let mut cacheinfo = OsString::new();
    cacheinfo.push(&mode);
    cacheinfo.push(",");
    cacheinfo.push(&blob_id);
    cacheinfo.push(",");
    cacheinfo.push(path.as_os_str());

    Command::new("git")
        .args(["update-index", "--cacheinfo"])
        .arg(cacheinfo)
        .current_dir(workspace)
        .status()
        .map_err(|e| Error::Database(DatabaseError::IOError(e)))?;

    Ok(())
}

/// Verify that none of the given staged files have unstaged changes.
///
/// This check prevents data loss when applying fixes to staged files:
/// if a file has both staged and unstaged changes, modifying it on disk
/// and re-staging would include the previously-unstaged changes in the commit.
///
/// # Arguments
///
/// * `workspace` - The git repository root directory
/// * `staged_files` - The list of staged file paths to check
///
/// # Returns
///
/// `Ok(())` if all staged files are clean, or `Err(Error::StagedFileHasUnstagedChanges)`
/// if any staged file also has unstaged modifications.
pub fn ensure_staged_files_are_clean(workspace: &Path, staged_files: &[PathBuf]) -> Result<(), Error> {
    let files_with_unstaged = get_files_with_unstaged_changes(workspace)?;

    for staged_file in staged_files {
        if files_with_unstaged.contains(staged_file) {
            return Err(Error::StagedFileHasUnstagedChanges(staged_file.display().to_string()));
        }
    }

    Ok(())
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

/// Creates a new blob in the git object store for the given contents
///
/// # Arguments
///
/// * `workspace` - The git repository root directory
/// * `path` - The path of the file for which the contents are meant, used to apply git filters
/// * `content` - The contents for the blob object
///
/// # Returns
///
/// A string containing the id of the newly created blob
fn create_blob(workspace: &Path, path: &Path, content: String) -> Result<String, Error> {
    let mut child = Command::new("git")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .args(["hash-object", "-w", "--stdin", "--path"])
        .arg(path)
        .current_dir(workspace)
        .spawn()
        .map_err(|e| Error::Database(DatabaseError::IOError(e)))?;

    let mut stdin = child.stdin.take().expect("failed to get stdin");
    std::thread::spawn(move || {
        stdin.write_all(content.as_bytes()).expect("failed to write to stdin");
    });

    let output = child.wait_with_output().map_err(|e| Error::Database(DatabaseError::IOError(e)))?;

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

/// Gets the object mode for the given path in the git index
///
/// # Arguments
///
/// * `workspace` - The git repository root directory
/// * `path` - The path for which the object mode is requested
///
/// # Returns
///
/// A string containing the object mode for the path
fn get_mode(workspace: &Path, path: &Path) -> Result<String, Error> {
    let output = Command::new("git")
        .args(["ls-files", "--format=%(objectmode)"])
        .arg(path)
        .current_dir(workspace)
        .output()
        .map_err(|e| Error::Database(DatabaseError::IOError(e)))?;

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}
