//! Baseline functionality for filtering known issues.
//!
//! This module provides functionality to track and filter known issues using baseline files.
//! A baseline allows teams to adopt static analysis gradually by suppressing existing issues
//! while ensuring no new issues are introduced.
//!
//! Two baseline variants are supported:
//!
//! - **Strict**: Stores exact line numbers for each issue. Changes to line numbers require
//!   baseline regeneration. This is the most precise variant.
//!
//! - **Loose**: Stores issue counts per (file, code, message) tuple. More resilient to code
//!   changes as line number shifts don't affect the baseline. This is the default.
//!
//! File paths in baselines are normalized to use forward slashes for cross-platform compatibility,
//! ensuring baselines created on Windows work on Unix systems and vice versa.

use std::borrow::Cow;
use std::collections::BTreeMap;

use foldhash::HashMap;
use foldhash::HashSet;
use schemars::JsonSchema;

use mago_database::DatabaseReader;
use mago_database::ReadDatabase;

use crate::Annotation;
use crate::Issue;
use crate::IssueCollection;

/// The annotation a baseline uses to identify an issue's location.
///
/// Prefers a primary annotation, falls back to the first annotation. This
/// keeps the baseline resilient to rules that forget to mark any
/// annotation as primary — those issues still get baselined/filtered
/// instead of leaking through as unbaselined on every re-run.
#[inline]
fn baseline_annotation(issue: &Issue) -> Option<&Annotation> {
    issue.annotations.iter().find(|a| a.is_primary()).or_else(|| issue.annotations.first())
}

/// The variant of baseline format to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum BaselineVariant {
    /// Strict baseline with exact line matching.
    ///
    /// Each issue is stored with its exact start and end line numbers.
    /// Any change in line numbers requires baseline regeneration.
    Strict,
    /// Loose baseline with count-based matching.
    ///
    /// Issues are grouped by (file, code, message) and stored with a count.
    /// More resilient to code changes as line shifts don't affect the baseline.
    #[default]
    Loose,
}

/// Represents a single issue in the strict baseline format.
///
/// This is a simplified representation of an issue for storage in the baseline file.
#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StrictBaselineIssue {
    pub code: String,
    pub start_line: u32,
    pub end_line: u32,
}

/// Represents a collection of issues for a specific file path in the strict baseline.
#[derive(Debug, Clone, Default, PartialEq, Eq, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StrictBaselineEntry {
    pub issues: Vec<StrictBaselineIssue>,
}

/// The strict baseline structure containing all entries organized by file path.
///
/// File paths are stored in a normalized format (using forward slashes)
/// to ensure cross-platform compatibility.
#[derive(Debug, Default, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StrictBaseline {
    /// The baseline variant marker. When present, indicates this is a strict baseline.
    /// When absent (for backward compatibility), the baseline is assumed to be strict.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub variant: Option<BaselineVariant>,
    /// The entries organized by file path.
    pub entries: BTreeMap<Cow<'static, str>, StrictBaselineEntry>,
}

/// Represents a single issue entry in the loose baseline format.
///
/// Issues are grouped by (file, code, message) tuple with a count.
#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LooseBaselineIssue {
    /// The normalized file path where the issues occur.
    pub file: String,
    /// The issue code (e.g., "missing-type-hint").
    pub code: String,
    /// The issue message.
    pub message: String,
    /// The number of occurrences of this issue.
    pub count: u32,
}

/// The loose baseline structure with count-based issue tracking.
#[derive(Debug, Default, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LooseBaseline {
    /// The baseline variant marker.
    pub variant: BaselineVariant,
    /// The list of issues with their counts.
    pub issues: Vec<LooseBaselineIssue>,
}

/// A baseline that can be either strict or loose.
#[derive(Debug, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum Baseline {
    /// Strict baseline with exact line matching.
    Strict(StrictBaseline),
    /// Loose baseline with count-based matching.
    Loose(LooseBaseline),
}

/// A specific issue that appeared or disappeared during a baseline comparison.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct BaselineChangeEntry {
    pub file: String,
    pub code: String,
    pub start_line: u32,
    pub end_line: u32,
}

/// The result of comparing a baseline with current issues.
#[derive(Debug, Clone)]
pub struct BaselineComparisonResult {
    /// Whether the baseline is up to date with current issues
    pub is_up_to_date: bool,
    /// Issues present in the current run but absent from the baseline
    pub new_issues: Vec<BaselineChangeEntry>,
    /// Issues recorded in the baseline that no longer appear
    pub removed_issues: Vec<BaselineChangeEntry>,
    /// Number of files with changes (new, removed, or modified issues)
    pub files_with_changes_count: usize,
}

