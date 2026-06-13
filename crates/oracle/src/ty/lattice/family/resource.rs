//! Resource family: `resource`, `open-resource`, `closed-resource`.
//!
//! `open` and `closed` both refine `resource`. They are not subtypes of
//! each other (an open resource is not a closed one, and vice versa).

use crate::ty::atom::Atom;
use crate::ty::atom::payload::resource::ResourceAtom;

/// `Open <: Resource` and `Closed <: Resource`. Reflexivity is the
/// dispatcher's job.
#[inline]
#[must_use]
pub const fn refines(input: Atom<'_>, container: Atom<'_>) -> bool {
    let (Atom::Resource(input_payload), Atom::Resource(container_payload)) = (input, container) else {
        return false;
    };

    matches!((input_payload, container_payload), (ResourceAtom::Open | ResourceAtom::Closed, ResourceAtom::Any))
}
