use foldhash::HashMap;
use mago_codex::metadata::parameter::FunctionLikeParameterMetadata;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_syntax::cst::ArrayElement;
use mago_syntax::cst::Expression;
use mago_syntax::cst::Literal;

use crate::utils::misc::unwrap_expression;

pub(crate) fn accepts_callable_collection(parameter: &FunctionLikeParameterMetadata) -> bool {
    parameter
        .type_metadata
        .as_ref()
        .is_some_and(|metadata| metadata.type_union.types.iter().any(atomic_is_callable_collection))
}

fn atomic_is_callable_collection(atomic: &TAtomic) -> bool {
    let TAtomic::Array(array) = atomic else {
        return false;
    };

    match array {
        TArray::Keyed(keyed) => {
            keyed.known_items.as_deref().is_some_and(|items| {
                items
                    .values()
                    .any(|(_, value)| value.has_callable() || value.types.iter().any(atomic_is_callable_collection))
            }) || keyed
                .parameters
                .as_ref()
                .is_some_and(|(_, value)| value.has_callable() || value.types.iter().any(atomic_is_callable_collection))
        }
        TArray::List(list) => {
            list.known_elements.as_deref().is_some_and(|items| {
                items
                    .values()
                    .any(|(_, value)| value.has_callable() || value.types.iter().any(atomic_is_callable_collection))
            }) || list.element_type.has_callable()
                || list.element_type.types.iter().any(atomic_is_callable_collection)
        }
    }
}

/// Extracts values from an inline array accepted as a collection of callbacks.
/// PHP's array construction keeps the last value for duplicate string keys, and
/// a spread is evaluated in place, so mirror those two cases for throw analysis.
pub(crate) fn callback_values_from_array<'ast, 'arena>(
    expression: &'ast Expression<'arena>,
) -> Vec<&'ast Expression<'arena>> {
    let mut values = Vec::new();
    let mut key_offsets = HashMap::default();
    collect_callback_values_from_array(expression, &mut values, &mut key_offsets);
    values
}

fn collect_callback_values_from_array<'ast, 'arena>(
    expression: &'ast Expression<'arena>,
    values: &mut Vec<&'ast Expression<'arena>>,
    key_offsets: &mut HashMap<&'arena [u8], usize>,
) {
    let expression = unwrap_expression(expression);
    let elements = match expression {
        Expression::Array(array) => &array.elements,
        Expression::LegacyArray(array) => &array.elements,
        _ => return,
    };

    for element in elements.iter() {
        match element {
            ArrayElement::KeyValue(element) => {
                let key = match unwrap_expression(element.key) {
                    Expression::Literal(Literal::String(key)) => key.value,
                    _ => None,
                };
                if let Some(key) = key {
                    if let Some(offset) = key_offsets.get(key).copied() {
                        values[offset] = unwrap_expression(element.value);
                        continue;
                    }

                    key_offsets.insert(key, values.len());
                }

                values.push(unwrap_expression(element.value));
            }
            ArrayElement::Value(element) => values.push(unwrap_expression(element.value)),
            ArrayElement::Variadic(element) => {
                collect_callback_values_from_array(element.value, values, key_offsets);
            }
            ArrayElement::Missing(_) => {}
        }
    }
}
