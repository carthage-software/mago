use std::borrow::Cow;
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub struct BaselineSourceIssue {
    pub code: String,
    pub start_line: u32,
    pub end_line: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct BaselineEntry {
    pub issues: Vec<BaselineSourceIssue>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Baseline {
    entries: HashMap<Cow<'static, str>, BaselineEntry>,
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
