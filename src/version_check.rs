//! Project-version pin parsing and compatibility checking.
//!
//! A project can pin the mago binary it expects to run against by setting
//! `version = "1"`, `version = "1.19"`, or `version = "1.19.3"` in its
//! `mago.toml`. This module parses those pin strings and compares them to
//! the running binary's version so that [`crate::run`] can warn on drift and
//! hard-fail on a major mismatch.
//!
//! # Invariants (**do not break**)
//!
//! The `version` field's location (top-level of `mago.toml`) and type
//! (string) are a load-bearing compatibility contract. A future mago 2.x
//! must be able to pick it up from a mago 1.x config via a permissive
//! top-level TOML pass and reject it with "this config is pinned to mago 1"
//! *before* hitting its own strict schema parser. That means:
//!
//! - the field must always live at the top level of the config,
//! - the field must always be a string,
//! - the pin grammar must always remain "major" / "major.minor" / "major.minor.patch".
//!
//! No `[metadata] version = ...`, no renaming, no upgrading to an object.
//! Ever.

/// Outcome of comparing a project version pin against the installed binary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionCheck {
    /// The installed binary satisfies the pin at the level it was specified at.
    Match,
    /// Only the patch component drifts (pin specified at patch level).
    PatchDrift,
    /// The minor component drifts (pin specified at minor or patch level).
    MinorDrift,
    /// The major component differs. Always fatal.
    MajorDrift,
}

/// A parsed `version = "..."` pin from `mago.toml`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VersionPin {
    major: u64,
    minor: Option<u64>,
    patch: Option<u64>,
}

impl VersionPin {
    /// Parses a pin string.
    ///
    /// Accepts `"<major>"`, `"<major>.<minor>"`, or `"<major>.<minor>.<patch>"`.
    /// Each component must be a base-10 integer. Anything else (ranges,
    /// operators, pre-release suffixes) is rejected; this is a pin grammar,
    /// not semver.
    pub fn parse(pin: &str) -> Result<Self, VersionPinParseError> {
        let trimmed = pin.trim();
        if trimmed.is_empty() {
            return Err(VersionPinParseError::EmptyPin);
        }

        let make_invalid_component = |component: &str| VersionPinParseError::InvalidPinComponent {
            pin: trimmed.to_owned(),
            component: component.to_owned(),
        };

        let mut parts = trimmed.split('.');

        let Some(major_str) = parts.next() else {
            return Err(VersionPinParseError::EmptyPin);
        };
        let major = major_str.parse::<u64>().map_err(|_| make_invalid_component(major_str))?;

        let minor = match parts.next() {
            Some(minor_str) => Some(minor_str.parse::<u64>().map_err(|_| make_invalid_component(minor_str))?),
            None => None,
        };

        let patch = match parts.next() {
            Some(patch_str) => Some(patch_str.parse::<u64>().map_err(|_| make_invalid_component(patch_str))?),
            None => None,
        };

        if parts.next().is_some() {
            return Err(VersionPinParseError::TooManyPinComponents(trimmed.to_owned()));
        }

        if patch.is_some() && minor.is_none() {
            // Unreachable given the iterator order, but belt-and-braces.
            return Err(make_invalid_component(trimmed));
        }

        Ok(VersionPin { major, minor, patch })
    }

    /// Compares this pin against the installed binary version.
    ///
    /// `installed` must be parseable as `major.minor.patch` (optionally with
    /// a pre-release or build suffix, both of which are ignored for the
    /// comparison). Non-parseable input yields a [`VersionPinParseError`].
    pub fn check(&self, installed: &str) -> Result<VersionCheck, VersionPinParseError> {
        let (installed_major, installed_minor, installed_patch) = parse_installed_version(installed)?;

        if self.major != installed_major {
            return Ok(VersionCheck::MajorDrift);
        }

        let Some(pin_minor) = self.minor else {
            return Ok(VersionCheck::Match);
        };

        if pin_minor != installed_minor {
            return Ok(VersionCheck::MinorDrift);
        }

        let Some(pin_patch) = self.patch else {
            return Ok(VersionCheck::Match);
        };

        if pin_patch != installed_patch { Ok(VersionCheck::PatchDrift) } else { Ok(VersionCheck::Match) }
    }

