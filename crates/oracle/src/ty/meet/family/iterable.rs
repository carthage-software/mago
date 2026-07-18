//! `Iterable` family meet: `iterable<K1, V1> ∧ iterable<K2, V2>` is
//! `iterable<K1 ∧ K2, V1 ∧ V2>` (both axes covariant).

use mago_allocator::Arena;

use crate::symbol::SymbolTable;
use crate::ty::atom::Atom;
use crate::ty::atom::payload::iterable::IterableAtom;
use crate::ty::builder::TypeBuilder;
use crate::ty::lattice::LatticeOptions;
use crate::ty::lattice::LatticeReport;

pub(in crate::ty::meet) fn iterable_meet<'arena, S, A>(
    a: Atom<'arena>,
    b: Atom<'arena>,
    symbols: &SymbolTable<'arena, A>,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Option<Atom<'arena>>
where
    S: Arena,
    A: Arena,
{
    let (Atom::Iterable(a_payload), Atom::Iterable(b_payload)) = (a, b) else {
        return None;
    };

    let key_type = crate::ty::meet::compute(a_payload.key_type, b_payload.key_type, symbols, options, report, builder);
    let value_type =
        crate::ty::meet::compute(a_payload.value_type, b_payload.value_type, symbols, options, report, builder);

    Some(builder.iterable(IterableAtom { key_type, value_type }))
}
