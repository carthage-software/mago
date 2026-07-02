#[cfg(feature = "serde")]
use serde::Serialize;

/// Provenance and analysis-state bits attached to a [`Typed`](crate::Typed)
/// value, carried as a `U16Flags<FlowFlag>`.
///
/// Flow flags do not participate in the denotational meaning of a type: two
/// unions with identical atoms but different flags inhabit the same set of
/// values. They affect diagnostics, narrowing, and substitution.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u16)]
pub enum FlowFlag {
    HadTemplate = 1 << 0,
    FromTemplateDefault = 1 << 1,
    Populated = 1 << 2,
    PossiblyUndefined = 1 << 3,
    PossiblyUndefinedFromTry = 1 << 4,
    IgnoreNullableIssues = 1 << 5,
    IgnoreFalsableIssues = 1 << 6,
    NullsafeNull = 1 << 7,
    ByReference = 1 << 8,
    ReferenceFree = 1 << 9,
}

impl From<FlowFlag> for u16 {
    fn from(flag: FlowFlag) -> Self {
        flag as u16
    }
}
