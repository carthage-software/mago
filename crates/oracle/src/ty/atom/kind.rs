#[cfg(feature = "serde")]
use serde::Serialize;

/// The top-level dispatch tag of a single [`Atom`](crate::Atom).
///
/// Discriminants are pinned: they define the canonical ordering of atoms
/// within a union, the bit positions of [`AtomKindSet`](crate::AtomKindSet),
/// and the stable rendering order.
#[repr(u8)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AtomKind {
    Null = 1,
    Never,
    Void,
    Placeholder,
    Mixed,
    Bool,
    True,
    False,
    Int,
    Float,
    String,
    ClassLikeString,
    Scalar,
    Numeric,
    ArrayKey,
    Object,
    Enum,
    ObjectShape,
    HasMethod,
    HasProperty,
    Array,
    List,
    Iterable,
    Callable,
    Resource,
    GenericParameter,
    Variable,
    Reference,
    MemberReference,
    GlobalReference,
    Alias,
    Conditional,
    Derived,
    ObjectAny,
    Negated,
    Intersected,
}

impl AtomKind {
    /// `true` when the kind is fully described by its tag: no payload, one
    /// canonical instance.
    #[inline]
    #[must_use]
    pub const fn is_trivial(self) -> bool {
        matches!(
            self,
            Self::Null
                | Self::Never
                | Self::Void
                | Self::Placeholder
                | Self::Bool
                | Self::True
                | Self::False
                | Self::Scalar
                | Self::Numeric
                | Self::ArrayKey
                | Self::ObjectAny
        )
    }

    #[inline]
    #[must_use]
    pub const fn discriminant(self) -> u8 {
        self as u8
    }
}
