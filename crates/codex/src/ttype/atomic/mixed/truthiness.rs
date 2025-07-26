use serde::Deserialize;
use serde::Serialize;

/// Describes the known truthiness of a `Mixed` type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord, Default)]
#[repr(u8)]
pub enum TMixedTruthiness {
    /// The value could be truthy or falsy.
    #[default]
    Undetermined,
    /// The value is definitely truthy (evaluates to `true`). Excludes null, false, 0, "", [], etc.
    Truthy,
    /// The value is definitely falsy (evaluates to `false`). Includes null, false, 0, "", [], etc.
    Falsy,
}

impl TMixedTruthiness {
    /// Returns true if the value is undetermined (could be truthy or falsy).
    #[inline]
    pub const fn is_undetermined(&self) -> bool {
        matches!(self, TMixedTruthiness::Undetermined)
    }

    /// Returns true if the value is definitely truthy.
    #[inline]
    pub const fn is_truthy(&self) -> bool {
        matches!(self, TMixedTruthiness::Truthy)
    }

    /// Returns true if the value is definitely falsy.
    #[inline]
    pub const fn is_falsy(&self) -> bool {
        matches!(self, TMixedTruthiness::Falsy)
    }
}
