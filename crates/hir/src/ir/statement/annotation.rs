use serde::Serialize;

use crate::ir::r#type::annotation::TypeAnnotation;
use crate::ir::variable::DirectVariable;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct VariableBindingAnnotation<'arena> {
    pub variable: DirectVariable<'arena>,
    pub type_annotation: &'arena TypeAnnotation<'arena>,
}
