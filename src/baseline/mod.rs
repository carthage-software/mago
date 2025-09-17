use std::borrow::Cow;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

use serde::Deserialize;
use serde::Serialize;

use mago_database::DatabaseReader;
use mago_database::ReadDatabase;
use mago_reporting::IssueCollection;

use crate::error::Error;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
pub struct BaselineSourceIssue {
    pub code: String,
    pub start_line: u32,
    pub end_line: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct BaselineEntry {
    pub issues: Vec<BaselineSourceIssue>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Baseline {
    pub entries: BTreeMap<Cow<'static, str>, BaselineEntry>,
}

/// Generates a `Baseline` from a collection of issues.
///
/// This function processes a list of issues and groups them by source file,
/// calculating a content hash for each file to ensure the baseline is only
/// applied to unmodified files.
pub fn generate_baseline_from_issues(issues: IssueCollection, database: &ReadDatabase) -> Result<Baseline, Error> {
    let mut baseline = Baseline::default();

    for issue in issues {
        let Some(code) = issue.code else { continue };
        let Some(annotation) = issue
            .annotations
            .iter()
            .find(|a| a.is_primary())
            .or_else(|| issue.annotations.iter().find(|a| !a.is_primary()))
        else {
            tracing::warn!("Issue with code '{code}' has no annotations, it will not be included in the baseline.");

            continue;
        };

        let start = annotation.span.start;
        let end = annotation.span.end;
        let source_file = database.get(&annotation.span.file_id)?;

        let entry = baseline.entries.entry(source_file.name.clone()).or_default();

        entry.issues.push(BaselineSourceIssue {
            code: code.to_string(),
            start_line: source_file.line_number(start.offset),
            end_line: source_file.line_number(end.offset),
        });
    }

    // Sort issues within each entry for consistent ordering
    for entry in baseline.entries.values_mut() {
        entry.issues.sort();
    }

    Ok(baseline)
}

/// Serializes a `Baseline` to a TOML file.
///
/// If a file already exists at the given path, it will be handled based on the `backup` flag.
///
/// # Arguments
///
/// * `path` - The path to write the baseline file to.
/// * `baseline` - The `Baseline` object to serialize.
/// * `backup` - If `true`, renames an existing baseline file to `[path].bkp`. If `false`, deletes it.
pub fn serialize_baseline(path: &Path, baseline: &Baseline, backup: bool) -> Result<(), Error> {
    if path.exists() {
        if backup {
            let backup_path = path.with_extension("toml.bkp");
            fs::rename(path, backup_path).map_err(Error::CreatingBaselineFile)?;
        } else {
            fs::remove_file(path).map_err(Error::CreatingBaselineFile)?;
        }
    }

    let toml_string = toml::to_string_pretty(baseline).map_err(Error::SerializingToml)?;
    fs::write(path, toml_string).map_err(Error::CreatingBaselineFile)?;
    Ok(())
}

/// Deserializes a `Baseline` from a TOML file.
pub fn unserialize_baseline(path: &Path) -> Result<Baseline, Error> {
    let toml_string = fs::read_to_string(path).map_err(Error::ReadingBaselineFile)?;
    toml::from_str(&toml_string).map_err(Error::DeserializingToml)
}

/// Filters a collection of `Issue` objects against a baseline.
///
/// # Returns
///
/// A tuple containing:
///
/// 1. `IssueCollection`: The collection of issues that were *not* found in the baseline.
/// 2. `usize`: The number of issues that were found in the baseline and thus filtered out.
/// 3. `bool`: `true` if the baseline contains dead/stale issues that no longer exist in the code.
pub fn filter_issues(
    baseline: &Baseline,
    issues: IssueCollection,
    database: &ReadDatabase,
) -> Result<(IssueCollection, usize, bool), Error> {
    let baseline_sets: HashMap<Cow<'static, str>, HashSet<BaselineSourceIssue>> =
        baseline.entries.iter().map(|(path, entry)| (path.clone(), entry.issues.iter().cloned().collect())).collect();

    let mut filtered_issues = IssueCollection::new();
    let mut seen_baseline_issues: HashMap<Cow<'static, str>, HashSet<BaselineSourceIssue>> = HashMap::new();

    for issue in issues {
        let Some(annotation) = issue
            .annotations
            .iter()
            .find(|a| a.is_primary())
            .or_else(|| issue.annotations.iter().find(|a| !a.is_primary()))
        else {
            filtered_issues.push(issue);
            continue;
        };

        let source_file = database.get(&annotation.span.file_id)?;

        let Some(baseline_issue_set) = baseline_sets.get(&source_file.name) else {
            // File is not in the baseline, so the issue is new.
            filtered_issues.push(issue);
            continue;
        };

        let Some(code) = &issue.code else {
            filtered_issues.push(issue);
            continue;
        };

        let issue_to_check = BaselineSourceIssue {
            code: code.to_string(),
            start_line: source_file.line_number(annotation.span.start.offset),
            end_line: source_file.line_number(annotation.span.end.offset),
        };

        if baseline_issue_set.contains(&issue_to_check) {
            // Issue is in the baseline, so we ignore it and mark it as "seen".
            seen_baseline_issues.entry(source_file.name.clone()).or_default().insert(issue_to_check);
        } else {
            // Issue is not in the baseline, so it's a new one.
            filtered_issues.push(issue);
        }
    }

    let seen_count = seen_baseline_issues.values().map(|set| set.len()).sum();

    // Check for dead issues (in baseline but not "seen").
    let mut has_dead_issues = false;
    for (path, baseline_issue_set) in &baseline_sets {
        if let Some(seen_set) = seen_baseline_issues.get(path) {
            if seen_set.len() != baseline_issue_set.len() {
                has_dead_issues = true;
                break;
            }
        } else {
            // If we have a baseline for a file but saw no issues from it, all its baseline issues are dead.
            // This can happen if all issues in a file were fixed.
            has_dead_issues = true;
            break;
        }
    }

    Ok((filtered_issues, seen_count, has_dead_issues))
}

/// Represents the result of comparing a baseline against current issues.
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

/// Compares a baseline against current issues to determine if it's up to date.
///
/// This function generates a baseline from the current issues and compares it
/// against the provided baseline to determine what has changed.
///
/// # Arguments
///
/// * `baseline` - The existing baseline to compare against
/// * `issues` - The current collection of issues
/// * `database` - The read-only database for file information
///
/// # Returns
///
/// A `BaselineComparisonResult` containing detailed information about the differences
pub fn compare_baseline_with_issues(
    baseline: &Baseline,
    issues: IssueCollection,
    database: &ReadDatabase,
) -> Result<BaselineComparisonResult, Error> {
    let current_baseline = generate_baseline_from_issues(issues, database)?;

    // Quick check - if they're exactly the same, we're done
    if baseline.entries == current_baseline.entries {
        return Ok(BaselineComparisonResult {
            is_up_to_date: true,
            new_issues_count: 0,
            removed_issues_count: 0,
            files_with_changes_count: 0,
        });
    }

    // Analyze what's different
    let mut new_issues = 0;
    let mut removed_issues = 0;
    let mut files_with_changes = std::collections::HashSet::new();

    // Check for new issues (in current but not in baseline)
    for (file_path, current_entry) in &current_baseline.entries {
        if let Some(baseline_entry) = baseline.entries.get(file_path) {
            let baseline_issues: std::collections::HashSet<_> = baseline_entry.issues.iter().collect();
            let current_issues: std::collections::HashSet<_> = current_entry.issues.iter().collect();

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
    for (file_path, baseline_entry) in &baseline.entries {
        if !current_baseline.entries.contains_key(file_path) {
            removed_issues += baseline_entry.issues.len();
            files_with_changes.insert(file_path.as_ref());
        }
    }

    Ok(BaselineComparisonResult {
        is_up_to_date: false,
        new_issues_count: new_issues,
        removed_issues_count: removed_issues,
        files_with_changes_count: files_with_changes.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::borrow::Cow;

    use tempfile::NamedTempFile;

    // Mock structures for testing
    use mago_database::Database;
    use mago_reporting::IssueCollection;

    #[test]
    fn test_baseline_source_issue_ordering() {
        let mut issues = [
            BaselineSourceIssue { code: "error.type".to_string(), start_line: 5, end_line: 5 },
            BaselineSourceIssue { code: "error.syntax".to_string(), start_line: 3, end_line: 3 },
            BaselineSourceIssue { code: "error.syntax".to_string(), start_line: 1, end_line: 1 },
            BaselineSourceIssue { code: "error.syntax".to_string(), start_line: 3, end_line: 5 },
        ];

        issues.sort();

        // Should be sorted by: code, then start_line, then end_line
        assert_eq!(issues[0].code, "error.syntax");
        assert_eq!(issues[0].start_line, 1);

        assert_eq!(issues[1].code, "error.syntax");
        assert_eq!(issues[1].start_line, 3);
        assert_eq!(issues[1].end_line, 3);

        assert_eq!(issues[2].code, "error.syntax");
        assert_eq!(issues[2].start_line, 3);
        assert_eq!(issues[2].end_line, 5);

        assert_eq!(issues[3].code, "error.type");
        assert_eq!(issues[3].start_line, 5);
    }

    #[test]
    fn test_baseline_source_issue_equality() {
        let issue1 = BaselineSourceIssue { code: "error.test".to_string(), start_line: 10, end_line: 10 };

        let issue2 = BaselineSourceIssue { code: "error.test".to_string(), start_line: 10, end_line: 10 };

        let issue3 = BaselineSourceIssue { code: "error.test".to_string(), start_line: 11, end_line: 11 };

        assert_eq!(issue1, issue2);
        assert_ne!(issue1, issue3);
    }

    #[test]
    fn test_baseline_entry_equality() {
        let entry1 = BaselineEntry {
            issues: vec![BaselineSourceIssue { code: "error.test".to_string(), start_line: 1, end_line: 1 }],
        };

        let entry2 = BaselineEntry {
            issues: vec![BaselineSourceIssue { code: "error.test".to_string(), start_line: 1, end_line: 1 }],
        };

        let entry3 = BaselineEntry {
            issues: vec![BaselineSourceIssue { code: "error.different".to_string(), start_line: 1, end_line: 1 }],
        };

        assert_eq!(entry1, entry2);
        assert_ne!(entry1, entry3);
    }

    #[test]
    fn test_baseline_default() {
        let baseline = Baseline::default();
        assert!(baseline.entries.is_empty());
    }

    #[test]
    fn test_baseline_serialization_roundtrip() {
        let mut baseline = Baseline::default();
        baseline.entries.insert(
            Cow::Borrowed("src/main.php"),
            BaselineEntry {
                issues: vec![
                    BaselineSourceIssue { code: "error.syntax".to_string(), start_line: 10, end_line: 10 },
                    BaselineSourceIssue { code: "error.type".to_string(), start_line: 20, end_line: 22 },
                ],
            },
        );
        baseline.entries.insert(
            Cow::Borrowed("src/helper.php"),
            BaselineEntry {
                issues: vec![BaselineSourceIssue { code: "warning.unused".to_string(), start_line: 5, end_line: 5 }],
            },
        );

        // Serialize to TOML
        let toml_string = toml::to_string_pretty(&baseline).expect("Failed to serialize baseline");

        // Check that files are in alphabetical order (BTreeMap guarantees this)
        let lines: Vec<&str> = toml_string.lines().collect();
        let helper_index = lines.iter().position(|line| line.contains("src/helper.php"));
        let main_index = lines.iter().position(|line| line.contains("src/main.php"));

        assert!(helper_index.is_some());
        assert!(main_index.is_some());
        assert!(helper_index.unwrap() < main_index.unwrap(), "Files should be in alphabetical order");

        // Deserialize back
        let deserialized: Baseline = toml::from_str(&toml_string).expect("Failed to deserialize baseline");

        // Check that the data matches
        assert_eq!(baseline.entries.len(), deserialized.entries.len());

        for (key, value) in &baseline.entries {
            let deserialized_value = deserialized.entries.get(key).expect("Key not found");
            assert_eq!(value, deserialized_value);
        }
    }

    #[test]
    fn test_serialize_baseline_creates_backup() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let temp_path = temp_file.path();

        // Create initial content
        std::fs::write(temp_path, "initial content").expect("Failed to write initial content");

        let baseline = Baseline::default();

        // Serialize with backup
        serialize_baseline(temp_path, &baseline, true).expect("Failed to serialize baseline");

        // Check that backup file was created
        let backup_path = temp_path.with_extension("toml.bkp");
        assert!(backup_path.exists(), "Backup file should be created");

        let backup_content = std::fs::read_to_string(&backup_path).expect("Failed to read backup");
        assert_eq!(backup_content, "initial content");

        // Check that new file contains the baseline
        let new_content = std::fs::read_to_string(temp_path).expect("Failed to read new content");
        assert!(new_content.contains("[entries]") || new_content.contains("entries = {}"));
    }

    #[test]
    fn test_serialize_baseline_without_backup() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let temp_path = temp_file.path();

        // Create initial content
        std::fs::write(temp_path, "initial content").expect("Failed to write initial content");

        let baseline = Baseline::default();

        // Serialize without backup
        serialize_baseline(temp_path, &baseline, false).expect("Failed to serialize baseline");

        // Check that backup file was NOT created
        let backup_path = temp_path.with_extension("toml.bkp");
        assert!(!backup_path.exists(), "Backup file should not be created");

        // Check that new file contains the baseline
        let new_content = std::fs::read_to_string(temp_path).expect("Failed to read new content");
        assert!(new_content.contains("[entries]") || new_content.contains("entries = {}"));
    }

    #[test]
    fn test_serialize_baseline_new_file() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let temp_path = temp_dir.path().join("new_baseline.toml");

        let mut baseline = Baseline::default();
        baseline.entries.insert(
            Cow::Borrowed("test.php"),
            BaselineEntry {
                issues: vec![BaselineSourceIssue { code: "error.test".to_string(), start_line: 1, end_line: 1 }],
            },
        );

        serialize_baseline(&temp_path, &baseline, true).expect("Failed to serialize baseline");

        assert!(temp_path.exists(), "New file should be created");

        let content = std::fs::read_to_string(&temp_path).expect("Failed to read content");
        assert!(content.contains("test.php"));
        assert!(content.contains("error.test"));
    }

    #[test]
    fn test_unserialize_baseline_valid_file() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let temp_path = temp_file.path();

        let toml_content = r#"
[entries."src/test.php"]
issues = [
    { code = "error.syntax", start_line = 10, end_line = 10 },
    { code = "error.type", start_line = 20, end_line = 22 },
]

[entries."src/helper.php"]
issues = [
    { code = "warning.unused", start_line = 5, end_line = 5 },
]
"#;

        std::fs::write(temp_path, toml_content).expect("Failed to write TOML content");

        let baseline = unserialize_baseline(temp_path).expect("Failed to deserialize baseline");

        assert_eq!(baseline.entries.len(), 2);

        let test_entry = baseline.entries.get("src/test.php").expect("test.php entry not found");
        assert_eq!(test_entry.issues.len(), 2);
        assert_eq!(test_entry.issues[0].code, "error.syntax");
        assert_eq!(test_entry.issues[0].start_line, 10);
        assert_eq!(test_entry.issues[1].code, "error.type");
        assert_eq!(test_entry.issues[1].start_line, 20);
        assert_eq!(test_entry.issues[1].end_line, 22);

        let helper_entry = baseline.entries.get("src/helper.php").expect("helper.php entry not found");
        assert_eq!(helper_entry.issues.len(), 1);
        assert_eq!(helper_entry.issues[0].code, "warning.unused");
        assert_eq!(helper_entry.issues[0].start_line, 5);
    }

    #[test]
    fn test_unserialize_baseline_empty_file() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let temp_path = temp_file.path();

        let toml_content = r#"
[entries]
"#;

        std::fs::write(temp_path, toml_content).expect("Failed to write TOML content");

        let baseline = unserialize_baseline(temp_path).expect("Failed to deserialize baseline");

        assert_eq!(baseline.entries.len(), 0);
    }

    #[test]
    fn test_unserialize_baseline_nonexistent_file() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let nonexistent_path = temp_dir.path().join("nonexistent.toml");

        let result = unserialize_baseline(&nonexistent_path);
        assert!(result.is_err(), "Should fail when file doesn't exist");

        if let Err(Error::ReadingBaselineFile(_)) = result {
            // Expected error type
        } else {
            panic!("Expected ReadingBaselineFile error");
        }
    }

