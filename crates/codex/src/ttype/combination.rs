use std::collections::BTreeMap;

use ahash::HashSet;
use ordered_float::OrderedFloat;

use mago_atom::Atom;
use mago_atom::AtomMap;
use mago_atom::AtomSet;

use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::array::key::ArrayKey;
use crate::ttype::atomic::derived::TDerived;
use crate::ttype::atomic::scalar::int::TInteger;
use crate::ttype::union::TUnion;

#[derive(Debug)]
pub struct TypeCombination {
    pub value_types: AtomMap<TAtomic>,
    pub has_object_top_type: bool,
    pub enum_names: HashSet<(Atom, Option<Atom>)>,
    pub object_type_params: AtomMap<(Atom, Vec<TUnion>)>,
    pub object_static: AtomMap<bool>,
    pub list_array_counts: Option<HashSet<usize>>,
    pub list_array_sometimes_filled: bool,
    pub list_array_always_filled: bool,
    pub keyed_array_sometimes_filled: bool,
    pub keyed_array_always_filled: bool,
    pub has_empty_array: bool,
    pub has_keyed_array: bool,
    pub keyed_array_entries: BTreeMap<ArrayKey, (bool, TUnion)>,
    pub list_array_entries: BTreeMap<usize, (bool, TUnion)>,
    pub keyed_array_parameters: Option<(TUnion, TUnion)>,
    pub list_array_parameter: Option<TUnion>,
    pub falsy_mixed: Option<bool>,
    pub truthy_mixed: Option<bool>,
    pub nonnull_mixed: Option<bool>,
    pub generic_mixed: bool,
    pub has_mixed: bool,
    pub mixed_from_loop_isset: Option<bool>,
    pub integers: HashSet<TInteger>,
    pub literal_strings: AtomSet,
    pub literal_floats: HashSet<OrderedFloat<f64>>,
    pub class_string_types: AtomMap<TAtomic>,
    pub derived_types: HashSet<TDerived>,
    pub resource: bool,
    pub open_resource: bool,
    pub closed_resource: bool,
}

impl Default for TypeCombination {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeCombination {
    pub fn new() -> Self {
        Self {
            value_types: AtomMap::default(),
            has_object_top_type: false,
            object_type_params: AtomMap::default(),
            object_static: AtomMap::default(),
            list_array_counts: Some(HashSet::default()),
            list_array_sometimes_filled: false,
            list_array_always_filled: true,
            keyed_array_sometimes_filled: false,
            keyed_array_always_filled: true,
            has_empty_array: false,
            has_keyed_array: false,
            keyed_array_entries: BTreeMap::new(),
            list_array_entries: BTreeMap::new(),
            keyed_array_parameters: None,
            list_array_parameter: None,
            falsy_mixed: None,
            truthy_mixed: None,
            nonnull_mixed: None,
            generic_mixed: false,
            has_mixed: false,
            mixed_from_loop_isset: None,
            literal_strings: AtomSet::default(),
            integers: HashSet::default(),
            literal_floats: HashSet::default(),
            class_string_types: AtomMap::default(),
            enum_names: HashSet::default(),
            derived_types: HashSet::default(),
            resource: false,
            open_resource: false,
            closed_resource: false,
        }
    }

    #[inline]
    pub fn is_simple(&self) -> bool {
        if self.value_types.len() == 1
            && !self.has_keyed_array
            && !self.has_empty_array
            && !self.resource
            && !self.open_resource
            && !self.closed_resource
            && let (None, None) = (&self.keyed_array_parameters, &self.list_array_parameter)
        {
            return self.keyed_array_entries.is_empty()
                && self.list_array_entries.is_empty()
                && self.object_type_params.is_empty()
                && self.enum_names.is_empty()
                && self.literal_strings.is_empty()
                && self.class_string_types.is_empty()
                && self.integers.is_empty()
                && self.derived_types.is_empty();
        }

        false
    }
}