    /// Returns the canonical string representation of this pin
    /// (`"1"`, `"1.19"`, or `"1.19.3"`).
    #[must_use]
    pub fn as_string(&self) -> String {
        match (self.minor, self.patch) {
            (Some(minor), Some(patch)) => format!("{}.{}.{}", self.major, minor, patch),
            (Some(minor), None) => format!("{}.{}", self.major, minor),
            _ => format!("{}", self.major),
        }
    }

    /// Returns `true` when this pin nails every version component
    /// (`"major.minor.patch"`), which is the only form that maps unambiguously
    /// onto a single release tag for `self-update --to-project-version`.
    #[must_use]
    pub const fn is_exact(&self) -> bool {
        self.minor.is_some() && self.patch.is_some()
    }
}

impl std::fmt::Display for VersionPin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.as_string())
    }
}

/// Parses `"1.2.3"`, `"1.2.3-rc1"`, or `"1.2.3+build"` into `(major, minor, patch)`.
///
/// Pre-release and build suffixes are tolerated but discarded; mago only
/// compares structural version components.
fn parse_installed_version(version: &str) -> Result<(u64, u64, u64), VersionPinParseError> {
    let core = version.split(['-', '+']).next().unwrap_or(version).trim();
    let mut parts = core.split('.');

    let invalid = || VersionPinParseError::InvalidInstalledVersion(version.to_owned());

    let major = parts.next().ok_or_else(invalid)?.parse::<u64>().map_err(|_| invalid())?;
    let minor = parts.next().ok_or_else(invalid)?.parse::<u64>().map_err(|_| invalid())?;
    let patch = parts.next().ok_or_else(invalid)?.parse::<u64>().map_err(|_| invalid())?;

    Ok((major, minor, patch))
}

/// Errors that can occur while parsing a `version` pin or the installed
/// binary version.
///
/// Every variant carries enough context to describe itself, so that a
/// `From<VersionPinParseError> for crate::error::Error` impl does not need
/// extra fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionPinParseError {
    /// The pin string was empty or whitespace-only.
    EmptyPin,
    /// A component of the pin (major, minor, or patch) was not a base-10
    /// integer. Carries both the original pin and the offending component.
    InvalidPinComponent { pin: String, component: String },
    /// The pin had more than three dot-separated components. Carries the
    /// original pin.
    TooManyPinComponents(String),
    /// The installed mago binary's version string could not be parsed as
    /// `major.minor.patch`. Carries the offending version string.
    InvalidInstalledVersion(String),
}

impl std::fmt::Display for VersionPinParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyPin => f.write_str("`version` pin in mago.toml must not be empty"),
            Self::InvalidPinComponent { pin, component } => {
                write!(
                    f,
                    "Invalid `version` pin `{pin}` in mago.toml: `{component}` is not a valid version component (expected a non-negative integer)"
                )
            }
            Self::TooManyPinComponents(pin) => {
                write!(
                    f,
                    "Invalid `version` pin `{pin}` in mago.toml: must be `<major>`, `<major>.<minor>`, or `<major>.<minor>.<patch>` (at most three components)"
                )
            }
            Self::InvalidInstalledVersion(version) => {
                write!(f, "installed mago version `{version}` is not in `<major>.<minor>.<patch>` form")
            }
        }
    }
}

