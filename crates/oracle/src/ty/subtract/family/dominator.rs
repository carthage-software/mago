//! True-union dominator subtract: split `scalar` / `numeric` /
//! `array-key` into their constituents when the right-hand side
//! lands inside one of the sub-families.

use mago_allocator::Arena;
use mago_allocator::vec::Vec as ScratchVec;

use crate::ty::atom::Atom;
use crate::ty::atom::kind::AtomKind;
use crate::ty::builder::TypeBuilder;
use crate::ty::lattice::LatticeOptions;
use crate::ty::lattice::LatticeReport;
use crate::ty::well_known::ARRAY_KEY;
use crate::ty::well_known::BOOL;
use crate::ty::well_known::FLOAT;
use crate::ty::well_known::INT;
use crate::ty::well_known::NUMERIC;
use crate::ty::well_known::NUMERIC_STRING;
use crate::ty::well_known::SCALAR;
use crate::ty::well_known::STRING;
use crate::world::World;

/// Fan out a true-union dominator (`scalar`, `numeric`, `array-key`)
/// when the right-hand side is a member of one of its sub-families.
/// The dominator's value-set is the disjoint union of its members;
/// subtracting splits the dominator into its constituents and
/// delegates the in-family subtraction to the recursive call. The fan
/// out only fires when the removed side lands inside one of the
/// sub-families; otherwise the dominator's constituents would be
/// needlessly re-emitted for an unrelated subtraction (e.g.
/// `scalar \ Foo`).
///
/// `scalar = bool | int | float | string`,
/// `numeric = int | float | numeric-string`,
/// `array-key = int | string`.
pub(in crate::ty::subtract) fn true_union_minus<'scratch, 'arena, S, A, W>(
    input: Atom<'arena>,
    removed: Atom<'arena>,
    world: &W,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
    out: &mut ScratchVec<'scratch, Atom<'arena>, S>,
) -> bool
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    let members: &[Atom<'arena>] = if input == SCALAR {
        &[BOOL, INT, FLOAT, STRING]
    } else if input == NUMERIC {
        &[INT, FLOAT, NUMERIC_STRING]
    } else if input == ARRAY_KEY {
        &[INT, STRING]
    } else {
        return false;
    };

    if !members.iter().any(|member| dominator_member_covers(*member, removed)) {
        return false;
    }

    for &member in members {
        crate::ty::subtract::atom_minus(member, removed, world, options, report, builder, out);
    }

    true
}

/// `true` iff `member` and `removed` share at least one runtime axis,
/// so splitting the dominator into its members lets the per-member
/// subtract drop or narrow some pieces. Same-axis pairs (`int \ int`)
/// and subsuming dominators (`array-key \ numeric` where `int` is
/// in both) both qualify.
#[inline]
const fn dominator_member_covers(member: Atom<'_>, removed: Atom<'_>) -> bool {
    matches!(
        (member.kind(), removed.kind()),
        (AtomKind::Bool, AtomKind::Bool | AtomKind::True | AtomKind::False)
            | (AtomKind::Int, AtomKind::Int)
            | (AtomKind::Float, AtomKind::Float)
            | (AtomKind::String, AtomKind::String | AtomKind::ClassLikeString)
            | (AtomKind::Int, AtomKind::Numeric | AtomKind::Scalar | AtomKind::ArrayKey)
            | (AtomKind::Float, AtomKind::Numeric | AtomKind::Scalar)
            | (AtomKind::Bool, AtomKind::Scalar)
            | (AtomKind::String, AtomKind::Numeric | AtomKind::Scalar | AtomKind::ArrayKey)
    )
}
