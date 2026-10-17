use std::ops::Range;

use foldhash::HashMap;
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_syntax::cst::ArrayElement;
use mago_syntax::cst::Expression;
use mago_syntax::cst::Literal;

use crate::invocation::InvocationTarget;
use crate::utils::misc::unwrap_expression;

#[derive(Clone, Copy)]
pub(crate) struct BuiltinCallbackSpec {
    callback_parameter_offset: Option<usize>,
    callback_map_parameter_offset: Option<usize>,
    comparison_callback_count: Option<usize>,
}

impl BuiltinCallbackSpec {
    const fn callback_parameter(offset: usize) -> Self {
        Self {
            callback_parameter_offset: Some(offset),
            callback_map_parameter_offset: None,
            comparison_callback_count: None,
        }
    }

    const fn callback_map_parameter(offset: usize) -> Self {
        Self {
            callback_parameter_offset: None,
            callback_map_parameter_offset: Some(offset),
            comparison_callback_count: None,
        }
    }

    const fn comparison_callbacks(count: usize) -> Self {
        Self {
            callback_parameter_offset: None,
            callback_map_parameter_offset: None,
            comparison_callback_count: Some(count),
        }
    }

    pub(crate) fn callback_parameter_offset(self) -> Option<usize> {
        self.callback_parameter_offset
    }

    pub(crate) fn callback_map_parameter_offset(self) -> Option<usize> {
        self.callback_map_parameter_offset
    }

    pub(crate) fn comparison_callback_argument_range(self, argument_count: usize) -> Option<Range<usize>> {
        let callback_count = self.comparison_callback_count?;
        if argument_count < callback_count + 2 {
            return None;
        }

        Some(argument_count - callback_count..argument_count)
    }

    pub(crate) fn is_immediate_callback_argument(
        self,
        parameter_offset: usize,
        argument_offset: usize,
        argument_count: usize,
    ) -> bool {
        self.callback_parameter_offset() == Some(parameter_offset)
            || self
                .comparison_callback_argument_range(argument_count)
                .is_some_and(|range| range.contains(&argument_offset))
    }
}

pub(crate) fn builtin_callback_spec(target: &InvocationTarget<'_>) -> Option<&'static BuiltinCallbackSpec> {
    let name = builtin_function_name(target)?;

    BUILTIN_CALLBACKS.iter().find_map(|(builtin, spec)| name.eq_ignore_ascii_case(builtin).then_some(spec))
}

pub(crate) fn immediate_builtin_callback_map_values<'ast, 'arena>(
    expression: &'ast Expression<'arena>,
) -> Vec<&'ast Expression<'arena>> {
    let mut values = Vec::new();
    let mut key_offsets = HashMap::default();
    collect_immediate_builtin_callback_map_values(expression, &mut values, &mut key_offsets);
    values
}

fn collect_immediate_builtin_callback_map_values<'ast, 'arena>(
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
                collect_immediate_builtin_callback_map_values(element.value, values, key_offsets);
            }
            ArrayElement::Missing(_) => {}
        }
    }
}

const BUILTIN_CALLBACKS: &[(&[u8], BuiltinCallbackSpec)] = &[
    (b"array_map", BuiltinCallbackSpec::callback_parameter(0)),
    (b"call_user_func", BuiltinCallbackSpec::callback_parameter(0)),
    (b"call_user_func_array", BuiltinCallbackSpec::callback_parameter(0)),
    (b"forward_static_call", BuiltinCallbackSpec::callback_parameter(0)),
    (b"forward_static_call_array", BuiltinCallbackSpec::callback_parameter(0)),
    (b"array_all", BuiltinCallbackSpec::callback_parameter(1)),
    (b"array_any", BuiltinCallbackSpec::callback_parameter(1)),
    (b"array_filter", BuiltinCallbackSpec::callback_parameter(1)),
    (b"array_find", BuiltinCallbackSpec::callback_parameter(1)),
    (b"array_find_key", BuiltinCallbackSpec::callback_parameter(1)),
    (b"array_reduce", BuiltinCallbackSpec::callback_parameter(1)),
    (b"array_walk", BuiltinCallbackSpec::callback_parameter(1)),
    (b"array_walk_recursive", BuiltinCallbackSpec::callback_parameter(1)),
    (b"iterator_apply", BuiltinCallbackSpec::callback_parameter(1)),
    (b"mb_ereg_replace_callback", BuiltinCallbackSpec::callback_parameter(1)),
    (b"preg_replace_callback", BuiltinCallbackSpec::callback_parameter(1)),
    (b"uasort", BuiltinCallbackSpec::callback_parameter(1)),
    (b"uksort", BuiltinCallbackSpec::callback_parameter(1)),
    (b"usort", BuiltinCallbackSpec::callback_parameter(1)),
    (b"preg_replace_callback_array", BuiltinCallbackSpec::callback_map_parameter(0)),
    (b"array_udiff_uassoc", BuiltinCallbackSpec::comparison_callbacks(2)),
    (b"array_uintersect_uassoc", BuiltinCallbackSpec::comparison_callbacks(2)),
    (b"array_diff_ukey", BuiltinCallbackSpec::comparison_callbacks(1)),
    (b"array_udiff", BuiltinCallbackSpec::comparison_callbacks(1)),
    (b"array_udiff_assoc", BuiltinCallbackSpec::comparison_callbacks(1)),
    (b"array_diff_uassoc", BuiltinCallbackSpec::comparison_callbacks(1)),
    (b"array_uintersect_assoc", BuiltinCallbackSpec::comparison_callbacks(1)),
    (b"array_intersect_uassoc", BuiltinCallbackSpec::comparison_callbacks(1)),
    (b"array_intersect_ukey", BuiltinCallbackSpec::comparison_callbacks(1)),
    (b"array_uintersect", BuiltinCallbackSpec::comparison_callbacks(1)),
];

fn builtin_function_name<'target>(target: &'target InvocationTarget<'_>) -> Option<&'target [u8]> {
    let metadata = target.get_function_like_metadata()?;
    if !metadata.flags.is_built_in() {
        return None;
    }

    let Some(FunctionLikeIdentifier::Function(name)) = target.get_function_like_identifier() else {
        return None;
    };

    Some(name.as_bytes().rsplit(|byte| *byte == b'\\').next().unwrap_or(name.as_bytes()))
}
