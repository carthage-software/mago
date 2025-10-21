//! Baseline functionality for filtering known issues.
//!
//! This module provides functionality to track and filter known issues using baseline files.
//! A baseline allows teams to adopt static analysis gradually by suppressing existing issues
//! while ensuring no new issues are introduced.
//!
//! Baseline files store a snapshot of all issues at a specific point in time, organized by
//! file path and line numbers. The baseline can then be used to filter these known issues
//! from future analysis runs.
//!
//! File paths in baselines are normalized to use forward slashes for cross-platform compatibility,
//! ensuring baselines created on Windows work on Unix systems and vice versa.

use std::borrow::Cow;
use std::collections::BTreeMap;

use ahash::HashSet;
use serde::Deserialize;
use serde::Serialize;

use mago_database::DatabaseReader;
use mago_database::ReadDatabase;

use crate::IssueCollection;

/// Represents a single issue in the baseline format.
///
/// This is a simplified representation of an issue for storage in the baseline file.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
pub struct BaselineSourceIssue {
    pub code: String,
    pub start_line: u32,
    pub end_line: u32,
}

/// Represents a collection of issues for a specific file path in the baseline.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct BaselineEntry {
    pub issues: Vec<BaselineSourceIssue>,
}

/// The main baseline structure containing all entries organized by file path.
///
/// File paths are stored in a normalized format (using forward slashes)
/// to ensure cross-platform compatibility.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Baseline {
    pub entries: BTreeMap<Cow<'static, str>, BaselineEntry>,
}

/// The result of comparing a baseline with current issues.
#[derive(Debug, Clone)]
pub struct BaselineComparisonResult {
    /// Whether the baseline is up to date with current issues
    pub is_up_to_date: bool,
    /// Number of new issues not in the baseline
    pub new_issues_count: usize,
    /// Number of issues in baseline that no longer exist
    pub removed_issues_count: usize,
    /// Number of files with changes (new, removed, or modified issues)
    pub files_with_changes_count: usize,
}

impl Baseline {
    /// Creates a new empty baseline.
    pub fn new() -> Self {
        Self::default()
    }

    /// Normalizes a file path to use forward slashes for cross-platform compatibility.
    ///
    /// This ensures that baselines created on Windows work on Linux and vice versa.
    fn normalize_path(path: &str) -> String {
        path.replace('\\', "/")
    }

    /// Generates a baseline from a collection of issues.
    ///
    /// The baseline will contain all issues organized by their file paths.
    /// File paths are normalized to ensure cross-platform compatibility.
    pub fn generate_from_issues(issues: &IssueCollection, read_database: &ReadDatabase) -> Self {
        let mut entries: BTreeMap<Cow<'static, str>, BaselineEntry> = BTreeMap::new();

        for issue in issues.iter() {
            let Some(primary_annotation) = issue.annotations.iter().find(|a| a.is_primary()) else {
                continue;
            };

            let file = match read_database.get(&primary_annotation.span.file_id) {
                Ok(file) => file,
                Err(_) => continue,
            };

            // Normalize the file path to use forward slashes
            let normalized_path = Self::normalize_path(&file.name);

            let entry = entries.entry(Cow::Owned(normalized_path)).or_default();

            let start_line = file.line_number(primary_annotation.span.start.offset);
            let end_line = file.line_number(primary_annotation.span.end.offset);

            let baseline_issue = BaselineSourceIssue {
                code: issue.code.as_ref().unwrap_or(&String::from("unknown")).clone(),
                start_line,
                end_line,
            };

            if !entry.issues.contains(&baseline_issue) {
                entry.issues.push(baseline_issue);
            }
        }

        // Sort issues within each entry for consistent output
        for entry in entries.values_mut() {
            entry.issues.sort();
        }

        Self { entries }
    }

    /// Filters an issue collection against this baseline.
    ///
    /// Returns a new issue collection containing only issues that are not in the baseline.
    pub fn filter_issues(&self, issues: IssueCollection, read_database: &ReadDatabase) -> IssueCollection {
        let mut filtered_issues = Vec::new();

        for issue in issues.into_iter() {
            let Some(primary_annotation) = issue.annotations.iter().find(|a| a.is_primary()) else {
                filtered_issues.push(issue);
                continue;
            };

            let file = match read_database.get(&primary_annotation.span.file_id) {
                Ok(file) => file,
                Err(_) => {
                    filtered_issues.push(issue);
                    continue;
                }
            };

            // Normalize the file path for lookup
            let normalized_path = Self::normalize_path(&file.name);

            let Some(baseline_entry) = self.entries.get(normalized_path.as_str()) else {
                filtered_issues.push(issue);
                continue;
            };

            let start_line = file.line_number(primary_annotation.span.start.offset);
            let end_line = file.line_number(primary_annotation.span.end.offset);

            let baseline_issue = BaselineSourceIssue {
                code: issue.code.as_ref().unwrap_or(&String::from("unknown")).clone(),
                start_line,
                end_line,
            };

            if !baseline_entry.issues.contains(&baseline_issue) {
                filtered_issues.push(issue);
            }
        }

        IssueCollection::from(filtered_issues)
    }

