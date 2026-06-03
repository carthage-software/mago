use serde::Serialize;

use crate::ir::attribute::Attribute;
use crate::ir::expression::Expression;
use crate::ir::hook::Hook;
use crate::ir::modifier::Modifier;
use crate::ir::r#type::Type;
use crate::ir::r#type::annotation::TypeAnnotation;
use crate::ir::variable::DirectVariable;

pub mod annotation;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Parameter<'arena, S, D, E> {
    pub attributes: &'arena [Attribute<'arena, S, D, E>],
    pub modifiers: &'arena [Modifier],
    pub r#type: Option<&'arena Type<'arena>>,
    pub type_annotation: Option<&'arena TypeAnnotation<'arena>>,
    pub out_annotation: Option<&'arena TypeAnnotation<'arena>>,
    pub is_by_reference: bool,
    pub is_variadic: bool,
    pub variable: DirectVariable<'arena>,
    pub default_value: Option<&'arena Expression<'arena, S, D, E>>,
    pub hooks: &'arena [Hook<'arena, S, D, E>],
}
