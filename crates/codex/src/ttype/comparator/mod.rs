use mago_atom::Atom;

use crate::ttype::atomic::TAtomic;
use crate::ttype::template::TemplateBound;
use crate::ttype::union::TUnion;

mod array_comparator;
mod callable_comparator;
mod class_string_comparator;
mod derived_comparator;
mod generic_comparator;
mod integer_comparator;
mod iterable_comparator;
mod resource_comparator;
mod scalar_comparator;

pub(super) mod object_comparator;

pub mod atomic_comparator;
pub mod union_comparator;

#[derive(Debug)]
pub struct ComparisonResult {
    pub type_coerced: Option<bool>,
    pub type_coerced_from_nested_mixed: Option<bool>,
    pub type_coerced_from_as_mixed: Option<bool>,
    pub type_coerced_to_literal: Option<bool>,
    pub replacement_union_type: Option<TUnion>,
    pub replacement_atomic_type: Option<TAtomic>,
    pub type_variable_lower_bounds: Vec<(Atom, TemplateBound)>,
    pub type_variable_upper_bounds: Vec<(Atom, TemplateBound)>,
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
            type_coerced_from_as_mixed: None,
            type_coerced_to_literal: None,
            replacement_union_type: None,
            replacement_atomic_type: None,
            type_variable_lower_bounds: vec![],
            type_variable_upper_bounds: vec![],
        }
    }
}