    /// Compares this baseline with a collection of current issues.
    ///
    /// Returns a comparison result with statistics about differences between the baseline
    /// and current issues.
    pub fn compare_with_issues(
        &self,
        issues: &IssueCollection,
        read_database: &ReadDatabase,
    ) -> BaselineComparisonResult {
        let current_baseline = Self::generate_from_issues(issues, read_database);

        // Quick check - if they're exactly the same, we're done
        if self.entries == current_baseline.entries {
            return BaselineComparisonResult {
                is_up_to_date: true,
                new_issues_count: 0,
                removed_issues_count: 0,
                files_with_changes_count: 0,
            };
        }

        // Analyze what's different
        let mut new_issues = 0;
        let mut removed_issues = 0;
        let mut files_with_changes = HashSet::default();

        // Check for new issues (in current but not in baseline)
        for (file_path, current_entry) in &current_baseline.entries {
            if let Some(baseline_entry) = self.entries.get(file_path) {
                let baseline_issues: HashSet<_> = baseline_entry.issues.iter().collect();
                let current_issues: HashSet<_> = current_entry.issues.iter().collect();

                let new_in_file = current_issues.difference(&baseline_issues).count();
                let removed_in_file = baseline_issues.difference(&current_issues).count();

                if new_in_file > 0 || removed_in_file > 0 {
                    files_with_changes.insert(file_path.as_ref());
                    new_issues += new_in_file;
                    removed_issues += removed_in_file;
                }
            } else {
                // Entire file is new
                new_issues += current_entry.issues.len();
                files_with_changes.insert(file_path.as_ref());
            }
        }

        // Check for files that were removed entirely
        for (file_path, baseline_entry) in &self.entries {
            if !current_baseline.entries.contains_key(file_path) {
                removed_issues += baseline_entry.issues.len();
                files_with_changes.insert(file_path.as_ref());
            }
        }

        BaselineComparisonResult {
            is_up_to_date: false,
            new_issues_count: new_issues,
            removed_issues_count: removed_issues,
            files_with_changes_count: files_with_changes.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Annotation;
    use crate::Issue;
    use mago_database::Database;
    use mago_database::file::File;
    use mago_database::file::FileId;
    use mago_span::Position;
    use mago_span::Span;

    fn create_test_database() -> (Database, FileId) {
        let file =
            File::ephemeral(Cow::Borrowed("test.php"), Cow::Borrowed("<?php\n// Line 1\n// Line 2\n// Line 3\n"));
        let file_id = file.id;
        let db = Database::single(file);
        (db, file_id)
    }

    fn create_test_issue(file_id: FileId, code: &str, start_offset: u32, end_offset: u32) -> Issue {
        Issue::error("test error").with_code(code).with_annotation(Annotation::primary(Span::new(
            file_id,
            Position::new(start_offset),
            Position::new(end_offset),
        )))
    }

    #[test]
    fn test_normalize_path() {
        assert_eq!(Baseline::normalize_path("foo/bar/baz.php"), "foo/bar/baz.php");
        assert_eq!(Baseline::normalize_path("foo\\bar\\baz.php"), "foo/bar/baz.php");
        assert_eq!(Baseline::normalize_path("C:\\Users\\test\\file.php"), "C:/Users/test/file.php");
    }

    #[test]
    fn test_generate_baseline_from_issues() {
        let (db, file_id) = create_test_database();
        let read_db = db.read_only();

        let mut issues = IssueCollection::new();
        issues.push(create_test_issue(file_id, "E001", 0, 5));
        issues.push(create_test_issue(file_id, "E002", 10, 15));

        let baseline = Baseline::generate_from_issues(&issues, &read_db);

        assert_eq!(baseline.entries.len(), 1);
        let entry = baseline.entries.get("test.php").unwrap();
        assert_eq!(entry.issues.len(), 2);
    }

    #[test]
    fn test_filter_issues() {
        let (db, file_id) = create_test_database();
        let read_db = db.read_only();

        let mut baseline = Baseline::new();
        let mut entry = BaselineEntry::default();
        entry.issues.push(BaselineSourceIssue { code: "E001".to_string(), start_line: 0, end_line: 0 });
        baseline.entries.insert(Cow::Borrowed("test.php"), entry);

        let mut issues = IssueCollection::new();
        issues.push(create_test_issue(file_id, "E001", 0, 5));
        issues.push(create_test_issue(file_id, "E002", 10, 15));

        let filtered = baseline.filter_issues(issues, &read_db);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered.iter().next().unwrap().code.as_ref().unwrap(), "E002");
    }

    #[test]
    fn test_compare_baseline_with_issues() {
        let (db, file_id) = create_test_database();
        let read_db = db.read_only();

        let mut baseline = Baseline::new();
        let mut entry = BaselineEntry::default();
        entry.issues.push(BaselineSourceIssue { code: "E001".to_string(), start_line: 0, end_line: 0 });
        entry.issues.push(BaselineSourceIssue { code: "E003".to_string(), start_line: 2, end_line: 2 });
        baseline.entries.insert(Cow::Borrowed("test.php"), entry);

        let mut issues = IssueCollection::new();
        issues.push(create_test_issue(file_id, "E001", 0, 5));
        issues.push(create_test_issue(file_id, "E002", 10, 15));

        let result = baseline.compare_with_issues(&issues, &read_db);

        assert!(!result.is_up_to_date);
        assert_eq!(result.removed_issues_count, 1);

        assert_eq!(result.new_issues_count, 1);
        assert_eq!(result.files_with_changes_count, 1);
    }
}
