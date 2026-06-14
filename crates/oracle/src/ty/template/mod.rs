//! Template parameter inference: shared identity and bound types.
//!
//! The standin walk (bound collection), reconciliation (bound resolution),
//! and substitution (parameter replacement) algorithms build on these
//! types. The lattice also records [`Bound`]s surfaced during comparisons
//! into its report.

use mago_span::Span;

use crate::name::Name;
use crate::ty::Type;
use crate::ty::atom::payload::generic_parameter::DefiningEntity;
use crate::world::Variance;

pub use self::reconcile::reconcile;
pub use self::standin::GenericTemplate;
pub use self::standin::TemplateResult;
pub use self::standin::TemplateState;
pub use self::standin::standin;
pub use self::substitute::substitute;

mod reconcile;
mod standin;
mod substitute;

/// Identity of a template parameter inside the inference environment. Two
/// parameters with the same surface name in different defining entities
/// (e.g. two unrelated `T`s in two classes) are distinct keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TemplateKey<'arena> {
    pub defining_entity: DefiningEntity<'arena>,
    pub name: Name<'arena>,
}

/// What kind of constraint a [`Bound`] places on its template parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BoundKind {
    /// `T ≽ τ` - `T` must be a supertype of `τ`. Collected at covariant
    /// positions.
    Lower,
    /// `T ≼ τ` - `T` must be a subtype of `τ`. Collected at contravariant
    /// positions.
    Upper,
    /// `T = τ` - collected at invariant positions; equivalent to a Lower
    /// and Upper bound at the same time.
    Equality,
}

/// One bound entry recorded for a template parameter during the standin
/// walk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Bound<'arena> {
    pub kind: BoundKind,
    pub ty: Type<'arena>,
    /// Call-site argument index.
    pub argument_offset: u32,
    /// Structural depth at which the bound was collected. The top of the
    /// parameter type is depth `0`; each descent into a generic-parameter
    /// application increments by one.
    pub depth: u32,
    /// For [`BoundKind::Equality`] bounds, the class whose template
    /// parameter declaration introduced the equality (the class whose
    /// type-argument position is invariant). `None` for non-equality
    /// bounds and for equality bounds collected outside any class context.
    pub equality_bound_classlike: Option<Name<'arena>>,
    /// Source location of the binding site. `None` when the caller did not
    /// supply one; span propagation is opt-in via
    /// [`StandinOptions::span`].
    pub span: Option<Span>,
}

/// Caller-controlled options for the standin walk.
#[derive(Debug, Clone, Copy)]
pub struct StandinOptions {
    /// The call-site argument index this walk corresponds to. Used to tag
    /// bounds so reconciliation can group them per-position.
    pub argument_offset: u32,
    /// Variance assumed for the top-level walk. Defaults to `Invariant` -
    /// the soundest choice when no surrounding container declares a
    /// position-specific variance.
    pub default_variance: Variance,
    /// Maximum structural descent depth. Walks past this depth replace the
    /// parameter slot with its constraint (no further bound is recorded).
    /// Defaults to `8`, which is enough for realistic PHP generics while
    /// bounding cost on cycles in template constraints.
    pub max_depth: u32,
    /// Source location of the call-site argument the walk is operating on.
    /// Stamped onto every recorded [`Bound`].
    pub span: Option<Span>,
}

impl Default for StandinOptions {
    #[inline]
    fn default() -> Self {
        Self { argument_offset: 0, default_variance: Variance::Invariant, max_depth: 8, span: None }
    }
}
