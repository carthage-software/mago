use serde::Serialize;

use crate::ir::expression::Expression;
use crate::ir::r#type::annotation::TypeAnnotation;
use crate::ir::variable::DirectVariable;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ParameterAnnotation<'arena, S, D, E> {
    pub r#type: Option<&'arena TypeAnnotation<'arena>>,
    pub is_by_reference: bool,
    pub is_variadic: bool,
    pub variable: DirectVariable<'arena>,
    pub default_value: Option<&'arena Expression<'arena, S, D, E>>,
}
