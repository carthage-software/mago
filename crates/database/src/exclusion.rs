//! File and directory exclusion patterns for database loading.
//!
//! This module provides the [`Exclusion`] enum for defining rules that determine
//! which files and directories should be excluded when scanning a project with
//! [`DatabaseLoader`](crate::loader::DatabaseLoader).
//!
//! Exclusions support two modes:
//!
//! - **Exact Paths**: Exclude specific files or directories by their filesystem path
//! - **Glob Patterns**: Exclude multiple paths matching a pattern (e.g., `*.tmp`, `**/test/**`)
//!
//! # Use Cases
//!
//! - Exclude build artifacts and cache directories (`target/`, `.cache/`)
//! - Exclude version control metadata (`.git/`, `.svn/`)
//! - Exclude test files or fixtures (`tests/fixtures/`)
//! - Exclude temporary files (`*.swp`, `*.tmp`)

use std::borrow::Cow;
use std::path::Path;

/// A rule for excluding files or directories from filesystem scans.
///
/// This enum represents two types of exclusion rules that can be used when loading
/// a database to filter out unwanted files. Exclusions are evaluated during the
/// scan performed by [`DatabaseLoader`](crate::loader::DatabaseLoader).
///
/// # Lifetime
///
/// The `'a` lifetime parameter allows the exclusion to borrow paths and patterns
/// from configuration or other long-lived data structures, avoiding unnecessary
/// allocations during database construction.
///
/// # Variants
///
/// ## Path Exclusion
///
/// Excludes an exact filesystem path, which can be either a file or directory.
/// When a directory is excluded, all files and subdirectories within it are also
/// excluded.
///
/// Paths can be absolute or relative to the workspace. Relative paths are
/// canonicalized during loader construction.
///
/// ## Pattern Exclusion
///
/// Excludes paths matching a glob pattern. Glob patterns support wildcards and
/// are matched against the full path of each discovered file.
///
/// Common pattern syntax:
/// - `*` matches any characters except path separators
/// - `**` matches any characters including path separators (recursive)
/// - `?` matches a single character
/// - `[abc]` matches one character from the set
///
/// # Ordering
///
/// Exclusions are ordered first by variant (Path before Pattern), then by the
/// value within the variant. This enables efficient deduplication and set operations.
#[derive(Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum Exclusion<'a> {
    /// Exclude a specific file or directory by its exact path.
    ///
    /// The path can be absolute or relative to the workspace directory.
    /// If a directory is specified, all contents within it are recursively excluded.
    ///
    /// Uses `Cow` to allow borrowing paths from configuration while supporting
    /// owned paths when canonicalization is required.
    Path(Cow<'a, Path>),

    /// Exclude files and directories matching a glob pattern.
    ///
    /// The pattern is evaluated against the full path of each file discovered
    /// during the scan. Supports standard glob syntax including wildcards,
    /// character classes, and recursive matchers.
    ///
    /// Uses `Cow` to avoid allocating when borrowing patterns from configuration.
    Pattern(Cow<'a, str>),
}
