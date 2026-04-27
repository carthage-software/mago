//! `array_map()` return type provider.
//!
//! Preserves array shape (known items/elements) through `array_map` calls
//! by replacing value types with the callback's return type.

use std::collections::BTreeMap;
use std::sync::Arc;

use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::key::ArrayKey;
use mago_codex::ttype::atomic::array::keyed::TKeyedArray;
use mago_codex::ttype::atomic::array::list::TList;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::union::TUnion;
use mago_codex::ttype::wrap_atomic;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::function::FunctionReturnTypeProvider;
use crate::plugin::provider::function::FunctionTarget;

static META: ProviderMeta =
    ProviderMeta::new("php::array::array_map", "array_map", "Preserves array shape through array_map");

#[derive(Default)]
pub struct ArrayMapProvider;

impl Provider for ArrayMapProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl FunctionReturnTypeProvider for ArrayMapProvider {
    fn targets() -> FunctionTarget {
        FunctionTarget::Exact("array_map")
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        // array_map(?callable $callback, array $array, array ...$arrays)
        let argument_count = invocation.argument_count();
        if argument_count < 2 {
            return None;
        }

        let callback_arg = invocation.get_argument(0, &["callback"])?;
        let callback_type = context.get_expression_type(callback_arg)?;
        let callback_is_null = callback_type.is_null();

        if callback_is_null && argument_count == 2 {
            let array_arg = invocation.get_argument(1, &["array"])?;
            return context.get_expression_type(array_arg).cloned();
        }

        if callback_is_null && argument_count > 2 {
            return zip_input_arrays(context, invocation, argument_count);
        }

        if argument_count != 2 {
            return None;
        }

        let array_arg = invocation.get_argument(1, &["array"])?;
        let callback_metadata = context.get_callable_metadata(callback_arg)?;
        let return_type = &callback_metadata.return_type_metadata.as_ref()?.type_union;

        let array_type = context.get_expression_type(array_arg)?;
        let array = array_type.get_single_array()?;

        match array {
            TArray::Keyed(keyed) if keyed.has_known_items() => {
                let known_items = keyed.get_known_items()?;
                let new_items: BTreeMap<_, _> =
                    known_items.iter().map(|(key, (optional, _))| (*key, (*optional, return_type.clone()))).collect();

                let mut result = TKeyedArray::new().with_known_items(new_items).with_non_empty(keyed.is_non_empty());

                if let Some((key_type, _)) = keyed.parameters.as_ref() {
                    result = result.with_parameters(key_type.clone(), Arc::new(return_type.clone()));
                }

                Some(wrap_atomic(TAtomic::Array(TArray::Keyed(result))))
            }
            TArray::List(list) if list.known_elements.is_some() => {
                let known_elements = list.known_elements.as_ref()?;
                let new_elements: BTreeMap<_, _> = known_elements
                    .iter()
                    .map(|(idx, (optional, _))| (*idx, (*optional, return_type.clone())))
                    .collect();

                let result = TList {
                    element_type: if list.element_type.is_never() {
                        list.element_type.clone()
                    } else {
                        Arc::new(return_type.clone())
                    },
                    known_elements: Some(new_elements),
                    known_count: list.known_count,
                    non_empty: list.non_empty,
                };

                Some(wrap_atomic(TAtomic::Array(TArray::List(result))))
            }
            _ => None,
        }
    }
}

fn zip_input_arrays(
    context: &ProviderContext<'_, '_, '_>,
    invocation: &InvocationInfo<'_, '_, '_>,
    argument_count: usize,
) -> Option<TUnion> {
    let array_count = argument_count - 1;
    let mut tuple_items: BTreeMap<ArrayKey, (bool, TUnion)> = BTreeMap::new();
    let mut all_inputs_non_empty = true;

    for offset in 0..array_count {
        let array_arg = invocation.get_argument(offset + 1, &[])?;
        let array_type = context.get_expression_type(array_arg)?;
        let array = array_type.get_single_array()?;

        let value_type = array_value_type(array);
        all_inputs_non_empty &= match array {
            TArray::List(list) => list.non_empty,
            TArray::Keyed(keyed) => keyed.is_non_empty(),
        };

        tuple_items.insert(ArrayKey::Integer(offset as i64), (false, value_type.as_nullable()));
    }

    Some(wrap_atomic(TAtomic::Array(TArray::List(TList {
        element_type: Arc::new(wrap_atomic(TAtomic::Array(TArray::Keyed(
            TKeyedArray::new().with_known_items(tuple_items).with_non_empty(true),
        )))),
        known_elements: None,
        known_count: None,
        non_empty: all_inputs_non_empty,
    }))))
}

fn array_value_type(array: &TArray) -> TUnion {
    match array {
        TArray::List(list) => (*list.element_type).clone(),
        TArray::Keyed(keyed) => keyed.get_value_type().cloned().unwrap_or_else(get_mixed),
    }
}
