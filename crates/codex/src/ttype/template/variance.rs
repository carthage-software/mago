use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub enum Variance {
    Invariant,
    Covariant,
    Contravariant,
}

impl Variance {
    #[inline]
    pub const fn is_invariant(&self) -> bool {
        matches!(self, Variance::Invariant)
    }

    #[inline]
    pub const fn is_covariant(&self) -> bool {
        matches!(self, Variance::Covariant)
    }

    #[inline]
    pub const fn is_contravariant(&self) -> bool {
        matches!(self, Variance::Contravariant)
    }

    #[inline]
    pub const fn is_readonly(&self) -> bool {
        matches!(self, Variance::Covariant | Variance::Invariant)
    }

    /// Combines an outer variance context with an inner variance context.
    ///
    /// This is used when resolving nested templates, e.g., `Outer<Inner<T>>`.
    /// The variance of `T` relative to the outermost context depends on both
    /// the variance of `T` within `Inner` and the variance of `Inner` within `Outer`.
    ///
    /// Rules:
    ///
    /// - Anything combined with Invariant results in Invariant.
    /// - Covariant + Covariant = Covariant
    /// - Contravariant + Contravariant = Covariant
    /// - Covariant + Contravariant = Contravariant
    /// - Contravariant + Covariant = Contravariant
    #[inline]
    pub const fn combine(outer_variance: Self, inner_variance: Self) -> Self {
        match (outer_variance, inner_variance) {
            // If either is invariant, the result is invariant
            (Variance::Invariant, _) | (_, Variance::Invariant) => Variance::Invariant,
            // Co + Co = Co
            (Variance::Covariant, Variance::Covariant) => Variance::Covariant,
            // Contra + Contra = Co (double negative flips back)
            (Variance::Contravariant, Variance::Contravariant) => Variance::Covariant,
            // Co + Contra = Contra
            (Variance::Covariant, Variance::Contravariant) => Variance::Contravariant,
            // Contra + Co = Contra
            (Variance::Contravariant, Variance::Covariant) => Variance::Contravariant,
        }
    }
}

impl std::fmt::Display for Variance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Variance::Invariant => write!(f, "invariant"),
            Variance::Covariant => write!(f, "covariant"),
            Variance::Contravariant => write!(f, "contravariant"),
        }
    }
}
