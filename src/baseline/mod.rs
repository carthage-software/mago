use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::error::Error;

// Re-export baseline types from the reporting crate
pub use mago_reporting::baseline::Baseline;
pub use mago_reporting::baseline::BaselineVariant;
pub use mago_reporting::baseline::LooseBaseline;
pub use mago_reporting::baseline::StrictBaseline;

/// Intermediate struct for detecting variant from file header.
#[derive(Deserialize)]
struct BaselineHeader {
    #[serde(default)]
    variant: Option<BaselineVariant>,
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

    let toml_string = match baseline {
        Baseline::Strict(strict) => toml::to_string_pretty(strict).map_err(Error::SerializingToml)?,
        Baseline::Loose(loose) => toml::to_string_pretty(loose).map_err(Error::SerializingToml)?,
    };

    fs::write(path, toml_string).map_err(Error::CreatingBaselineFile)?;
    Ok(())
}

/// Deserializes a `Baseline` from a TOML file.
///
/// Returns a tuple of `(Baseline, bool)` where the boolean indicates if a warning should be shown.
/// The warning is shown when the baseline file does not have a `variant` header, indicating it was
/// created with an older version of mago. In this case, the baseline is assumed to be strict.
///
/// # Arguments
///
/// * `path` - The path to read the baseline file from.
///
/// # Returns
///
/// * `Ok((baseline, needs_warning))` - The deserialized baseline and whether a warning is needed.
/// * `Err(_)` - If reading or parsing the file fails.
pub fn unserialize_baseline(path: &Path) -> Result<(Baseline, bool), Error> {
    let toml_string = fs::read_to_string(path).map_err(Error::ReadingBaselineFile)?;

    // First pass: detect variant from header
    let header: BaselineHeader = toml::from_str(&toml_string).map_err(Error::DeserializingToml)?;

    match header.variant {
        Some(BaselineVariant::Loose) => {
            let loose: LooseBaseline = toml::from_str(&toml_string).map_err(Error::DeserializingToml)?;
            Ok((Baseline::Loose(loose), false))
        }
        Some(BaselineVariant::Strict) => {
            let strict: StrictBaseline = toml::from_str(&toml_string).map_err(Error::DeserializingToml)?;
            Ok((Baseline::Strict(strict), false))
        }
        None => {
            // No variant header - assume strict for backward compatibility
            let strict: StrictBaseline = toml::from_str(&toml_string).map_err(Error::DeserializingToml)?;
            Ok((Baseline::Strict(strict), true)) // needs_warning = true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mago_reporting::baseline::LooseBaselineIssue;
    use mago_reporting::baseline::StrictBaselineEntry;
    use mago_reporting::baseline::StrictBaselineIssue;
    use std::borrow::Cow;
    use tempfile::NamedTempFile;

    #[test]
    fn test_serialize_strict_baseline_creates_backup() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let temp_path = temp_file.path();

        // Create initial content
        std::fs::write(temp_path, "initial content").expect("Failed to write initial content");

        let baseline = Baseline::Strict(StrictBaseline::new());

        // Serialize with backup
        serialize_baseline(temp_path, &baseline, true).expect("Failed to serialize baseline");

        // Check that backup file was created
        let backup_path = temp_path.with_extension("toml.bkp");
        assert!(backup_path.exists(), "Backup file should be created");

        let backup_content = std::fs::read_to_string(&backup_path).expect("Failed to read backup");
        assert_eq!(backup_content, "initial content");

        // Check that new file contains the baseline with variant header
        let new_content = std::fs::read_to_string(temp_path).expect("Failed to read new content");
        assert!(new_content.contains("variant = \"strict\""));
    }

    #[test]
    fn test_serialize_loose_baseline() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let temp_path = temp_dir.path().join("loose_baseline.toml");

        let baseline = Baseline::Loose(LooseBaseline {
            variant: BaselineVariant::Loose,
            issues: vec![LooseBaselineIssue {
                file: "src/Service/PaymentProcessor.php".to_string(),
                code: "null-argument".to_string(),
                message: "Argument #1 of `process` cannot be `null`.".to_string(),
                count: 2,
            }],
        });

        serialize_baseline(&temp_path, &baseline, false).expect("Failed to serialize baseline");

        let content = std::fs::read_to_string(&temp_path).expect("Failed to read content");
        assert!(content.contains("variant = \"loose\""));
        assert!(content.contains("src/Service/PaymentProcessor.php"));
        assert!(content.contains("null-argument"));
        assert!(content.contains("count = 2"));
    }

    #[test]
    fn test_serialize_strict_baseline_new_file() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let temp_path = temp_dir.path().join("new_baseline.toml");

        let mut strict = StrictBaseline::new();
        strict.entries.insert(
            Cow::Borrowed("src/Controller/UserController.php"),
            StrictBaselineEntry {
                issues: vec![StrictBaselineIssue {
                    code: "redundant-docblock-type".to_string(),
                    start_line: 29,
                    end_line: 29,
                }],
            },
        );
        let baseline = Baseline::Strict(strict);

        serialize_baseline(&temp_path, &baseline, true).expect("Failed to serialize baseline");

        assert!(temp_path.exists(), "New file should be created");

        let content = std::fs::read_to_string(&temp_path).expect("Failed to read content");
        assert!(content.contains("variant = \"strict\""));
        assert!(content.contains("src/Controller/UserController.php"));
        assert!(content.contains("redundant-docblock-type"));
    }

    #[test]
    fn test_unserialize_strict_baseline_with_header() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let temp_path = temp_file.path();

        let toml_content = r#"
