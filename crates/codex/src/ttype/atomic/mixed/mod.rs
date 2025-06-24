use serde::Deserialize;
use serde::Serialize;

use crate::ttype::TType;
use crate::ttype::atomic::mixed::truthiness::TMixedTruthiness;

pub mod truthiness;

/// Represents the `mixed` type, potentially with constraints applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct TMixed {
    is_isset_from_loop: bool,
    is_any: bool,
    is_non_null: bool,
    truthiness: TMixedTruthiness,
}

impl TMixed {
    /// Creates a `Mixed` type representing a vanilla `mixed` with no specific constraints known yet.
    ///
    /// Equivalent to `Mixed::default()`.
    #[inline]
    pub const fn vanilla() -> Self {
        Self {
            is_isset_from_loop: false,
            is_any: false,
            is_non_null: false,
            truthiness: TMixedTruthiness::Undetermined,
        }
    }

    /// Creates a `Mixed` type explicitly marked as the "any" type (most general form).
    #[inline]
    pub const fn any() -> Self {
        Self {
            is_isset_from_loop: false,
            is_any: true, // Mark as 'any'
            is_non_null: false,
            truthiness: TMixedTruthiness::Undetermined,
        }
    }

    /// Creates a `Mixed` type constrained to be non-null.
    #[inline]
    pub const fn non_null() -> Self {
        Self { is_isset_from_loop: false, is_any: false, is_non_null: true, truthiness: TMixedTruthiness::Undetermined }
    }

    /// Creates a `Mixed` type marked as originating from `isset()` in a loop.
    #[inline]
    pub const fn isset_from_loop() -> Self {
        Self {
            is_isset_from_loop: true, // Mark origin
            is_any: false,
            is_non_null: false,
            truthiness: TMixedTruthiness::Undetermined,
        }
    }

    /// Creates a `Mixed` type that may be marked as originating from `isset()` in a loop.
    #[inline]
    pub const fn maybe_isset_from_loop(from_loop: bool) -> Self {
        Self {
            is_isset_from_loop: from_loop,
            is_any: false,
            is_non_null: false,
            truthiness: TMixedTruthiness::Undetermined,
        }
    }

    /// Creates a `Mixed` type constrained to be truthy. Automatically sets `is_non_null` to `true`.
    #[inline]
    pub const fn truthy() -> Self {
        Self {
            is_isset_from_loop: false,
            is_any: false,
            is_non_null: true, // Truthy implies non-null
            truthiness: TMixedTruthiness::Truthy,
        }
    }

    /// Creates a `Mixed` type constrained to be falsy. May include null.
    #[inline]
    pub const fn falsy() -> Self {
        Self {
            is_isset_from_loop: false,
            is_any: false,
            is_non_null: false, // Falsy *can* be null
            truthiness: TMixedTruthiness::Falsy,
        }
    }

    /// Checks if this `mixed` type could be truthy or non-null.
    #[inline]
    pub const fn could_be_truthy_or_non_null(&self) -> bool {
        self.is_vanilla() || self.is_any() || self.is_non_null()
    }

    /// Checks if this `mixed` originated from `isset()` in a loop.
    #[inline]
    pub const fn is_isset_from_loop(&self) -> bool {
        self.is_isset_from_loop
    }

    /// Checks if this `mixed` type is a vanilla `mixed` type.
    #[inline]
    pub const fn is_vanilla(&self) -> bool {
        !self.is_any && !self.is_non_null && matches!(self.truthiness, TMixedTruthiness::Undetermined)
    }

    /// Checks if this represents the most general "any" mixed type.
    #[inline]
    pub const fn is_any(&self) -> bool {
        self.is_any
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

    /// Returns a new instance with the `is_any` flag set.
    #[inline]
    pub const fn with_is_any(mut self, is_any: bool) -> Self {
        self.is_any = is_any;
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

    /// Ensures consistency between `is_non_null` and `truthiness`.
    #[inline]
    const fn ensure_consistency(&mut self) {
        if self.is_truthy() {
            self.is_non_null = true; // Truthy always implies non-null
        }
    }
}

impl TType for TMixed {
    fn get_id(&self, _interner: Option<&mago_interner::ThreadedInterner>) -> String {
        if self.is_any {
            match self.truthiness {
                TMixedTruthiness::Truthy => "truthy-from-any".to_string(),
                TMixedTruthiness::Falsy => "falsy-from-any".to_string(),
                TMixedTruthiness::Undetermined => {
                    if self.is_non_null {
                        "nonnull-from-any".to_string()
                    } else {
                        "any".to_string()
                    }
                }
            }
        } else {
            match self.truthiness {
                TMixedTruthiness::Truthy => "truthy-mixed".to_string(),
                TMixedTruthiness::Falsy => "falsy-mixed".to_string(),
                TMixedTruthiness::Undetermined => {
                    if self.is_non_null {
                        "nonnull".to_string()
                    } else {
                        "mixed".to_string()
                    }
                }
            }
        }
    }
}

impl Default for TMixed {
    fn default() -> Self {
        Self {
            is_isset_from_loop: false,
            is_any: false,
            is_non_null: false,
            truthiness: TMixedTruthiness::Undetermined,
        }
    }
}
