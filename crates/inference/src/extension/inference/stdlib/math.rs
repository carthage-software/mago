use mago_allocator::Arena;
use mago_allocator::vec::Vec;
use mago_hir::ir::expression::Call;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::atom::payload::scalar::float::FloatAtom;
use mago_oracle::ty::atom::payload::scalar::int::IntAtom;
use mago_oracle::ty::well_known::FLOAT;

use crate::extension::ExtensionContext;
use crate::extension::inference::stdlib::literal_int;
use crate::extension::inference::stdlib::nth_argument;
use crate::extension::inference::stdlib::positional_arguments;
use crate::flow::Flow;

pub(super) fn fold_abs<'arena, A: Arena>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    argument: Option<Type<'arena>>,
) -> Option<Type<'arena>> {
    match argument?.atoms {
        [Atom::Int(integer)] => {
            let (lower, upper) = abs_bounds(int_bounds(integer));
            match (lower, upper) {
                (Some(low), Some(high)) if low == high => Some(context.int(low)),
                (low, high) => Some(context.int_range(low, high)),
            }
        }
        [Atom::Float(FloatAtom::Literal(value))] => Some(context.float(value.0.into_inner().abs())),
        [Atom::Float(_)] => Some(context.union(&[FLOAT])),
        _ => None,
    }
}

fn abs_bounds((lower, upper): (Option<i64>, Option<i64>)) -> (Option<i64>, Option<i64>) {
    match (lower, upper) {
        (Some(low), _) if low >= 0 => (lower, upper),
        (_, Some(high)) if high <= 0 => (high.checked_neg(), lower.and_then(i64::checked_neg)),
        (Some(low), Some(high)) => (Some(0), Some(low.checked_neg().unwrap_or(i64::MAX).max(high))),
        _ => (Some(0), None),
    }
}

pub(super) fn int_bounds(integer: &IntAtom<'_>) -> (Option<i64>, Option<i64>) {
    match integer {
        IntAtom::Literal(value) => (Some(*value), Some(*value)),
        IntAtom::Range(range) => (range.lower(), range.upper()),
        _ => (None, None),
    }
}

pub(super) fn fold_intdiv<'arena, A: Arena>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    call: &Call<'arena, SymbolId, Flow, Type<'arena>>,
) -> Option<Type<'arena>> {
    let dividend = nth_argument(call, 0).and_then(literal_int)?;
    let divisor = nth_argument(call, 1).and_then(literal_int)?;

    dividend.checked_div(divisor).map(|quotient| context.int(quotient))
}

pub(super) fn fold_min_max<'arena, A: Arena>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    call: &Call<'arena, SymbolId, Flow, Type<'arena>>,
    max: bool,
) -> Option<Type<'arena>> {
    let atoms = candidate_atoms(context.arena(), call)?;
    if atoms.is_empty() {
        return None;
    }

    match reduce_int_bounds(&atoms, max) {
        Some((lower, upper)) => Some(int_type(context, lower, upper)),
        None => Some(context.union(&atoms)),
    }
}

fn candidate_atoms<'arena, A: Arena>(
    arena: &'arena A,
    call: &Call<'arena, SymbolId, Flow, Type<'arena>>,
) -> Option<Vec<'arena, Atom<'arena>, A>> {
    let arguments = positional_arguments(arena, call);
    match arguments.as_slice() {
        [] => None,
        [single] => array_value_atoms(arena, *single),
        many => {
            let mut atoms = Vec::new_in(arena);
            for ty in many {
                atoms.extend_from_slice(ty.atoms);
            }

            Some(atoms)
        }
    }
}

fn array_value_atoms<'arena, A: Arena>(
    arena: &'arena A,
    ty: Type<'arena>,
) -> Option<Vec<'arena, Atom<'arena>, A>> {
    let mut atoms = Vec::new_in(arena);
    match ty.atoms {
        [Atom::Array(array)] => {
            if let Some(items) = array.known_items {
                for item in items {
                    atoms.extend_from_slice(item.value.atoms);
                }
            }
            if let Some(value) = array.value_param {
                atoms.extend_from_slice(value.atoms);
            }
        }
        [Atom::List(list)] => {
            if let Some(elements) = list.known_elements {
                for element in elements {
                    atoms.extend_from_slice(element.value.atoms);
                }
            }
            if !list.element_type.is_never() {
                atoms.extend_from_slice(list.element_type.atoms);
            }
        }
        _ => return None,
    }

    Some(atoms)
}

fn reduce_int_bounds(atoms: &[Atom<'_>], max: bool) -> Option<(Option<i64>, Option<i64>)> {
    let mut iterator = atoms.iter();
    let (mut lower, mut upper) = atom_int_bounds(iterator.next()?)?;

    for atom in iterator {
        let (other_lower, other_upper) = atom_int_bounds(atom)?;
        if max {
            lower = pick_bound(lower, other_lower, i64::max, true);
            upper = pick_bound(upper, other_upper, i64::max, false);
        } else {
            lower = pick_bound(lower, other_lower, i64::min, false);
            upper = pick_bound(upper, other_upper, i64::min, true);
        }
    }

    Some((lower, upper))
}

fn pick_bound(current: Option<i64>, other: Option<i64>, combine: fn(i64, i64) -> i64, keep_known: bool) -> Option<i64> {
    match (current, other) {
        (Some(left), Some(right)) => Some(combine(left, right)),
        (left, right) if keep_known => left.or(right),
        _ => None,
    }
}

fn atom_int_bounds(atom: &Atom<'_>) -> Option<(Option<i64>, Option<i64>)> {
    match atom {
        Atom::Int(integer) => Some(int_bounds(integer)),
        _ => None,
    }
}

fn int_type<'arena, A: Arena>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    lower: Option<i64>,
    upper: Option<i64>,
) -> Type<'arena> {
    match (lower, upper) {
        (Some(low), Some(high)) if low == high => context.int(low),
        (low, high) => context.int_range(low, high),
    }
}
