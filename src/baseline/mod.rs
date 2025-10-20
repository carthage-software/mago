use std::fs;
use std::path::Path;

use crate::error::Error;

// Re-export baseline types from the reporting crate
pub use mago_reporting::baseline::Baseline;

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

#[cfg(test)]
mod tests {
    use super::*;
    use mago_reporting::baseline::BaselineEntry;
    use mago_reporting::baseline::BaselineSourceIssue;
    use std::borrow::Cow;
    use tempfile::NamedTempFile;

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
}
