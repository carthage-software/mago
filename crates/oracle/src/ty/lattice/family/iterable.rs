//! Iterable family: `iterable<K, V>` and intersected forms.
//!
//! `iterable` accepts:
//!
//! - other `iterable<K', V'>` when `K' <: K` and `V' <: V` (key + value
//!   covariance ; iterables are read-only at the type level so values are
//!   covariant; PHP doesn't model write positions on `iterable`)
//! - `list<E>` when `E <: V` and `array-key <: K` (lists key by `int <:
//!   array-key`)
//! - keyed arrays when their key/value parameters refine the container's
//! - empty array when the container's value side accepts nothing extra
//!   (vacuously true: empty has no entries)
//!
//! `iterable` does NOT accept generic `\Traversable` named-objects yet
//! because that requires symbol-table-driven hierarchy queries.

use mago_allocator::Arena;

use crate::symbol::SymbolTable;
use crate::ty::atom::Atom;
use crate::ty::builder::TypeBuilder;
use crate::ty::lattice::LatticeOptions;
use crate::ty::lattice::LatticeReport;
use crate::ty::lattice::refines as type_refines;
use crate::ty::well_known;

#[inline]
pub fn refines<'arena, S, A>(
    input: Atom<'arena>,
    container: Atom<'arena>,
    symbols: &SymbolTable<'arena, A>,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    let Atom::Iterable(container_payload) = container else {
        return false;
    };

    if input == well_known::EMPTY_ARRAY {
        return true;
    }

    match input {
        Atom::Iterable(input_payload) => {
            type_refines(input_payload.key_type, container_payload.key_type, symbols, options, report, builder)
                && type_refines(
                    input_payload.value_type,
                    container_payload.value_type,
                    symbols,
                    options,
                    report,
                    builder,
                )
        }
        Atom::List(input_payload) => {
            type_refines(well_known::TYPE_INT, container_payload.key_type, symbols, options, report, builder)
                && type_refines(
                    input_payload.element_type,
                    container_payload.value_type,
                    symbols,
                    options,
                    report,
                    builder,
                )
        }
        Atom::Array(input_payload) => {
            let key = input_payload.key_param.unwrap_or(well_known::TYPE_ARRAY_KEY);
            let value = input_payload.value_param.unwrap_or(container_payload.value_type);

            type_refines(key, container_payload.key_type, symbols, options, report, builder)
                && type_refines(value, container_payload.value_type, symbols, options, report, builder)
        }
        _ => false,
    }
}
