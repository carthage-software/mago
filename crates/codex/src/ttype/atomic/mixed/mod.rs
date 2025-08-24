use serde::Deserialize;
use serde::Serialize;

use mago_atom::Atom;
use mago_atom::atom;

use crate::ttype::TType;
use crate::ttype::atomic::mixed::truthiness::TMixedTruthiness;

pub mod truthiness;

/// Represents the `mixed` type, potentially with constraints applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct TMixed {
    is_isset_from_loop: bool,
    is_non_null: bool,
    is_empty: bool,
    truthiness: TMixedTruthiness,
}

impl TMixed {
    /// Creates a `Mixed` type representing a `mixed` with no specific constraints known yet.
    ///
    /// Equivalent to `Mixed::default()`.
    #[inline]
    pub const fn new() -> Self {
        Self {
            is_isset_from_loop: false,
            is_non_null: false,
            is_empty: false,
            truthiness: TMixedTruthiness::Undetermined,
        }
    }

    /// Creates a `Mixed` type constrained to be non-null.
    #[inline]
    pub const fn non_null() -> Self {
        Self {
            is_isset_from_loop: false,
            is_non_null: true,
            is_empty: false,
            truthiness: TMixedTruthiness::Undetermined,
        }
    }

    /// Creates a `Mixed` type marked as originating from `isset()` in a loop.
    #[inline]
    pub const fn isset_from_loop() -> Self {
        Self {
            is_isset_from_loop: true, // Mark origin
            is_non_null: false,
            is_empty: false,
            truthiness: TMixedTruthiness::Undetermined,
        }
    }

    /// Creates a `Mixed` type that may be marked as originating from `isset()` in a loop.
    #[inline]
    pub const fn maybe_isset_from_loop(from_loop: bool) -> Self {
        Self {
            is_isset_from_loop: from_loop,
            is_non_null: false,
            is_empty: false,
            truthiness: TMixedTruthiness::Undetermined,
        }
    }

    /// Creates a `Mixed` type constrained to be truthy. Automatically sets `is_non_null` to `true`.
    #[inline]
    pub const fn truthy() -> Self {
        Self { is_isset_from_loop: false, is_non_null: true, is_empty: false, truthiness: TMixedTruthiness::Truthy }
    }

    /// Creates a `Mixed` type constrained to be falsy. May include null.
    #[inline]
    pub const fn falsy() -> Self {
        Self { is_isset_from_loop: false, is_non_null: false, is_empty: false, truthiness: TMixedTruthiness::Falsy }
    }

    /// Checks if this `mixed` type could be truthy or non-null.
    #[inline]
    pub const fn could_be_truthy_or_non_null(&self) -> bool {
        self.is_vanilla() || self.is_non_null()
    }

    /// Checks if this `mixed` originated from `isset()` in a loop.
    #[inline]
    pub const fn is_isset_from_loop(&self) -> bool {
        self.is_isset_from_loop
    }

    /// Checks if this `mixed` type is a vanilla `mixed` type.
    #[inline]
    pub const fn is_vanilla(&self) -> bool {
        !self.is_non_null && !self.is_empty && matches!(self.truthiness, TMixedTruthiness::Undetermined)
    }

    /// Checks if `null` is explicitly excluded from this `mixed` type.
    #[inline]
    pub const fn is_non_null(&self) -> bool {
        self.is_non_null
    }

    /// Returns the known truthiness constraint for this `mixed` type.
    #[inline]
    pub const fn get_truthiness(&self) -> TMixedTruthiness {
        self.truthiness
    }

    /// Checks if the type is constrained to only truthy values.
    #[inline]
    pub const fn is_truthy(&self) -> bool {
        matches!(self.truthiness, TMixedTruthiness::Truthy)
    }

    /// Checks if the type is constrained to only falsy values.
    #[inline]
    pub const fn is_falsy(&self) -> bool {
        matches!(self.truthiness, TMixedTruthiness::Falsy)
    }

    /// Checks if the truthiness constraint is undetermined.
    #[inline]
    pub const fn is_truthiness_undetermined(&self) -> bool {
        matches!(self.truthiness, TMixedTruthiness::Undetermined)
    }

    /// Returns a new instance with the `is_isset_from_loop` flag set.
    #[inline]
    pub const fn with_is_isset_from_loop(mut self, is_isset_from_loop: bool) -> Self {
        self.is_isset_from_loop = is_isset_from_loop;
        self
    }

    /// Returns a new instance with the `is_non_null` flag set and consistency ensured.
    #[inline]
    pub const fn with_is_non_null(mut self, is_non_null: bool) -> Self {
        self.is_non_null = is_non_null;
        self
    }

    /// Returns a new instance with the `truthiness` value set. Ensures consistency with `is_non_null`.
    #[inline]
    pub const fn with_truthiness(mut self, truthiness: TMixedTruthiness) -> Self {
        self.truthiness = truthiness;
        self.ensure_consistency();
        self
    }

    pub const fn as_empty(mut self) -> Self {
        self.is_empty = true;
        self.ensure_consistency();

        self
    }

    /// Ensures consistency between `is_non_null` and `truthiness`.
    #[inline]
    const fn ensure_consistency(&mut self) {
        if self.is_truthy() {
            self.is_non_null = true;
        }

        if self.is_empty {
            self.is_non_null = false;
            self.truthiness = TMixedTruthiness::Falsy;
        }
    }
}

impl TType for TMixed {
    fn needs_population(&self) -> bool {
        false
    }

    fn is_expandable(&self) -> bool {
        false
    }

    fn get_id(&self) -> Atom {
        if self.is_empty {
            atom(match self.truthiness {
                TMixedTruthiness::Truthy => "empty-truthy-mixed",
                TMixedTruthiness::Falsy => "empty-falsy-mixed",
                TMixedTruthiness::Undetermined if self.is_non_null => "empty-nonnull",
                TMixedTruthiness::Undetermined => "empty-mixed",
            })
        } else {
            atom(match self.truthiness {
                TMixedTruthiness::Truthy => "truthy-mixed",
                TMixedTruthiness::Falsy => "falsy-mixed",
                TMixedTruthiness::Undetermined if self.is_non_null => "nonnull",
                TMixedTruthiness::Undetermined => "mixed",
            })
        }
    }
}

impl Default for TMixed {
    fn default() -> Self {
        Self {
            is_isset_from_loop: false,
            is_non_null: false,
            is_empty: false,
            truthiness: TMixedTruthiness::Undetermined,
        }
    }
}
