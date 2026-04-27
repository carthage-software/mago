mod abs;
mod intdiv;
mod max;
mod min;

pub use abs::AbsProvider;
pub use intdiv::IntdivHook;
pub use max::MaxProvider;
pub use min::MinProvider;

use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::int::TInteger;
use mago_codex::ttype::union::TUnion;

/// Extract a `TInteger` from a union type by computing the combined bounds of all integer atomics.
/// Returns `None` if any atomic in the union is not an integer.
fn get_integer_from_type(ty: &TUnion) -> Option<TInteger> {
    let mut result_lb: Option<Option<i64>> = None;
    let mut result_ub: Option<Option<i64>> = None;

    for atomic in ty.types.iter() {
        let TAtomic::Scalar(TScalar::Integer(integer)) = atomic else {
            return None;
        };

        let (lb, ub) = integer.get_bounds();

        result_lb = Some(match result_lb {
            None => lb,
            Some(prev) => match (prev, lb) {
                (Some(a), Some(b)) => Some(std::cmp::min(a, b)),
                _ => None,
            },
        });

        result_ub = Some(match result_ub {
            None => ub,
            Some(prev) => match (prev, ub) {
                (Some(a), Some(b)) => Some(std::cmp::max(a, b)),
                _ => None,
            },
        });
    }

    Some(TInteger::from_bounds(result_lb?, result_ub?))
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