/// Normalizes a file path to use forward slashes for cross-platform compatibility.
///
/// This ensures that baselines created on Windows work on Linux and vice versa.
fn normalize_path(path: &[u8]) -> String {
    String::from_utf8_lossy(path).replace('\\', "/")
}

impl StrictBaseline {
    /// Creates a new empty strict baseline.
    #[must_use]
    pub fn new() -> Self {
        Self { variant: Some(BaselineVariant::Strict), entries: BTreeMap::new() }
    }

    /// Generates a strict baseline from a collection of issues.
    ///
    /// The baseline will contain all issues organized by their file paths with exact line numbers.
    /// File paths are normalized to ensure cross-platform compatibility.
    #[must_use]
    pub fn generate_from_issues(issues: &IssueCollection, read_database: &ReadDatabase) -> Self {
        let mut entries: BTreeMap<Cow<'static, str>, StrictBaselineEntry> = BTreeMap::new();

        for issue in issues.iter() {
            let Some(annotation) = baseline_annotation(issue) else {
                continue;
            };

            let Ok(file) = read_database.get(&annotation.span.file_id) else {
                continue;
            };

            let normalized_path = normalize_path(&file.name);
            let entry = entries.entry(Cow::Owned(normalized_path)).or_default();

            let start_line = file.line_number(annotation.span.start.offset);
            let end_line = file.line_number(annotation.span.end.offset);

            let baseline_issue = StrictBaselineIssue {
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

        Self { variant: Some(BaselineVariant::Strict), entries }
    }

    /// Filters an issue collection against this strict baseline.
    ///
    /// Returns a new issue collection containing only issues that are not in the baseline.
    /// Issues are matched by exact file path, code, and line numbers.
    #[must_use]
    pub fn filter_issues(&self, issues: IssueCollection, read_database: &ReadDatabase) -> IssueCollection {
        let mut filtered_issues = Vec::new();

        for issue in issues {
            let Some(annotation) = baseline_annotation(&issue) else {
                filtered_issues.push(issue);
                continue;
            };

            let Ok(file) = read_database.get(&annotation.span.file_id) else {
                filtered_issues.push(issue);
                continue;
            };

            let normalized_path = normalize_path(&file.name);
            let Some(baseline_entry) = self.entries.get(normalized_path.as_str()) else {
                filtered_issues.push(issue);
                continue;
            };

            let start_line = file.line_number(annotation.span.start.offset);
            let end_line = file.line_number(annotation.span.end.offset);

            let baseline_issue = StrictBaselineIssue {
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

    /// Compares this strict baseline with a collection of current issues.
    ///
    /// Returns a comparison result with statistics about differences between the baseline
    /// and current issues.
    #[must_use]
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
                new_issues: vec![],
                removed_issues: vec![],
                files_with_changes_count: 0,
            };
        }

        // Analyze what's different
        let mut new_issues: Vec<BaselineChangeEntry> = vec![];
        let mut removed_issues: Vec<BaselineChangeEntry> = vec![];
        let mut files_with_changes = HashSet::default();

        // Check for new issues (in current but not in baseline)
        for (file_path, current_entry) in &current_baseline.entries {
            if let Some(baseline_entry) = self.entries.get(file_path) {
                let baseline_issues: HashSet<_> = baseline_entry.issues.iter().collect();
                let current_issues: HashSet<_> = current_entry.issues.iter().collect();

                for issue in current_issues.difference(&baseline_issues) {
                    new_issues.push(BaselineChangeEntry {
                        file: file_path.to_string(),
                        code: issue.code.clone(),
                        start_line: issue.start_line,
                        end_line: issue.end_line,
                    });
                    files_with_changes.insert(file_path.as_ref());
                }
                for issue in baseline_issues.difference(&current_issues) {
                    removed_issues.push(BaselineChangeEntry {
                        file: file_path.to_string(),
                        code: issue.code.clone(),
                        start_line: issue.start_line,
                        end_line: issue.end_line,
                    });
                    files_with_changes.insert(file_path.as_ref());
                }
            } else {
                // Entire file is new
                for issue in &current_entry.issues {
                    new_issues.push(BaselineChangeEntry {
                        file: file_path.to_string(),
                        code: issue.code.clone(),
                        start_line: issue.start_line,
                        end_line: issue.end_line,
                    });
                }
                files_with_changes.insert(file_path.as_ref());
            }
        }

        // Check for files that were removed entirely
        for (file_path, baseline_entry) in &self.entries {
            if !current_baseline.entries.contains_key(file_path) {
                for issue in &baseline_entry.issues {
                    removed_issues.push(BaselineChangeEntry {
                        file: file_path.to_string(),
                        code: issue.code.clone(),
                        start_line: issue.start_line,
                        end_line: issue.end_line,
                    });
                }
                files_with_changes.insert(file_path.as_ref());
            }
        }

        BaselineComparisonResult {
            is_up_to_date: false,
            new_issues,
            removed_issues,
            files_with_changes_count: files_with_changes.len(),
        }
    }

    /// Removes baseline entries that no longer correspond to a current issue.
    ///
    /// Unlike [`generate_from_issues`](Self::generate_from_issues), this never adds new
    /// entries. The returned baseline keeps only entries that still match an issue in
    /// `issues`; the second tuple element is the number of stale entries removed.
    #[must_use]
    pub fn prune_outdated_entries(&self, issues: &IssueCollection, read_database: &ReadDatabase) -> (Self, usize) {
        let current = Self::generate_from_issues(issues, read_database);

        let mut entries: BTreeMap<Cow<'static, str>, StrictBaselineEntry> = BTreeMap::new();
        let mut removed_count = 0;

        for (file_path, entry) in &self.entries {
            let current_issues: HashSet<&StrictBaselineIssue> = match current.entries.get(file_path) {
                Some(current_entry) => current_entry.issues.iter().collect(),
                None => HashSet::default(),
            };

            let mut kept = Vec::new();
            for issue in &entry.issues {
                if current_issues.contains(issue) {
                    kept.push(issue.clone());
                } else {
                    removed_count += 1;
                }
            }

            if !kept.is_empty() {
                entries.insert(file_path.clone(), StrictBaselineEntry { issues: kept });
            }
        }

        (Self { variant: Some(BaselineVariant::Strict), entries }, removed_count)
    }
}

impl LooseBaseline {
    /// Creates a new empty loose baseline.
    #[must_use]
    pub fn new() -> Self {
        Self { variant: BaselineVariant::Loose, issues: Vec::new() }
    }

    /// Generates a loose baseline from a collection of issues.
    ///
    /// Issues are grouped by (file, code, message) tuple and stored with a count.
    /// File paths are normalized to ensure cross-platform compatibility.
    #[must_use]
    pub fn generate_from_issues(issues: &IssueCollection, read_database: &ReadDatabase) -> Self {
        let mut issue_counts: HashMap<(String, String, String), u32> = HashMap::default();

        for issue in issues.iter() {
            let Some(annotation) = baseline_annotation(issue) else {
                continue;
            };

            let Ok(file) = read_database.get(&annotation.span.file_id) else {
                continue;
            };

            let normalized_path = normalize_path(&file.name);
            let code = issue.code.as_ref().unwrap_or(&String::from("unknown")).clone();
            let message = issue.message.clone();

            let key = (normalized_path, code, message);
            *issue_counts.entry(key).or_insert(0) += 1;
        }

        let mut baseline_issues: Vec<LooseBaselineIssue> = issue_counts
            .into_iter()
            .map(|((file, code, message), count)| LooseBaselineIssue { file, code, message, count })
            .collect();

        baseline_issues.sort();

        Self { variant: BaselineVariant::Loose, issues: baseline_issues }
    }

    /// Filters an issue collection against this loose baseline.
    ///
    /// Returns a new issue collection containing only issues that exceed the baseline counts.
    /// For each (file, code, message) tuple, issues are filtered out up to the count in the baseline.
    #[must_use]
    pub fn filter_issues(&self, issues: IssueCollection, read_database: &ReadDatabase) -> IssueCollection {
        let mut remaining_counts: HashMap<(String, String, String), u32> =
            self.issues.iter().map(|i| ((i.file.clone(), i.code.clone(), i.message.clone()), i.count)).collect();

        let mut filtered_issues = Vec::new();

        for issue in issues {
            let Some(annotation) = baseline_annotation(&issue) else {
                filtered_issues.push(issue);
                continue;
            };

            let Ok(file) = read_database.get(&annotation.span.file_id) else {
                filtered_issues.push(issue);
                continue;
            };

            let normalized_path = normalize_path(&file.name);
            let code = issue.code.as_ref().unwrap_or(&String::from("unknown")).clone();
            let key = (normalized_path, code, issue.message.clone());

            if let Some(count) = remaining_counts.get_mut(&key)
                && *count > 0
            {
                *count -= 1;
                continue;
            }

            filtered_issues.push(issue);
        }

        IssueCollection::from(filtered_issues)
    }

    /// Compares this loose baseline with a collection of current issues.
    ///
    /// Returns a comparison result with statistics about differences between the baseline
    /// and current issues.
    #[must_use]
    pub fn compare_with_issues(
        &self,
        issues: &IssueCollection,
        read_database: &ReadDatabase,
    ) -> BaselineComparisonResult {
        let current = Self::generate_from_issues(issues, read_database);

        let current_map: HashMap<_, _> =
            current.issues.iter().map(|i| ((i.file.clone(), i.code.clone(), i.message.clone()), i.count)).collect();

        let baseline_map: HashMap<_, _> =
            self.issues.iter().map(|i| ((i.file.clone(), i.code.clone(), i.message.clone()), i.count)).collect();

        let mut new_issues: Vec<BaselineChangeEntry> = vec![];
        let mut removed_issues: Vec<BaselineChangeEntry> = vec![];
        let mut files_with_changes: HashSet<&str> = HashSet::default();

        for (key, &current_count) in &current_map {
            let baseline_count = baseline_map.get(key).copied().unwrap_or(0);
            if current_count > baseline_count {
                let delta = (current_count - baseline_count) as usize;
                for _ in 0..delta {
                    new_issues.push(BaselineChangeEntry {
                        file: key.0.clone(),
                        code: key.1.clone(),
                        start_line: 0,
                        end_line: 0,
                    });
                }
                files_with_changes.insert(key.0.as_str());
            }
        }

        for (key, &baseline_count) in &baseline_map {
            let current_count = current_map.get(key).copied().unwrap_or(0);
            if baseline_count > current_count {
                let delta = (baseline_count - current_count) as usize;
                for _ in 0..delta {
                    removed_issues.push(BaselineChangeEntry {
                        file: key.0.clone(),
                        code: key.1.clone(),
                        start_line: 0,
                        end_line: 0,
                    });
                }
                files_with_changes.insert(key.0.as_str());
            }
        }

        BaselineComparisonResult {
            is_up_to_date: new_issues.is_empty() && removed_issues.is_empty(),
            new_issues,
            removed_issues,
            files_with_changes_count: files_with_changes.len(),
        }
    }

    /// Removes baseline issues that no longer correspond to a current issue.
    ///
    /// Unlike [`generate_from_issues`](Self::generate_from_issues), this never adds new
    /// issues and never raises a count. For each `(file, code, message)` tuple the count
    /// is capped at the current number of occurrences; tuples with no current occurrence
    /// are dropped. The second tuple element is the number of stale occurrences removed.
    #[must_use]
    pub fn prune_outdated_entries(&self, issues: &IssueCollection, read_database: &ReadDatabase) -> (Self, usize) {
        let current = Self::generate_from_issues(issues, read_database);
        let current_counts: HashMap<(String, String, String), u32> = current
            .issues
            .iter()
            .map(|issue| ((issue.file.clone(), issue.code.clone(), issue.message.clone()), issue.count))
            .collect();

        let mut kept: Vec<LooseBaselineIssue> = Vec::new();
        let mut removed_count = 0;

        for issue in &self.issues {
            let key = (issue.file.clone(), issue.code.clone(), issue.message.clone());
            let current_count = current_counts.get(&key).copied().unwrap_or(0);
            let new_count = issue.count.min(current_count);

            removed_count += (issue.count - new_count) as usize;

            if new_count > 0 {
                kept.push(LooseBaselineIssue {
                    file: issue.file.clone(),
                    code: issue.code.clone(),
                    message: issue.message.clone(),
                    count: new_count,
                });
            }
        }

        kept.sort();

        (Self { variant: BaselineVariant::Loose, issues: kept }, removed_count)
    }
}

impl Baseline {
    /// Generates a baseline from a collection of issues using the specified variant.
    #[must_use]
    pub fn generate_from_issues(
        issues: &IssueCollection,
        read_database: &ReadDatabase,
        variant: BaselineVariant,
    ) -> Self {
        match variant {
            BaselineVariant::Strict => Baseline::Strict(StrictBaseline::generate_from_issues(issues, read_database)),
            BaselineVariant::Loose => Baseline::Loose(LooseBaseline::generate_from_issues(issues, read_database)),
        }
    }

    /// Filters an issue collection against this baseline.
    ///
    /// Returns a new issue collection containing only issues that are not in the baseline.
    #[must_use]
    pub fn filter_issues(&self, issues: IssueCollection, read_database: &ReadDatabase) -> IssueCollection {
        match self {
            Baseline::Strict(strict) => strict.filter_issues(issues, read_database),
            Baseline::Loose(loose) => loose.filter_issues(issues, read_database),
        }
    }

    /// Compares this baseline with a collection of current issues.
    ///
    /// Returns a comparison result with statistics about differences between the baseline
    /// and current issues.
    #[must_use]
    pub fn compare_with_issues(
        &self,
        issues: &IssueCollection,
        read_database: &ReadDatabase,
    ) -> BaselineComparisonResult {
        match self {
            Baseline::Strict(strict) => strict.compare_with_issues(issues, read_database),
            Baseline::Loose(loose) => loose.compare_with_issues(issues, read_database),
        }
    }

    /// Removes baseline entries that no longer correspond to a current issue.
    ///
    /// This never adds new entries, so issues introduced since the baseline was created
    /// remain reportable. The second tuple element is the number of stale entries removed.
    #[must_use]
    pub fn prune_outdated_entries(&self, issues: &IssueCollection, read_database: &ReadDatabase) -> (Self, usize) {
        match self {
            Baseline::Strict(strict) => {
                let (pruned, removed) = strict.prune_outdated_entries(issues, read_database);
                (Baseline::Strict(pruned), removed)
            }
            Baseline::Loose(loose) => {
                let (pruned, removed) = loose.prune_outdated_entries(issues, read_database);
                (Baseline::Loose(pruned), removed)
            }
        }
    }

    /// Returns the variant of this baseline.
    #[must_use]
    pub fn variant(&self) -> BaselineVariant {
        match self {
            Baseline::Strict(_) => BaselineVariant::Strict,
            Baseline::Loose(_) => BaselineVariant::Loose,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::get_unwrap)]
mod tests {
    use super::*;
    use crate::Annotation;
    use crate::Issue;
    use mago_database::Database;
    use mago_database::file::File;
    use mago_database::file::FileId;
    use mago_span::Position;
    use mago_span::Span;

    fn create_test_database() -> (Database<'static>, FileId) {
        let file =
            File::ephemeral(Cow::Borrowed(b"test.php"), Cow::Borrowed(b"<?php\n// Line 1\n// Line 2\n// Line 3\n"));
        let file_id = file.id;
        let config =
            mago_database::DatabaseConfiguration::new(std::path::Path::new("/"), vec![], vec![], vec![], vec![])
                .into_static();
        let db = Database::single(file, config);
        (db, file_id)
    }

    fn create_test_issue(file_id: FileId, code: &str, start_offset: u32, end_offset: u32) -> Issue {
        Issue::error("test error").with_code(code).with_annotation(Annotation::primary(Span::new(
            file_id,
            Position::new(start_offset),
            Position::new(end_offset),
        )))
    }

    fn create_test_issue_with_message(
        file_id: FileId,
        code: &str,
        message: &str,
        start_offset: u32,
        end_offset: u32,
    ) -> Issue {
        Issue::error(message).with_code(code).with_annotation(Annotation::primary(Span::new(
            file_id,
            Position::new(start_offset),
            Position::new(end_offset),
        )))
    }

    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path(b"foo/bar/baz.php"), "foo/bar/baz.php");
        assert_eq!(normalize_path(b"foo\\bar\\baz.php"), "foo/bar/baz.php");
        assert_eq!(normalize_path(b"C:\\Users\\test\\file.php"), "C:/Users/test/file.php");
    }

    #[test]
    fn test_strict_generate_baseline_from_issues() {
        let (db, file_id) = create_test_database();
        let read_db = db.read_only();

        let mut issues = IssueCollection::new();
        issues.push(create_test_issue(file_id, "E001", 0, 5));
        issues.push(create_test_issue(file_id, "E002", 10, 15));

        let baseline = StrictBaseline::generate_from_issues(&issues, &read_db);

        assert_eq!(baseline.variant, Some(BaselineVariant::Strict));
        assert_eq!(baseline.entries.len(), 1);
        let entry = baseline.entries.get("test.php").unwrap();
        assert_eq!(entry.issues.len(), 2);
    }

    #[test]
    fn test_strict_filter_issues() {
        let (db, file_id) = create_test_database();
        let read_db = db.read_only();

        let mut baseline = StrictBaseline::new();
        let mut entry = StrictBaselineEntry::default();
        entry.issues.push(StrictBaselineIssue { code: "E001".to_string(), start_line: 0, end_line: 0 });
        baseline.entries.insert(Cow::Borrowed("test.php"), entry);

        let mut issues = IssueCollection::new();
        issues.push(create_test_issue(file_id, "E001", 0, 5));
        issues.push(create_test_issue(file_id, "E002", 10, 15));

        let filtered = baseline.filter_issues(issues, &read_db);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered.iter().next().unwrap().code.as_ref().unwrap(), "E002");
    }

