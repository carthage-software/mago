#[cfg(feature = "serde")]
use serde::Serialize;

use crate::ir::expression::Expression;
use crate::ir::variable::annotation::VariableAnnotation;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Annotation<'arena, I, S, E> {
    pub annotation: &'arena VariableAnnotation<'arena>,
    pub expression: &'arena Expression<'arena, I, S, E>,
}
