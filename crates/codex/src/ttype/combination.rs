use std::collections::BTreeMap;

use ahash::HashMap;
use ahash::HashSet;
use ordered_float::OrderedFloat;

use mago_interner::StringIdentifier;

use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::array::key::ArrayKey;
use crate::ttype::atomic::derived::TDerived;
use crate::ttype::union::TUnion;

use super::atomic::scalar::int::TInteger;

#[derive(Debug)]
pub struct TypeCombination {
    pub value_types: HashMap<String, TAtomic>,
    pub has_object_top_type: bool,
    pub enum_names: HashSet<(StringIdentifier, Option<StringIdentifier>)>,
    pub object_type_params: HashMap<String, (StringIdentifier, Vec<TUnion>)>,
    pub object_static: HashMap<StringIdentifier, bool>,
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
    pub any_mixed: bool,
    pub vanilla_mixed: bool,
    pub has_mixed: bool,
    pub mixed_from_loop_isset: Option<bool>,
    pub integers: HashSet<TInteger>,
    pub literal_strings: HashSet<String>,
    pub literal_floats: HashSet<OrderedFloat<f64>>,
    pub class_string_types: HashMap<String, TAtomic>,
    pub derived_types: HashSet<TDerived>,
}

impl Default for TypeCombination {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeCombination {
    pub fn new() -> Self {
        Self {
            value_types: HashMap::default(),
            has_object_top_type: false,
            object_type_params: HashMap::default(),
            object_static: HashMap::default(),
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
            vanilla_mixed: false,
            has_mixed: false,
            any_mixed: false,
            mixed_from_loop_isset: None,
            literal_strings: HashSet::default(),
            integers: HashSet::default(),
            literal_floats: HashSet::default(),
            class_string_types: HashMap::default(),
            enum_names: HashSet::default(),
            derived_types: HashSet::default(),
        }
    }

    #[inline]
    pub fn is_simple(&self) -> bool {
        if self.value_types.len() == 1
            && !self.has_keyed_array
            && !self.has_empty_array
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
