//! Command-line arguments for file-content substitution.
//!
//! This module defines [`SubstitutionArgs`], a reusable set of arguments for swapping one or
//! more host files with replacement files while keeping the rest of the project intact. The
//! primary consumer is mutation-testing tooling that wants the analyzer and linter to evaluate
//! a mutated copy of a source file without changing the original on disk.
//!
//! # Syntax
//!
//! Each `--substitute ORIG=TEMP` pair maps an existing host file inside the workspace (`ORIG`)
//! to a file providing replacement content (`TEMP`). Both paths must be absolute. The flag can
//! be given multiple times to substitute several files in a single invocation.
//!
//! # Mechanism
//!
//! Substitution is implemented at the CLI layer as path manipulation: each `TEMP` is appended
//! to the orchestrator's `paths` (so it becomes a host file) and each `ORIG` is appended to
//! `excludes` (so it is skipped during scanning). No loader-level changes are required; the
//! full project is still analyzed, just with the replacement file loaded in place of the
//! original.
//!
//! # Validation
//!
//! Argument parsing only checks surface shape (presence of `=`, non-empty segments). Existence
//! and absolute-path checks happen in [`SubstitutionArgs::resolve`] before the orchestrator is
//! handed the mapping.

use std::path::PathBuf;

use clap::Parser;

use crate::error::Error;

/// A parsed `--substitute ORIG=TEMP` pair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Substitution {
    /// Absolute path to the original host file being replaced.
    pub original: PathBuf,
    /// Absolute path to the file providing replacement content.
    pub temporary: PathBuf,
}

/// Command-line arguments for file-content substitution.
///
/// Designed to be flattened into command structs via `#[clap(flatten)]`.
#[derive(Parser, Debug, Clone, Default)]
pub struct SubstitutionArgs {
    /// Replace a host file with another file for this invocation (`ORIG=TEMP`).
    ///
    /// Both paths must be absolute. `ORIG` must be a host file inside the project; `TEMP` is
    /// any readable file providing the replacement content. Repeat the flag to substitute
    /// multiple files in a single run.
    ///
    /// Primarily intended for mutation-testing frameworks that generate a mutated copy of a
    /// source file and want the analyzer or linter to evaluate it against the rest of the
    /// project without writing the mutation to its original location.
    #[arg(
        long = "substitute",
        value_name = "ORIG=TEMP",
        conflicts_with_all = ["stdin_input"]
    )]
    pub substitutions: Vec<String>,
}

impl SubstitutionArgs {
    /// Parses and validates every `--substitute` value.
    ///
    /// Validates: `ORIG=TEMP` shape, absolute paths, existence of both files. Returns the
    /// typed pairs in the order they were given on the command line.
    pub fn resolve(&self) -> Result<Vec<Substitution>, Error> {
        self.substitutions.iter().map(|raw| parse_substitute_pair(raw)).collect()
    }
}

fn parse_substitute_pair(raw: &str) -> Result<Substitution, Error> {
    let (original, temporary) = raw.split_once('=').ok_or_else(|| {
        Error::InvalidArgument(format!("--substitute value `{raw}` is missing `=`; expected format is ORIG=TEMP"))
    })?;

    if original.is_empty() {
        return Err(Error::InvalidArgument(format!("--substitute value `{raw}` is missing the ORIG path before `=`")));
    }

    if temporary.is_empty() {
        return Err(Error::InvalidArgument(format!("--substitute value `{raw}` is missing the TEMP path after `=`")));
    }

    let original = PathBuf::from(original);
    let temporary = PathBuf::from(temporary);

    if !original.is_absolute() {
        return Err(Error::InvalidArgument(format!(
            "--substitute: original path `{}` must be absolute",
            original.display()
        )));
    }

    if !temporary.is_absolute() {
        return Err(Error::InvalidArgument(format!(
            "--substitute: temporary path `{}` must be absolute",
            temporary.display()
        )));
    }

    if !original.exists() {
        return Err(Error::InvalidArgument(format!(
            "--substitute: original path `{}` does not exist",
            original.display()
        )));
    }

    if !temporary.exists() {
        return Err(Error::InvalidArgument(format!(
            "--substitute: temporary path `{}` does not exist",
            temporary.display()
        )));
    }

    Ok(Substitution { original, temporary })
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;

    fn make_tempfile(contents: &str) -> PathBuf {
        let dir = std::env::temp_dir();
        let path = dir.join(format!("mago-substitute-test-{}.php", uuid_like()));
        fs::write(&path, contents).expect("write tempfile");
        path
    }

    fn uuid_like() -> u128 {
        use std::time::SystemTime;

        SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0)
    }

    #[test]
    fn parses_valid_pair() {
        let orig = make_tempfile("<?php // orig");
        let temp = make_tempfile("<?php // temp");

        let raw = format!("{}={}", orig.display(), temp.display());
        let pair = parse_substitute_pair(&raw).expect("parses");

        assert_eq!(pair.original, orig);
        assert_eq!(pair.temporary, temp);

        let _ = fs::remove_file(&orig);
        let _ = fs::remove_file(&temp);
    }

    #[test]
    fn rejects_missing_equals_sign() {
        let error = parse_substitute_pair("/abs/orig.php").unwrap_err();
        assert!(matches!(error, Error::InvalidArgument(_)));
    }

    #[test]
    fn rejects_empty_original() {
        let error = parse_substitute_pair("=/abs/temp.php").unwrap_err();
        assert!(matches!(error, Error::InvalidArgument(_)));
    }

    #[test]
    fn rejects_empty_temporary() {
        let error = parse_substitute_pair("/abs/orig.php=").unwrap_err();
        assert!(matches!(error, Error::InvalidArgument(_)));
    }

    #[test]
    fn rejects_relative_original() {
        let error = parse_substitute_pair("orig.php=/abs/temp.php").unwrap_err();
        assert!(matches!(error, Error::InvalidArgument(_)));
    }

    #[test]
    fn rejects_missing_original_file() {
        let error = parse_substitute_pair("/definitely/does/not/exist.php=/also/not/real.php").unwrap_err();
        assert!(matches!(error, Error::InvalidArgument(_)));
    }
}