    #[test]
    fn test_strict_compare_baseline_with_issues() {
        let (db, file_id) = create_test_database();
        let read_db = db.read_only();

        let mut baseline = StrictBaseline::new();
        let mut entry = StrictBaselineEntry::default();
        entry.issues.push(StrictBaselineIssue { code: "E001".to_string(), start_line: 0, end_line: 0 });
        entry.issues.push(StrictBaselineIssue { code: "E003".to_string(), start_line: 2, end_line: 2 });
        baseline.entries.insert(Cow::Borrowed("test.php"), entry);

        let mut issues = IssueCollection::new();
        issues.push(create_test_issue(file_id, "E001", 0, 5));
        issues.push(create_test_issue(file_id, "E002", 10, 15));

        let result = baseline.compare_with_issues(&issues, &read_db);

        assert!(!result.is_up_to_date);
        assert_eq!(result.removed_issues.len(), 1);
        assert_eq!(result.new_issues.len(), 1);
        assert_eq!(result.files_with_changes_count, 1);
    }

    #[test]
    fn test_loose_generate_baseline_from_issues() {
        let (db, file_id) = create_test_database();
        let read_db = db.read_only();

        let mut issues = IssueCollection::new();
        issues.push(create_test_issue_with_message(file_id, "E001", "error 1", 0, 5));
        issues.push(create_test_issue_with_message(file_id, "E001", "error 1", 10, 15));
        issues.push(create_test_issue_with_message(file_id, "E002", "error 2", 20, 25));

        let baseline = LooseBaseline::generate_from_issues(&issues, &read_db);

        assert_eq!(baseline.variant, BaselineVariant::Loose);
        assert_eq!(baseline.issues.len(), 2);

        let e001 = baseline.issues.iter().find(|i| i.code == "E001").unwrap();
        assert_eq!(e001.count, 2);

        let e002 = baseline.issues.iter().find(|i| i.code == "E002").unwrap();
        assert_eq!(e002.count, 1);
    }

