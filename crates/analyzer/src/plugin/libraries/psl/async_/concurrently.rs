//! `Psl\Async\concurrently()` return type provider.

use std::collections::BTreeMap;
use std::sync::Arc;

use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::keyed::TKeyedArray;
use mago_codex::ttype::atomic::array::list::TList;
use mago_codex::ttype::atomic::callable::TCallable;
use mago_codex::ttype::get_array_parameters;
use mago_codex::ttype::get_never;
use mago_codex::ttype::union::TUnion;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::function::FunctionReturnTypeProvider;
use crate::plugin::provider::function::FunctionTarget;

static META: ProviderMeta = ProviderMeta::new(
    "psl::async::concurrently",
    "Psl\\Async\\concurrently",
    "Extracts return types from closure array values, preserving array shape",
);

/// Provider for the `Psl\Async\concurrently()` function.
///
/// Transforms `array<K, (Closure(): V)>` → `array<K, V>`, preserving
/// sealed array shapes, list structure, and non-empty status.
#[derive(Default)]
pub struct ConcurrentlyProvider;

impl Provider for ConcurrentlyProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl FunctionReturnTypeProvider for ConcurrentlyProvider {
    fn targets() -> FunctionTarget {
        FunctionTarget::Exact("psl\\async\\concurrently")
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        let tasks_arg = invocation.get_argument(0, &["tasks"])?;
        let tasks_type = context.get_expression_type(tasks_arg)?;

        let array = tasks_type.get_single_array()?;

        unwrap_closure_return_array(array, context)
    }
}

/// Extracts closure return types from each element in an array type,
/// preserving array shape.
fn unwrap_closure_return_array(array: &TArray, context: &ProviderContext<'_, '_, '_>) -> Option<TUnion> {
    match array {
        TArray::List(list) => Some(TUnion::from_atomic(TAtomic::Array(TArray::List(TList {
            element_type: Arc::new(if list.element_type.is_never() {
                get_never()
            } else {
                extract_closure_return_type(&list.element_type)?
            }),
            known_count: list.known_count,
            non_empty: list.non_empty,
            known_elements: if let Some(known_elements) = &list.known_elements {
                let mut new_elements = BTreeMap::new();
                for (index, (possibly_undefined, element_type)) in known_elements {
                    let inner = extract_closure_return_type(element_type)?;
                    new_elements.insert(*index, (*possibly_undefined, inner));
                }

                Some(new_elements)
            } else {
                None
            },
        })))),
        TArray::Keyed(keyed) => {
            if let Some(known_items) = &keyed.known_items {
                let mut new_items = BTreeMap::new();
                for (key, (possibly_undefined, item_type)) in known_items {
                    let inner = extract_closure_return_type(item_type)?;
                    new_items.insert(*key, (*possibly_undefined, inner));
                }

                return Some(TUnion::from_atomic(TAtomic::Array(TArray::Keyed(TKeyedArray {
                    parameters: keyed.parameters.as_ref().map(|(k, v)| {
                        let unwrapped_v = extract_closure_return_type(v).unwrap_or_else(|| (**v).clone());
                        (Arc::clone(k), Arc::new(unwrapped_v))
                    }),
                    non_empty: keyed.non_empty,
                    known_items: Some(new_items),
                }))));
            }

            let (key_type, value_type) = get_array_parameters(array, context.codebase());
            let inner = extract_closure_return_type(&value_type)?;

            Some(TUnion::from_atomic(TAtomic::Array(TArray::Keyed(TKeyedArray {
                parameters: Some((Arc::new(key_type), Arc::new(inner))),
                non_empty: keyed.non_empty,
                known_items: None,
            }))))
        }
    }
}

/// Extracts the return type from a `Closure(): V` type.
fn extract_closure_return_type(union: &TUnion) -> Option<TUnion> {
    let mut result_types = Vec::new();

    for atomic in union.types.as_ref() {
        match atomic {
            TAtomic::Callable(TCallable::Signature(sig)) => {
                let return_type = sig.get_return_type()?;
                result_types.extend(return_type.types.iter().cloned());
            }
            _ => {
                return None;
            }
        }
    }

    if result_types.is_empty() {
        return None;
    }

    Some(TUnion::from_vec(result_types))
}
