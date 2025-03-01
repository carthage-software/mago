use std::str::FromStr;

use mago_php_version::PHPVersion;

pub fn extract_minimum_php_version(version_constraint: &str) -> Option<String> {
    // Remove any whitespace and split by logical OR operator
    let trimmed_constraint = version_constraint.replace(" ", "");
    let constraints = trimmed_constraint.split("||").collect::<Vec<_>>();

    // For each constraint, try to extract the minimum version
    let mut min_versions = Vec::new();
    for constraint in constraints {
        if let Some(version) = extract_version_from_constraint(constraint) {
            min_versions.push(version);
        }
    }

    // If we found any valid versions, return the minimum
    if !min_versions.is_empty() {
        min_versions.sort();

        return Some(min_versions[0].to_string());
    }

    None
}

fn extract_version_from_constraint(constraint: &str) -> Option<PHPVersion> {
    // Handle common version constraint patterns

    // Case: >=8.0
    if constraint.starts_with(">=") {
        let version_str = constraint.trim_start_matches(">=");

        return PHPVersion::from_str(version_str).ok();
    }

    // Case: >8.0 (we add 0.0.1 to get the minimum allowed version)
    if constraint.starts_with(">") {
        let version_str = constraint.trim_start_matches(">");
        if let Ok(version) = PHPVersion::from_str(version_str) {
            // Increment the patch version to get the minimum allowed
            return Some(PHPVersion::new(
                version.major(),
                version.minor(),
                version.patch().checked_add(1).unwrap_or(0),
            ));
        }
    }

    // Case: ~8.1 (compatible with 8.1.x)
    if constraint.starts_with("~") {
        let version_str = constraint.trim_start_matches("~");

        return PHPVersion::from_str(version_str).ok();
    }

    // Case: ^8.1 (compatible with 8.x.y where x >= 1)
    if constraint.starts_with("^") {
        let version_str = constraint.trim_start_matches("^");

        return PHPVersion::from_str(version_str).ok();
    }

    // Case: 8.1.* or 8.1
    if !constraint.starts_with("<") && !constraint.contains(">") {
        let version_str = constraint.replace(".*", "").replace("*", "");

        return PHPVersion::from_str(&version_str).ok();
    }

    None
}
