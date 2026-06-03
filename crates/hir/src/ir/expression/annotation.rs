use serde::Serialize;

use crate::ir::expression::Expression;
use crate::ir::r#type::annotation::TypeAnnotation;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Annotation<'arena, S, D, E> {
    pub expression: &'arena Expression<'arena, S, D, E>,
    pub type_annotation: &'arena TypeAnnotation<'arena>,
}
