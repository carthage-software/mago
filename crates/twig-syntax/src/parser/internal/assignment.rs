use mago_span::HasSpan;

use crate::cst::ArrayElement;
use crate::cst::Expression;
use crate::error::ParseError;

pub(super) fn validate<'arena>(expression: &Expression<'arena>) -> Result<(), ParseError<'arena>> {
    let target = ungroup(expression);
    let invalid = |message: &'static [u8]| ParseError::Message(message, target.span());
    match target {
        Expression::Name(_) => Ok(()),
        Expression::Array(array) => {
            let mut found = false;
            for element in array.elements.iter() {
                match element {
                    ArrayElement::Missing(_) => {}
                    ArrayElement::Value(value) if is_name(value.value) => found = true,
                    _ => return Err(invalid(b"A destructuring pattern can contain only variables and empty slots.")),
                }
            }
            if found { Ok(()) } else { Err(invalid(b"A destructuring pattern needs at least one variable.")) }
        }
        Expression::HashMap(mapping) => {
            if mapping.entries.is_empty() {
                return Err(invalid(b"A destructuring pattern needs at least one variable."));
            }
            for entry in mapping.entries.iter() {
                if entry.ellipsis.is_some() || !is_name(&entry.value) {
                    return Err(invalid(b"A mapping assignment needs variable values and no spread entries."));
                }
                if !entry.key.as_ref().is_some_and(|key| {
                    matches!(
                        ungroup(key),
                        Expression::String(_) | Expression::Number(_) | Expression::Bool(_) | Expression::Null(_)
                    )
                }) {
                    return Err(invalid(b"A mapping assignment needs literal keys."));
                }
            }
            Ok(())
        }
        _ => Err(invalid(b"Only variables and destructuring patterns can be assignment targets.")),
    }
}

fn is_name(expression: &Expression<'_>) -> bool {
    matches!(ungroup(expression), Expression::Name(_))
}

fn ungroup<'ast, 'arena>(mut expression: &'ast Expression<'arena>) -> &'ast Expression<'arena> {
    while let Expression::Parenthesized(group) = expression {
        expression = group.inner;
    }
    expression
}
