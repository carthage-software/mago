//! Bound reconciliation: pick the relevant bounds for a template
//! parameter from a [`TemplateState`] and union them into the
//! parameter's *witness*: the type the parameter resolves to in the
//! current call context.
//!
//! # Algorithm
//!
//! Given the bounds collected for a single template parameter, sorted
//! by ascending appearance depth (`d`):
//!
//! 1. The *baseline depth* `d_0` is the depth of the shallowest bound.
//! 2. Bounds at depth `d_0` are always relevant.
//! 3. A deeper bound (`d > d_0`) is included **only** when:
//!    - some bound seen so far carried the invariant marker
//!      (`equality_bound_classlike`, set when the bound was reached
//!      through an invariant generic position anywhere above it in the
//!      structural walk), **and**
//!    - the deeper bound's argument offset matches the baseline's
//!      offset.
//!
//! The relevant bounds' types are then unioned through the builder.
//! When no bounds were collected, the materialisation falls back to
//! the parameter's constraint (`κ(T)`).
//!
//! The invariant marker is the standin walk's `equality_bound_classlike`.
//! The walk's variance composition propagates invariance downward: once an
//! invariant position is crossed, every nested position is invariant too,
//! so a bound recorded at a covariant inner slot under an invariant outer
//! one still carries the marker.

use mago_allocator::Arena;

use crate::ty::Type;
use crate::ty::builder::TypeBuilder;
use crate::ty::template::Bound;
use crate::ty::template::TemplateKey;
use crate::ty::template::standin::TemplateState;

/// Run depth-based selection on a list of bounds and return the
/// unioned witness type. Returns `None` when the bound list is
/// empty so the caller can fall back to the parameter's constraint.
///
/// `bounds` is taken as a slice; ordering doesn't matter; this
/// function sorts internally.
#[inline]
#[must_use]
pub fn reconcile<'arena, S, A>(
    bounds: &[Bound<'arena>],
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Option<Type<'arena>>
where
    S: Arena,
    A: Arena,
{
    if bounds.is_empty() {
        return None;
    }

    let mut sorted: Vec<Bound<'arena>> = bounds.to_vec();
    sorted.sort_by_key(|bound| bound.depth);

    let first = sorted.first()?;
    let baseline_depth = first.depth;
    let baseline_offset = first.argument_offset;

    let mut seen_invariant = false;
    let mut relevant: Vec<Type<'arena>> = Vec::new();

    for bound in sorted {
        if bound.depth == baseline_depth {
            if bound.equality_bound_classlike.is_some() {
                seen_invariant = true;
            }
            relevant.push(bound.ty);
            continue;
        }

        if seen_invariant && bound.argument_offset == baseline_offset {
            relevant.push(bound.ty);
        }
    }

    let mut atoms = Vec::new();
    for ty in relevant {
        atoms.extend_from_slice(ty.atoms);
    }

    Some(builder.union_of(&atoms))
}

impl<'arena> TemplateState<'arena> {
    /// Materialise `key`'s witness from its collected bounds. Returns
    /// `fallback` (typically the parameter's constraint or `mixed`)
    /// when no bound was recorded.
    #[inline]
    #[must_use]
    pub fn witness<S, A>(
        &self,
        key: TemplateKey<'arena>,
        fallback: Type<'arena>,
        builder: &mut TypeBuilder<'_, 'arena, S, A>,
    ) -> Type<'arena>
    where
        S: Arena,
        A: Arena,
    {
        reconcile(self.bounds_for(key), builder).unwrap_or(fallback)
    }
}