    #[test]
    fn test_unserialize_baseline_invalid_toml() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let temp_path = temp_file.path();

        let invalid_toml = "this is not valid toml content [[[";
        std::fs::write(temp_path, invalid_toml).expect("Failed to write invalid TOML");

        let result = unserialize_baseline(temp_path);
        assert!(result.is_err(), "Should fail with invalid TOML");

        if let Err(Error::DeserializingToml(_)) = result {
            // Expected error type
        } else {
            panic!("Expected DeserializingToml error");
        }
    }

    #[test]
    fn test_baseline_entries_are_sorted_by_filename() {
        let mut baseline = Baseline::default();

        // Add entries in reverse alphabetical order
        baseline.entries.insert(Cow::Borrowed("z_last.php"), BaselineEntry::default());
        baseline.entries.insert(Cow::Borrowed("a_first.php"), BaselineEntry::default());
        baseline.entries.insert(Cow::Borrowed("m_middle.php"), BaselineEntry::default());

        let keys: Vec<_> = baseline.entries.keys().collect();
        assert_eq!(keys, vec!["a_first.php", "m_middle.php", "z_last.php"]);
    }

    #[test]
    fn test_baseline_entry_issues_are_sorted() {
        let mut entry = BaselineEntry::default();

        // Add issues in random order
        entry.issues.push(BaselineSourceIssue { code: "error.z".to_string(), start_line: 10, end_line: 10 });
        entry.issues.push(BaselineSourceIssue { code: "error.a".to_string(), start_line: 5, end_line: 5 });
        entry.issues.push(BaselineSourceIssue { code: "error.a".to_string(), start_line: 1, end_line: 1 });

        entry.issues.sort();

        assert_eq!(entry.issues[0].code, "error.a");
        assert_eq!(entry.issues[0].start_line, 1);

        assert_eq!(entry.issues[1].code, "error.a");
        assert_eq!(entry.issues[1].start_line, 5);

        assert_eq!(entry.issues[2].code, "error.z");
        assert_eq!(entry.issues[2].start_line, 10);
    }

    // These tests focus on the basic functionality that can be tested without mocking complex Issue structures

    #[test]
    fn test_generate_baseline_from_issues_empty() {
        let db = Database::new();
        let database = db.read_only();
        let issues = IssueCollection::new();

        let baseline = generate_baseline_from_issues(issues, &database).expect("Failed to generate baseline");

        assert!(baseline.entries.is_empty());
    }

    #[test]
    fn test_compare_baseline_with_issues_identical() {
        let db = Database::new();
        let database = db.read_only();
        let issues = IssueCollection::new();

        // Create baseline from empty issues
        let baseline1 = generate_baseline_from_issues(issues.clone(), &database).expect("Failed to generate baseline");

        let comparison =
            compare_baseline_with_issues(&baseline1, issues, &database).expect("Failed to compare baselines");

        assert!(comparison.is_up_to_date);
        assert_eq!(comparison.new_issues_count, 0);
        assert_eq!(comparison.removed_issues_count, 0);
        assert_eq!(comparison.files_with_changes_count, 0);
    }

    #[test]
    fn test_compare_baseline_with_issues_empty_vs_empty() {
        let db = Database::new();
        let database = db.read_only();
        let issues = IssueCollection::new();
        let baseline = Baseline::default();

        let comparison =
            compare_baseline_with_issues(&baseline, issues, &database).expect("Failed to compare baselines");

        assert!(comparison.is_up_to_date);
        assert_eq!(comparison.new_issues_count, 0);
        assert_eq!(comparison.removed_issues_count, 0);
        assert_eq!(comparison.files_with_changes_count, 0);
    }

    #[test]
    fn test_compare_baseline_with_different_entries() {
        let db = Database::new();
        let database = db.read_only();
        let issues = IssueCollection::new();

        // Create a baseline with some entries
        let mut baseline = Baseline::default();
        baseline.entries.insert(
            Cow::Borrowed("src/test.php"),
            BaselineEntry {
                issues: vec![
                    BaselineSourceIssue { code: "error.test".to_string(), start_line: 10, end_line: 10 },
                    BaselineSourceIssue { code: "warning.unused".to_string(), start_line: 5, end_line: 5 },
                ],
            },
        );
        baseline.entries.insert(
            Cow::Borrowed("src/helper.php"),
            BaselineEntry {
                issues: vec![BaselineSourceIssue { code: "error.syntax".to_string(), start_line: 1, end_line: 1 }],
            },
        );

        // Compare against empty issues (everything is removed)
        let comparison =
            compare_baseline_with_issues(&baseline, issues, &database).expect("Failed to compare baselines");

        assert!(!comparison.is_up_to_date);
        assert_eq!(comparison.new_issues_count, 0);
        assert_eq!(comparison.removed_issues_count, 3); // 2 + 1 issues removed
        assert_eq!(comparison.files_with_changes_count, 2); // 2 files changed
    }

    #[test]
    fn test_baseline_comparison_result_debug() {
        let result = BaselineComparisonResult {
            is_up_to_date: false,
            new_issues_count: 5,
            removed_issues_count: 3,
            files_with_changes_count: 2,
        };

        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("is_up_to_date: false"));
        assert!(debug_str.contains("new_issues_count: 5"));
        assert!(debug_str.contains("removed_issues_count: 3"));
        assert!(debug_str.contains("files_with_changes_count: 2"));
    }

    #[test]
    fn test_baseline_comparison_result_clone() {
        let result = BaselineComparisonResult {
            is_up_to_date: true,
            new_issues_count: 0,
            removed_issues_count: 0,
            files_with_changes_count: 0,
        };

        let cloned = result.clone();
        assert_eq!(result.is_up_to_date, cloned.is_up_to_date);
        assert_eq!(result.new_issues_count, cloned.new_issues_count);
        assert_eq!(result.removed_issues_count, cloned.removed_issues_count);
        assert_eq!(result.files_with_changes_count, cloned.files_with_changes_count);
    }
}
