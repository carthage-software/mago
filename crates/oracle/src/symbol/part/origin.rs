#[cfg(feature = "serde")]
use serde::Serialize;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Origin {
    /// Third-party code pulled in as a dependency (composer `vendor/`, etc.).
    /// Known so symbols resolve, but not code you own. least authoritative,
    /// and normally not linted.
    Dependency,
    /// Provided by the PHP runtime itself: language built-ins and the symbols
    /// of loaded extensions. There is no source body to analyze.
    Runtime,
    /// A hand-written stub that deliberately replaces a `Runtime` or
    /// `Dependency` declaration. your correction to someone else's symbol.
    Override,
    /// First-party code: the project under analysis. Most authoritative,
    /// wins every collision, and the only origin issues are reported against.
    Project,
}

impl Origin {
    #[inline]
    #[must_use]
    pub const fn is_dependency(&self) -> bool {
        matches!(self, Self::Dependency)
    }

    #[inline]
    #[must_use]
    pub const fn is_runtime(&self) -> bool {
        matches!(self, Self::Runtime)
    }

    #[inline]
    #[must_use]
    pub const fn is_override(&self) -> bool {
        matches!(self, Self::Override)
    }

    #[inline]
    #[must_use]
    pub const fn is_project(&self) -> bool {
        matches!(self, Self::Project)
    }
}
