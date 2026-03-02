use std::collections::BTreeMap;

use foldhash::HashSet;
use ordered_float::OrderedFloat;

use mago_atom::Atom;
use mago_atom::AtomMap;
use mago_atom::AtomSet;

use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::array::TArray;
use crate::ttype::atomic::array::key::ArrayKey;
use crate::ttype::atomic::derived::TDerived;
use crate::ttype::atomic::scalar::int::TInteger;
use crate::ttype::union::TUnion;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct CombinationFlags(u32);

impl CombinationFlags {
    pub const HAS_OBJECT_TOP_TYPE: CombinationFlags = CombinationFlags(1 << 0);
    pub const LIST_ARRAY_SOMETIMES_FILLED: CombinationFlags = CombinationFlags(1 << 1);
    pub const LIST_ARRAY_ALWAYS_FILLED: CombinationFlags = CombinationFlags(1 << 2);
    pub const KEYED_ARRAY_SOMETIMES_FILLED: CombinationFlags = CombinationFlags(1 << 3);
    pub const KEYED_ARRAY_ALWAYS_FILLED: CombinationFlags = CombinationFlags(1 << 4);
    pub const HAS_EMPTY_ARRAY: CombinationFlags = CombinationFlags(1 << 5);
    pub const HAS_KEYED_ARRAY: CombinationFlags = CombinationFlags(1 << 6);
    pub const GENERIC_MIXED: CombinationFlags = CombinationFlags(1 << 7);
    pub const HAS_MIXED: CombinationFlags = CombinationFlags(1 << 8);
    pub const RESOURCE: CombinationFlags = CombinationFlags(1 << 9);
    pub const OPEN_RESOURCE: CombinationFlags = CombinationFlags(1 << 10);
    pub const CLOSED_RESOURCE: CombinationFlags = CombinationFlags(1 << 11);
    // Tristate encoding: 2 bits each (None=neither set, Some(false)=SET only, Some(true)=SET+VALUE)
    const FALSY_MIXED_SET: CombinationFlags = CombinationFlags(1 << 12);
    const FALSY_MIXED_VALUE: CombinationFlags = CombinationFlags(1 << 13);
    const TRUTHY_MIXED_SET: CombinationFlags = CombinationFlags(1 << 14);
    const TRUTHY_MIXED_VALUE: CombinationFlags = CombinationFlags(1 << 15);
    const NONNULL_MIXED_SET: CombinationFlags = CombinationFlags(1 << 16);
    const NONNULL_MIXED_VALUE: CombinationFlags = CombinationFlags(1 << 17);
    const MIXED_FROM_LOOP_ISSET_SET: CombinationFlags = CombinationFlags(1 << 18);
    const MIXED_FROM_LOOP_ISSET_VALUE: CombinationFlags = CombinationFlags(1 << 19);
}

impl CombinationFlags {
    #[inline]
    pub const fn insert(&mut self, other: CombinationFlags) {
        self.0 |= other.0;
    }

    #[inline]
    pub const fn remove(&mut self, other: CombinationFlags) {
        self.0 &= !other.0;
    }

    #[inline]
    pub const fn contains(self, other: CombinationFlags) -> bool {
        (self.0 & other.0) == other.0
    }

    #[inline]
    pub const fn intersects(self, other: CombinationFlags) -> bool {
        (self.0 & other.0) != 0
    }

    /// Get a tristate value (Option<bool>) from two bits.
    #[inline]
    #[must_use]
    pub fn get_tristate(self, set_bit: CombinationFlags, value_bit: CombinationFlags) -> Option<bool> {
        if self.contains(set_bit) { Some(self.contains(value_bit)) } else { None }
    }

    /// Set a tristate value (Option<bool>) using two bits.
    #[inline]
    pub fn set_tristate(&mut self, set_bit: CombinationFlags, value_bit: CombinationFlags, value: Option<bool>) {
        match value {
            None => {
                self.remove(set_bit);
                self.remove(value_bit);
            }
            Some(false) => {
                self.insert(set_bit);
                self.remove(value_bit);
            }
            Some(true) => {
                self.insert(set_bit);
                self.insert(value_bit);
            }
        }
    }

    #[inline]
    #[must_use]
    pub fn falsy_mixed(self) -> Option<bool> {
        self.get_tristate(Self::FALSY_MIXED_SET, Self::FALSY_MIXED_VALUE)
    }

    #[inline]
    pub fn set_falsy_mixed(&mut self, value: Option<bool>) {
        self.set_tristate(Self::FALSY_MIXED_SET, Self::FALSY_MIXED_VALUE, value);
    }

    #[inline]
    #[must_use]
    pub fn truthy_mixed(self) -> Option<bool> {
        self.get_tristate(Self::TRUTHY_MIXED_SET, Self::TRUTHY_MIXED_VALUE)
    }