variant = "strict"

[[entries."src/Service/PaymentProcessor.php".issues]]
code = "possibly-null-argument"
start_line = 42
end_line = 42
"#;

        std::fs::write(temp_path, toml_content).expect("Failed to write TOML content");

        let (baseline, needs_warning) = unserialize_baseline(temp_path).expect("Failed to deserialize baseline");

        assert!(!needs_warning, "Should not need warning with variant header");
        assert!(matches!(baseline, Baseline::Strict(_)));

        if let Baseline::Strict(strict) = baseline {
            assert_eq!(strict.entries.len(), 1);
            let entry = strict.entries.get("src/Service/PaymentProcessor.php").expect("entry not found");
            assert_eq!(entry.issues[0].code, "possibly-null-argument");
        }
    }

    #[test]
    fn test_unserialize_loose_baseline() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let temp_path = temp_file.path();

        let toml_content = r#"
variant = "loose"

[[issues]]
file = "src/Service/PaymentProcessor.php"
code = "possibly-null-argument"
message = "Argument #1 of `process` expects `Order`, but `?Order` was given."
count = 3
"#;

        std::fs::write(temp_path, toml_content).expect("Failed to write TOML content");

        let (baseline, needs_warning) = unserialize_baseline(temp_path).expect("Failed to deserialize baseline");

        assert!(!needs_warning, "Should not need warning with variant header");
        assert!(matches!(baseline, Baseline::Loose(_)));

        if let Baseline::Loose(loose) = baseline {
            assert_eq!(loose.issues.len(), 1);
            assert_eq!(loose.issues[0].file, "src/Service/PaymentProcessor.php");
            assert_eq!(loose.issues[0].code, "possibly-null-argument");
            assert_eq!(loose.issues[0].count, 3);
        }
    }

    #[test]
    fn test_unserialize_baseline_without_header_assumes_strict() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let temp_path = temp_file.path();

        let toml_content = r#"
[[entries."src/Service/PaymentProcessor.php".issues]]
code = "invalid-argument"
start_line = 68
end_line = 71
"#;

        std::fs::write(temp_path, toml_content).expect("Failed to write TOML content");

        let (baseline, needs_warning) = unserialize_baseline(temp_path).expect("Failed to deserialize baseline");

        assert!(needs_warning, "Should need warning without variant header");
        assert!(matches!(baseline, Baseline::Strict(_)));

        if let Baseline::Strict(strict) = baseline {
            assert_eq!(strict.entries.len(), 1);
        }
    }

    #[test]
    fn test_unserialize_baseline_empty_strict() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let temp_path = temp_file.path();

        let toml_content = r#"
variant = "strict"

[entries]
"#;

        std::fs::write(temp_path, toml_content).expect("Failed to write TOML content");

        let (baseline, needs_warning) = unserialize_baseline(temp_path).expect("Failed to deserialize baseline");

        assert!(!needs_warning);
        if let Baseline::Strict(strict) = baseline {
            assert_eq!(strict.entries.len(), 0);
        }
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
    fn test_roundtrip_strict_baseline() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let temp_path = temp_dir.path().join("roundtrip_strict.toml");

        let mut strict = StrictBaseline::new();
        strict.entries.insert(
            Cow::Owned("src/Repository/UserRepository.php".to_string()),
            StrictBaselineEntry {
                issues: vec![StrictBaselineIssue {
                    code: "invalid-argument".to_string(),
                    start_line: 68,
                    end_line: 71,
                }],
            },
        );
        let original = Baseline::Strict(strict);

        serialize_baseline(&temp_path, &original, false).expect("Failed to serialize");
        let (loaded, needs_warning) = unserialize_baseline(&temp_path).expect("Failed to deserialize");

        assert!(!needs_warning);
        assert!(matches!(loaded, Baseline::Strict(_)));
    }

    #[test]
    fn test_roundtrip_loose_baseline() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let temp_path = temp_dir.path().join("roundtrip_loose.toml");

        let loose = LooseBaseline {
            variant: BaselineVariant::Loose,
            issues: vec![LooseBaselineIssue {
                file: "src/Repository/UserRepository.php".to_string(),
                code: "possibly-invalid-argument".to_string(),
                message: "Argument #2 of `findBy` expects `array<string, mixed>`, but `mixed` was given.".to_string(),
                count: 5,
            }],
        };
        let original = Baseline::Loose(loose);

        serialize_baseline(&temp_path, &original, false).expect("Failed to serialize");
        let (loaded, needs_warning) = unserialize_baseline(&temp_path).expect("Failed to deserialize");

        assert!(!needs_warning);
        assert!(matches!(loaded, Baseline::Loose(_)));

        if let Baseline::Loose(loose) = loaded {
            assert_eq!(loose.issues.len(), 1);
            assert_eq!(loose.issues[0].count, 5);
        }
    }
}
