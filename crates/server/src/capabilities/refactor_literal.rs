//! Native type hints for literal values in generated declarations.

use mago_syntax::cst::Literal;

pub(super) fn native_hint(literal: &Literal<'_>) -> Option<&'static str> {
    Some(match literal {
        Literal::Integer(integer) => {
            if integer.value? > i64::MAX as u64 {
                "float"
            } else {
                "int"
            }
        }
        Literal::Float(_) => "float",
        Literal::String(_) => "string",
        Literal::True(_) | Literal::False(_) => "bool",
        Literal::Null(_) => "null",
    })
}
