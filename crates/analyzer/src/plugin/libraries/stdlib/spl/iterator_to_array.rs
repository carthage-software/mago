//! `iterator_to_array()` return type provider.

use std::rc::Rc;

use mago_codex::ttype::add_optional_union_type;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::comparator::ComparisonResult;
use mago_codex::ttype::comparator::union_comparator::is_contained_by;
use mago_codex::ttype::get_arraykey;
use mago_codex::ttype::get_iterable_parameters;
use mago_codex::ttype::get_keyed_array;
use mago_codex::ttype::get_list;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::get_mixed_iterable;
use mago_codex::ttype::union::TUnion;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::function::FunctionReturnTypeProvider;
use crate::plugin::provider::function::FunctionTarget;

static META: ProviderMeta = ProviderMeta::new(
    "php::spl::iterator_to_array",
    "iterator_to_array",
    "Returns array with inferred key/value types from iterator",
);

/// Provider for the `iterator_to_array()` function.
///
/// Returns an array type with key/value types inferred from the iterator.
#[derive(Default)]
pub struct IteratorToArrayProvider;

impl Provider for IteratorToArrayProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl FunctionReturnTypeProvider for IteratorToArrayProvider {
    fn targets() -> FunctionTarget {
        FunctionTarget::Exact("iterator_to_array")
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        let preserve_keys = match invocation.get_argument(1, &["preserve_keys"]) {
            Some(argument) => context.get_expression_type(argument).and_then(|argument_type| {
                if argument_type.is_always_truthy() {
                    Some(true)
                } else if argument_type.is_always_falsy() {
                    Some(false)
                } else {
                    None
                }
            }),
            None => Some(true),
        };

        let iterator_argument = invocation
            .get_argument(0, &["iterator"])
            .and_then(|arg| context.get_rc_expression_type(arg))
            .cloned()
            .unwrap_or_else(|| Rc::new(get_mixed_iterable()));

        let codebase = context.codebase();

        let mut key_type: Option<TUnion> = None;
        let mut value_type: Option<TUnion> = None;

        let mut iterator_atomics: Vec<&TAtomic> = iterator_argument.types.iter().collect();
        while let Some(iterator_atomic) = iterator_atomics.pop() {
            if let TAtomic::GenericParameter(parameter) = iterator_atomic {
                iterator_atomics.extend(parameter.constraint.types.iter());
            }

            let Some((k, v)) = get_iterable_parameters(iterator_atomic, codebase) else {
                continue;
            };

            key_type = Some(add_optional_union_type(k, key_type.as_ref(), codebase));
            value_type = Some(add_optional_union_type(v, value_type.as_ref(), codebase));
        }

        let mut iterator_key_type = key_type.unwrap_or_else(get_arraykey);
        let iterator_value_type = value_type.unwrap_or_else(get_mixed);

        let Some(preserve_keys) = preserve_keys else {
            return Some(get_keyed_array(get_arraykey(), iterator_value_type));
        };

        if !preserve_keys {
            return Some(get_list(iterator_value_type));
        }

        if !is_contained_by(
            codebase,
            &iterator_key_type,
            &get_arraykey(),
            false,
            false,
            false,
            &mut ComparisonResult::default(),
        ) {
            iterator_key_type = get_arraykey();
        }

        Some(get_keyed_array(iterator_key_type, iterator_value_type))
    }
}
