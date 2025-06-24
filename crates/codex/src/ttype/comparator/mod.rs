use mago_interner::StringIdentifier;

use crate::data_flow::node::DataFlowNode;
use crate::ttype::atomic::TAtomic;
use crate::ttype::template::TemplateBound;
use crate::ttype::union::TUnion;

pub mod array_comparator;
pub mod atomic_comparator;
pub mod callable_comparator;
pub mod class_string_comparator;
pub mod generic_comparator;
pub mod integer_comparator;
pub mod iterable_comparator;
pub mod object_comparator;
pub mod resource_comparator;
pub mod scalar_comparator;
pub mod union_comparator;

#[derive(Debug)]
pub struct ComparisonResult {
    pub type_coerced: Option<bool>,
    pub type_coerced_from_nested_mixed: Option<bool>,
    pub type_coerced_from_nested_any: Option<bool>,
    pub type_coerced_from_as_mixed: Option<bool>,
    pub type_coerced_to_literal: Option<bool>,
    pub replacement_union_type: Option<TUnion>,
    pub replacement_atomic_type: Option<TAtomic>,
    pub type_variable_lower_bounds: Vec<(StringIdentifier, TemplateBound)>,
    pub type_variable_upper_bounds: Vec<(StringIdentifier, TemplateBound)>,
    pub type_mismatch_parents: Option<(Vec<DataFlowNode>, TUnion)>,
}

impl Default for ComparisonResult {
    fn default() -> Self {
        Self::new()
    }
}

impl ComparisonResult {
    pub fn new() -> Self {
        Self {
            type_coerced: None,
            type_coerced_from_nested_mixed: None,
            type_coerced_from_nested_any: None,
            type_coerced_from_as_mixed: None,
            type_coerced_to_literal: None,
            replacement_union_type: None,
            replacement_atomic_type: None,
            type_variable_lower_bounds: vec![],
            type_variable_upper_bounds: vec![],
            type_mismatch_parents: None,
        }
    }
}
