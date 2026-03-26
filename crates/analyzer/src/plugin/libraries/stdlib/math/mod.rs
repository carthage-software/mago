mod abs;
mod max;
mod min;

pub use abs::AbsProvider;
pub use max::MaxProvider;
pub use min::MinProvider;

use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::int::TInteger;
use mago_codex::ttype::union::TUnion;

/// Extract a single `TInteger` from a union type, if the type is a single integer atomic.
fn get_integer_from_type(ty: &TUnion) -> Option<TInteger> {
    if !ty.is_single() {
        return None;
    }

    match ty.get_single() {
        TAtomic::Scalar(TScalar::Integer(integer)) => Some(*integer),
        _ => None,
    }
}

/// Collect all integer atomics from a union type. Returns `None` if any atomic is not an integer.
fn collect_integers(ty: &TUnion) -> Option<Vec<TInteger>> {
    let mut integers = Vec::new();
    for atomic in ty.types.iter() {
        match atomic {
            TAtomic::Scalar(TScalar::Integer(integer)) => integers.push(*integer),
            _ => return None,
        }
    }

    Some(integers)
}