    #[test]
    fn test_loose_filter_issues() {
        let (db, file_id) = create_test_database();
        let read_db = db.read_only();

        let baseline = LooseBaseline {
            variant: BaselineVariant::Loose,
            issues: vec![LooseBaselineIssue {
                file: "test.php".to_string(),
                code: "E001".to_string(),
                message: "test error".to_string(),
                count: 2,
            }],
        };

        let mut issues = IssueCollection::new();
        issues.push(create_test_issue(file_id, "E001", 0, 5));
        issues.push(create_test_issue(file_id, "E001", 10, 15));
        issues.push(create_test_issue(file_id, "E001", 20, 25));

        let filtered = baseline.filter_issues(issues, &read_db);

        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_loose_compare_baseline_with_issues() {
        let (db, file_id) = create_test_database();
        let read_db = db.read_only();

        let baseline = LooseBaseline {
            variant: BaselineVariant::Loose,
            issues: vec![
                LooseBaselineIssue {
                    file: "test.php".to_string(),
                    code: "E001".to_string(),
                    message: "test error".to_string(),
                    count: 2,
                },
                LooseBaselineIssue {
                    file: "test.php".to_string(),
                    code: "E003".to_string(),
                    message: "test error".to_string(),
                    count: 1,
                },
            ],
        };

        let mut issues = IssueCollection::new();
        issues.push(create_test_issue(file_id, "E001", 0, 5));
        issues.push(create_test_issue(file_id, "E002", 10, 15));

        let result = baseline.compare_with_issues(&issues, &read_db);

        assert!(!result.is_up_to_date);
        assert_eq!(result.new_issues.len(), 1);
        assert_eq!(result.removed_issues.len(), 2);
        assert_eq!(result.files_with_changes_count, 1);
    }

    #[test]
    fn test_unified_baseline_generate_strict() {
        let (db, file_id) = create_test_database();
        let read_db = db.read_only();

        let mut issues = IssueCollection::new();
        issues.push(create_test_issue(file_id, "E001", 0, 5));

        let baseline = Baseline::generate_from_issues(&issues, &read_db, BaselineVariant::Strict);

        assert!(matches!(baseline, Baseline::Strict(_)));
        assert_eq!(baseline.variant(), BaselineVariant::Strict);
    }

    #[test]
    fn test_unified_baseline_generate_loose() {
        let (db, file_id) = create_test_database();
        let read_db = db.read_only();

        let mut issues = IssueCollection::new();
        issues.push(create_test_issue(file_id, "E001", 0, 5));

        let baseline = Baseline::generate_from_issues(&issues, &read_db, BaselineVariant::Loose);

        assert!(matches!(baseline, Baseline::Loose(_)));
        assert_eq!(baseline.variant(), BaselineVariant::Loose);
    }

    #[test]
    fn test_strict_prune_outdated_entries() {
        let (db, file_id) = create_test_database();
        let read_db = db.read_only();

        let mut baseline = StrictBaseline::new();
        let mut entry = StrictBaselineEntry::default();
        entry.issues.push(StrictBaselineIssue { code: "E001".to_string(), start_line: 0, end_line: 0 });
        entry.issues.push(StrictBaselineIssue { code: "E999".to_string(), start_line: 2, end_line: 2 });
        baseline.entries.insert(Cow::Borrowed("test.php"), entry);

        let mut issues = IssueCollection::new();
        issues.push(create_test_issue(file_id, "E001", 0, 5));
        issues.push(create_test_issue(file_id, "E500", 10, 15));

        let (pruned, removed) = baseline.prune_outdated_entries(&issues, &read_db);

        assert_eq!(removed, 1, "the stale `E999` entry should be removed");
        let Some(kept) = pruned.entries.get("test.php") else {
            panic!("the file entry should be kept");
        };
        assert_eq!(kept.issues.len(), 1, "the new `E500` issue must not be added");
        assert_eq!(kept.issues.first().map(|issue| issue.code.as_str()), Some("E001"));
    }

    #[test]
    fn test_strict_prune_drops_emptied_file_entry() {
        let (db, _file_id) = create_test_database();
        let read_db = db.read_only();

        let mut baseline = StrictBaseline::new();
        let mut entry = StrictBaselineEntry::default();
        entry.issues.push(StrictBaselineIssue { code: "E999".to_string(), start_line: 2, end_line: 2 });
        baseline.entries.insert(Cow::Borrowed("test.php"), entry);

        let issues = IssueCollection::new();

        let (pruned, removed) = baseline.prune_outdated_entries(&issues, &read_db);

        assert_eq!(removed, 1);
        assert!(pruned.entries.is_empty(), "a file with no surviving entries should be dropped");
    }

    #[test]
    fn test_loose_prune_outdated_entries() {
        let (db, file_id) = create_test_database();
        let read_db = db.read_only();

        let baseline = LooseBaseline {
            variant: BaselineVariant::Loose,
            issues: vec![
                LooseBaselineIssue {
                    file: "test.php".to_string(),
                    code: "E001".to_string(),
                    message: "test error".to_string(),
                    count: 3,
                },
                LooseBaselineIssue {
                    file: "test.php".to_string(),
                    code: "E999".to_string(),
                    message: "test error".to_string(),
                    count: 1,
                },
            ],
        };

        let mut issues = IssueCollection::new();
        issues.push(create_test_issue(file_id, "E001", 0, 5));
        issues.push(create_test_issue(file_id, "E500", 10, 15));

        let (pruned, removed) = baseline.prune_outdated_entries(&issues, &read_db);

        assert_eq!(removed, 3);
        assert_eq!(pruned.issues.len(), 1, "the new `E500` issue must not be added");
        assert_eq!(pruned.issues[0].code, "E001");
        assert_eq!(pruned.issues[0].count, 1, "the count must be capped at the current occurrences");
    }
}