impl std::error::Error for VersionPinParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_major_only_pin() {
        let pin = VersionPin::parse("1").unwrap();
        assert_eq!(pin.as_string(), "1");
        assert!(!pin.is_exact());
    }

    #[test]
    fn parses_major_minor_pin() {
        let pin = VersionPin::parse("1.19").unwrap();
        assert_eq!(pin.as_string(), "1.19");
        assert!(!pin.is_exact());
    }

    #[test]
    fn parses_exact_pin() {
        let pin = VersionPin::parse("1.19.3").unwrap();
        assert_eq!(pin.as_string(), "1.19.3");
        assert!(pin.is_exact());
    }

    #[test]
    fn rejects_empty_pin() {
        assert_eq!(VersionPin::parse(""), Err(VersionPinParseError::EmptyPin));
        assert_eq!(VersionPin::parse("   "), Err(VersionPinParseError::EmptyPin));
    }

    #[test]
    fn rejects_range_operators() {
        assert!(matches!(VersionPin::parse("^1"), Err(VersionPinParseError::InvalidPinComponent { .. })));
        assert!(matches!(VersionPin::parse(">=1"), Err(VersionPinParseError::InvalidPinComponent { .. })));
        assert!(matches!(VersionPin::parse("~1.2"), Err(VersionPinParseError::InvalidPinComponent { .. })));
    }

    #[test]
    fn rejects_four_components() {
        assert_eq!(VersionPin::parse("1.2.3.4"), Err(VersionPinParseError::TooManyPinComponents("1.2.3.4".to_owned())));
    }

    #[test]
    fn rejects_non_numeric_components() {
        assert!(matches!(VersionPin::parse("1.x.3"), Err(VersionPinParseError::InvalidPinComponent { .. })));
    }

    #[test]
    fn major_pin_matches_any_minor_or_patch() {
        let pin = VersionPin::parse("1").unwrap();
        assert_eq!(pin.check("1.19.3").unwrap(), VersionCheck::Match);
        assert_eq!(pin.check("1.0.0").unwrap(), VersionCheck::Match);
        assert_eq!(pin.check("1.99.99").unwrap(), VersionCheck::Match);
    }

    #[test]
    fn major_pin_fails_on_major_drift() {
        let pin = VersionPin::parse("1").unwrap();
        assert_eq!(pin.check("2.0.0").unwrap(), VersionCheck::MajorDrift);
        assert_eq!(pin.check("0.9.0").unwrap(), VersionCheck::MajorDrift);
    }

    #[test]
    fn minor_pin_matches_any_patch() {
        let pin = VersionPin::parse("1.19").unwrap();
        assert_eq!(pin.check("1.19.0").unwrap(), VersionCheck::Match);
        assert_eq!(pin.check("1.19.9").unwrap(), VersionCheck::Match);
    }

    #[test]
    fn minor_pin_drifts_on_different_minor() {
        let pin = VersionPin::parse("1.19").unwrap();
        assert_eq!(pin.check("1.20.0").unwrap(), VersionCheck::MinorDrift);
        assert_eq!(pin.check("1.18.0").unwrap(), VersionCheck::MinorDrift);
    }

    #[test]
    fn minor_pin_fails_on_major_drift() {
        let pin = VersionPin::parse("1.19").unwrap();
        assert_eq!(pin.check("2.19.0").unwrap(), VersionCheck::MajorDrift);
    }

    #[test]
    fn exact_pin_matches_exactly() {
        let pin = VersionPin::parse("1.19.3").unwrap();
        assert_eq!(pin.check("1.19.3").unwrap(), VersionCheck::Match);
    }

    #[test]
    fn exact_pin_drifts_on_patch() {
        let pin = VersionPin::parse("1.19.3").unwrap();
        assert_eq!(pin.check("1.19.4").unwrap(), VersionCheck::PatchDrift);
        assert_eq!(pin.check("1.19.2").unwrap(), VersionCheck::PatchDrift);
    }

    #[test]
    fn exact_pin_minor_drift_beats_patch_drift() {
        let pin = VersionPin::parse("1.19.3").unwrap();
        assert_eq!(pin.check("1.20.3").unwrap(), VersionCheck::MinorDrift);
    }

    #[test]
    fn major_drift_always_wins() {
        let pin = VersionPin::parse("1.19.3").unwrap();
        assert_eq!(pin.check("2.0.0").unwrap(), VersionCheck::MajorDrift);
    }

    #[test]
    fn installed_version_prerelease_is_ignored() {
        let pin = VersionPin::parse("1.19").unwrap();
        assert_eq!(pin.check("1.19.0-rc1").unwrap(), VersionCheck::Match);
        assert_eq!(pin.check("1.19.0+build.42").unwrap(), VersionCheck::Match);
    }
}
