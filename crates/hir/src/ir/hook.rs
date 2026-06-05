use serde::Serialize;

use crate::ir::attribute::Attribute;
use crate::ir::expression::Expression;
use crate::ir::modifier::Modifier;
use crate::ir::name::Name;
use crate::ir::parameter::Parameter;
use crate::ir::statement::Statement;
use crate::ir::r#type::annotation::TypeAnnotation;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Hook<'arena, S, D, E> {
    pub attributes: &'arena [Attribute<'arena, S, D, E>],
    pub modifiers: &'arena [Modifier],
    pub return_by_reference: bool,
    pub name: Name<'arena>,
    pub is_variadic: bool,
    pub parameters: &'arena [Parameter<'arena, S, D, E>],
    pub has_docblock: bool,
    pub return_type_annotation: Option<&'arena TypeAnnotation<'arena>>,
    pub body: Option<HookBody<'arena, S, D, E>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[serde(tag = "type", content = "value")]
pub enum HookBody<'arena, S, D, E> {
    Expression(&'arena Expression<'arena, S, D, E>),
    Statements(&'arena [Statement<'arena, S, D, E>]),
}
