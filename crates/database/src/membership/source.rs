use std::path::Path;
use std::path::PathBuf;

use glob::MatchOptions;
use glob::Pattern;

use crate::error::DatabaseError;
use crate::loader::calculate_pattern_specificity;
use crate::utils::bytes_to_path;
use crate::utils::bytes_to_string_lossy;

#[derive(Debug, Clone)]
pub(super) struct SourcePattern {
    pub root: PathBuf,
    pub specificity: usize,
    pattern: Option<Pattern>,
}

impl SourcePattern {
    pub fn new(workspace: &Path, value: &[u8]) -> Result<Self, DatabaseError> {
        let path = bytes_to_path(value);
        let absolute = if path.is_absolute() { path.into_owned() } else { workspace.join(path.as_ref()) };
        let specificity = calculate_pattern_specificity(value);
        let first_glob = value.iter().position(|byte| matches!(byte, b'*' | b'?' | b'[' | b'{'));
        if absolute.exists() || first_glob.is_none() {
            return Ok(Self { root: canonical_path(&absolute), specificity, pattern: None });
        }

        let end = value[..first_glob.unwrap_or(0)]
            .iter()
            .rposition(|byte| std::path::is_separator(char::from(*byte)))
            .map_or(0, |position| position + 1);

        let base = if end == 0 { PathBuf::from(".") } else { bytes_to_path(&value[..end]).into_owned() };
        let root = canonical_path(&if base.is_absolute() { base } else { workspace.join(base) });
        let pattern = Pattern::new(&bytes_to_string_lossy(&value[end..]))
            .map_err(|error| DatabaseError::Glob(error.to_string()))?;

        Ok(Self { root, specificity, pattern: Some(pattern) })
    }

    /// Whether the loader walks this source as a directory, rather than matching a glob or a single file.
    pub fn walks(&self) -> bool {
        self.pattern.is_none()
    }

    pub fn matches(&self, path: &Path, extension_ok: bool) -> bool {
        let Ok(relative) = path.strip_prefix(&self.root) else { return false };
        match &self.pattern {
            None => extension_ok || relative.as_os_str().is_empty(),
            Some(pattern) => {
                extension_ok
                    && pattern.matches_path_with(
                        relative,
                        MatchOptions { require_literal_separator: true, ..MatchOptions::new() },
                    )
            }
        }
    }
}

pub(super) fn canonical_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| {
        path.ancestors()
            .skip(1)
            .find_map(|ancestor| Some(ancestor.canonicalize().ok()?.join(path.strip_prefix(ancestor).ok()?)))
            .unwrap_or_else(|| path.to_path_buf())
    })
}
