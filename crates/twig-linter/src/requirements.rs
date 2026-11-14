#[cfg(feature = "serde")]
use serde::Serializer;
#[cfg(feature = "serde")]
use serde::ser::SerializeStruct;

use crate::version::TwigVersion;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, Hash, PartialOrd)]
pub enum RuleRequirements {
    /// The rule is active if the project's minimum Twig version is at least this version.
    TwigVersion(TwigVersion),
    /// The rule is always active.
    None,
}

impl RuleRequirements {
    /// Checks if the configured minimum Twig version satisfies the rule's requirements.
    #[must_use]
    pub fn are_met_by(&self, configured_version: TwigVersion) -> bool {
        match self {
            Self::TwigVersion(version) => configured_version >= *version,
            Self::None => true,
        }
    }

    /// The minimum Twig version the rule needs, if any.
    #[must_use]
    pub fn minimum_twig_version(&self) -> Option<TwigVersion> {
        match self {
            Self::TwigVersion(version) => Some(*version),
            Self::None => None,
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for RuleRequirements {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("RuleRequirements", 1)?;
        state.serialize_field("twig-version", &self.minimum_twig_version())?;
        state.end()
    }
}