    #[inline]
    pub fn set_truthy_mixed(&mut self, value: Option<bool>) {
        self.set_tristate(Self::TRUTHY_MIXED_SET, Self::TRUTHY_MIXED_VALUE, value);
    }

    #[inline]
    #[must_use]
    pub fn nonnull_mixed(self) -> Option<bool> {
        self.get_tristate(Self::NONNULL_MIXED_SET, Self::NONNULL_MIXED_VALUE)
    }

    #[inline]
    pub fn set_nonnull_mixed(&mut self, value: Option<bool>) {
        self.set_tristate(Self::NONNULL_MIXED_SET, Self::NONNULL_MIXED_VALUE, value);
    }

    #[inline]
    #[must_use]
    pub fn mixed_from_loop_isset(self) -> Option<bool> {
        self.get_tristate(Self::MIXED_FROM_LOOP_ISSET_SET, Self::MIXED_FROM_LOOP_ISSET_VALUE)
    }

    #[inline]
    pub fn set_mixed_from_loop_isset(&mut self, value: Option<bool>) {
        self.set_tristate(Self::MIXED_FROM_LOOP_ISSET_SET, Self::MIXED_FROM_LOOP_ISSET_VALUE, value);
    }
}

#[derive(Debug)]
pub struct TypeCombination {
    pub flags: CombinationFlags,
    pub value_types: AtomMap<TAtomic>,
    pub enum_names: HashSet<(Atom, Option<Atom>)>,
    pub object_type_params: AtomMap<(Atom, Vec<TUnion>)>,
    pub object_static: AtomMap<bool>,
    pub list_array_counts: Option<HashSet<usize>>,
    pub keyed_array_entries: BTreeMap<ArrayKey, (bool, TUnion)>,
    pub list_array_entries: BTreeMap<usize, (bool, TUnion)>,
    pub keyed_array_parameters: Option<(TUnion, TUnion)>,
    pub list_array_parameter: Option<TUnion>,
    pub sealed_arrays: Vec<TArray>,
    pub integers: Vec<TInteger>,
    pub literal_strings: AtomSet,
    pub literal_floats: Vec<OrderedFloat<f64>>,
    pub class_string_types: AtomMap<TAtomic>,
    pub derived_types: HashSet<TDerived>,
}

impl Default for TypeCombination {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeCombination {
    #[must_use]
    pub fn new() -> Self {
        let flags = CombinationFlags::LIST_ARRAY_ALWAYS_FILLED | CombinationFlags::KEYED_ARRAY_ALWAYS_FILLED;

        Self {
            flags,
            value_types: AtomMap::default(),
            object_type_params: AtomMap::default(),
            object_static: AtomMap::default(),
            list_array_counts: Some(HashSet::default()),
            keyed_array_entries: BTreeMap::new(),
            list_array_entries: BTreeMap::new(),
            keyed_array_parameters: None,
            list_array_parameter: None,
            sealed_arrays: Vec::new(),
            literal_strings: AtomSet::default(),
            integers: Vec::new(),
            literal_floats: Vec::new(),
            class_string_types: AtomMap::default(),
            enum_names: HashSet::default(),
            derived_types: HashSet::default(),
        }
    }

    #[inline]
    #[must_use]
    pub fn is_simple(&self) -> bool {
        if self.value_types.len() == 1
            && self.sealed_arrays.is_empty()
            && !self.flags.contains(CombinationFlags::HAS_KEYED_ARRAY)
            && !self.flags.contains(CombinationFlags::HAS_EMPTY_ARRAY)
            && !self.flags.intersects(
                CombinationFlags::RESOURCE | CombinationFlags::OPEN_RESOURCE | CombinationFlags::CLOSED_RESOURCE,
            )
            && self.keyed_array_parameters.is_none()
            && self.list_array_parameter.is_none()
        {
            return self.object_type_params.is_empty()
                && self.enum_names.is_empty()
                && self.literal_strings.is_empty()
                && self.literal_floats.is_empty()
                && self.class_string_types.is_empty()
                && self.integers.is_empty()
                && self.derived_types.is_empty();
        }

        false
    }
}

impl std::ops::BitOr for CombinationFlags {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        CombinationFlags(self.0 | rhs.0)
    }
}

impl std::ops::BitAnd for CombinationFlags {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        CombinationFlags(self.0 & rhs.0)
    }
}

impl std::ops::BitXor for CombinationFlags {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        CombinationFlags(self.0 ^ rhs.0)
    }
}

impl std::ops::Not for CombinationFlags {
    type Output = Self;

    #[inline]
    fn not(self) -> Self::Output {
        CombinationFlags(!self.0)
    }
}
