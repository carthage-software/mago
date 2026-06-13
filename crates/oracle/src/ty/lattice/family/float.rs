//! Float family: `float`, `literal-float`, float literals.
//!
//! `Float` accepts any Float-kind input; `UnspecifiedLiteral` accepts
//! `Literal(_)` and itself; concrete literals only fit themselves
//! (reflexivity handles that).
//!
//! `int` and `float` are disjoint at the value-set level: the runtime types
//! are distinct, and `is_float($x)` is `false` for an int. PHP's implicit
//! int→float coercion at parameter binding is a callsite convenience, not a
//! subtype relation, so it is intentionally not modeled here. Use a
//! separate "assignable" predicate if a downstream consumer needs the
//! coercion view.

use crate::ty::atom::Atom;
use crate::ty::atom::payload::scalar::float::FloatAtom;

#[inline]
#[must_use]
pub fn refines(input: Atom<'_>, container: Atom<'_>) -> bool {
    let (Atom::Float(input_payload), Atom::Float(container_payload)) = (input, container) else {
        return false;
    };

    matches!(
        (input_payload, container_payload),
        (_, FloatAtom::Unspecified)
            | (FloatAtom::Literal(_) | FloatAtom::UnspecifiedLiteral, FloatAtom::UnspecifiedLiteral),
    )
}
