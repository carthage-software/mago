use std::fmt;
use std::str::FromStr;

use schemars::JsonSchema;

/// The oldest Twig 3 version supported by a project.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, JsonSchema)]
#[schemars(with = "String")]
pub struct TwigVersion(u16);

impl TwigVersion {
    /// A version newer than any Twig release, which every rule supports.
    pub const LATEST: Self = Self(u16::MAX);

    /// The Twig 3 release with the given minor version.
    #[must_use]
    pub const fn new(minor: u16) -> Self {
        Self(minor)
    }

    #[must_use]
    pub const fn supports(self, minor: u16) -> bool {
        self.0 >= minor
    }
}

impl fmt::Display for TwigVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "3.{}", self.0)
    }
}

impl FromStr for TwigVersion {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let Some(minor) = value.strip_prefix("3.") else {
            return Err("expected a Twig 3 version such as '3.30'");
        };

        if minor.is_empty() || !minor.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err("expected a Twig 3 version such as '3.30'");
        }

        minor.parse().map(Self).map_err(|_| "Twig minor version is too large")
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for TwigVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for TwigVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = <String as serde::Deserialize>::deserialize(deserializer)?;
        value.parse().map_err(serde::de::Error::custom)
    }
}
