//! Simple-type constructors on [`TypeBuilder`]: literal atoms/types
//! (`int(5)`, `string('x')`), the well-known scalar atoms/types (`int`,
//! `string`, `bool`, …) so callers never reach into `well_known` directly, and
//! the canonicalizing binary [`union_type`](TypeBuilder::union_type).
//!
//! Naming is precise about the return: `*_atom` yields an [`Atom`], `*_type` a
//! [`Type`]; `int_literal_*` is `int(5)` while `int_*` is the well-known `int`.

use mago_allocator::Arena;

use crate::ty::Type;
use crate::ty::atom::Atom;
use crate::ty::atom::payload::array::KnownElement;
use crate::ty::atom::payload::scalar::float::FloatAtom;
use crate::ty::atom::payload::scalar::float::LiteralFloat;
use crate::ty::atom::payload::scalar::int::IntAtom;
use crate::ty::builder::TypeBuilder;
use crate::ty::join::JoinOptions;
use crate::ty::join::compute_with;
use crate::ty::well_known;
use ordered_float::OrderedFloat;

/// Generate a `*_atom` / `*_type` accessor pair for a well-known atom constant.
macro_rules! well_known_accessors {
    ($($atom_fn:ident, $type_fn:ident => $konst:path;)*) => {
        $(
            #[inline(always)]
            #[must_use]
            pub const fn $atom_fn(&self) -> Atom<'arena> {
                $konst
            }

            #[inline(always)]
            #[must_use]
            pub const fn $type_fn(&self) -> Type<'arena> {
                Type::from_canonical_atoms(&[$konst])
            }
        )*
    };
}

impl<'arena, S, A> TypeBuilder<'_, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    /// A literal integer atom (`int(value)`).
    #[inline]
    #[must_use]
    pub const fn int_literal_atom(&self, value: i64) -> Atom<'arena> {
        Atom::Int(IntAtom::Literal(value))
    }

    /// A literal integer type (`int(value)`).
    #[inline]
    #[must_use]
    pub fn int_literal_type(&mut self, value: i64) -> Type<'arena> {
        let atom = self.int_literal_atom(value);

        self.union_of(&[atom])
    }

    /// A literal float atom (`float(value)`).
    #[inline]
    #[must_use]
    pub const fn float_literal_atom(&self, value: f64) -> Atom<'arena> {
        Atom::Float(FloatAtom::Literal(LiteralFloat(OrderedFloat(value))))
    }

    /// A literal float type; a non-finite value widens to the well-known
    /// `float`.
    #[inline]
    #[must_use]
    pub fn float_literal_type(&mut self, value: f64) -> Type<'arena> {
        if !value.is_finite() {
            return self.float_type();
        }

        let atom = self.float_literal_atom(value);
        self.union_of(&[atom])
    }

    /// A literal string type (`string('value')`), interning the bytes and
    /// deriving refinement flags as [`string_literal_atom`](Self::string_literal_atom) does.
    #[inline]
    #[must_use]
    pub fn string_literal_type(&mut self, value: &[u8]) -> Type<'arena> {
        let atom = self.string_literal_atom(value);

        self.union_of(&[atom])
    }

    /// An integer-range type (`int<lower, upper>`), either bound open when
    /// `None`.
    #[inline]
    #[must_use]
    pub fn int_range_type(&mut self, lower: Option<i64>, upper: Option<i64>) -> Type<'arena> {
        let atom = self.int_range_atom(lower, upper);

        self.union_of(&[atom])
    }

    /// An unsealed list type (`list<element>` / `non-empty-list<element>`).
    #[inline]
    #[must_use]
    pub fn list_of_type(&mut self, element: Type<'arena>, non_empty: bool) -> Type<'arena> {
        let atom = self.list_of_atom(element, non_empty);

        self.union_of(&[atom])
    }

    /// A sealed list type (`list{0: T0, 1: T1, …}`) from the given element
    /// types in order, all required.
    #[inline]
    #[must_use]
    pub fn sealed_list_type(&mut self, elements: &[Type<'arena>], non_empty: bool) -> Type<'arena> {
        let mut known = self.scratch_vec::<KnownElement<'arena>>();
        for (index, &value) in elements.iter().enumerate() {
            known.push(KnownElement { index: index as u32, value, optional: false });
        }

        let atom = self.sealed_list_atom(&known, non_empty);
        self.union_of(&[atom])
    }

    /// An unsealed keyed-array type (`array<K, V>` / `non-empty-array<K, V>`).
    #[inline]
    #[must_use]
    pub fn keyed_unsealed_type(&mut self, key: Type<'arena>, value: Type<'arena>, non_empty: bool) -> Type<'arena> {
        let atom = self.unsealed_keyed_array_atom(key, value, non_empty);

        self.union_of(&[atom])
    }

    /// The union of two types, canonicalized under the given [`JoinOptions`].
    /// There is no default: the join policy (literal collapse, range merging,
    /// …) is the caller's to choose.
    #[must_use]
    pub fn union_type(&mut self, left: Type<'arena>, right: Type<'arena>, options: &JoinOptions) -> Type<'arena> {
        let mut atoms = self.scratch_vec::<Atom<'arena>>();
        atoms.extend_from_slice(left.atoms);
        atoms.extend_from_slice(right.atoms);

        let canonical = compute_with(&atoms, options, self);
        self.union_of(&canonical)
    }

    well_known_accessors! {
        null_atom, null_type => well_known::NULL;
        never_atom, never_type => well_known::NEVER;
        void_atom, void_type => well_known::VOID;
        true_atom, true_type => well_known::TRUE;
        false_atom, false_type => well_known::FALSE;
        bool_atom, bool_type => well_known::BOOL;
        int_atom, int_type => well_known::INT;
        float_atom, float_type => well_known::FLOAT;
        string_atom, string_type => well_known::STRING;
        object_atom, object_type => well_known::OBJECT;
        scalar_atom, scalar_type => well_known::SCALAR;
        numeric_atom, numeric_type => well_known::NUMERIC;
        mixed_atom, mixed_type => well_known::MIXED;
        array_key_atom, array_key_type => well_known::ARRAY_KEY;
        non_negative_int_atom, non_negative_int_type => well_known::NON_NEGATIVE_INT;
        positive_int_atom, positive_int_type => well_known::POSITIVE_INT;
        negative_int_atom, negative_int_type => well_known::NEGATIVE_INT;
        non_positive_int_atom, non_positive_int_type => well_known::NON_POSITIVE_INT;
        non_empty_string_atom, non_empty_string_type => well_known::NON_EMPTY_STRING;
        empty_string_atom, empty_string_type => well_known::EMPTY_STRING;
        empty_array_atom, empty_array_type => well_known::EMPTY_ARRAY;
    }
}
