//! `Psl\Async\all()` return type provider.

use std::collections::BTreeMap;
use std::sync::Arc;

use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::keyed::TKeyedArray;
use mago_codex::ttype::atomic::array::list::TList;
use mago_codex::ttype::atomic::object::TObject;
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
    "psl::async::all",
    "Psl\\Async\\all",
    "Unwraps Awaitable types from array values, preserving array shape",
);

/// Provider for the `Psl\Async\all()` function.
///
/// Transforms `array<K, Awaitable<V>>` → `array<K, V>`, preserving
/// sealed array shapes, list structure, and non-empty status.
#[derive(Default)]
pub struct AllProvider;

impl Provider for AllProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl FunctionReturnTypeProvider for AllProvider {
    fn targets() -> FunctionTarget {
        FunctionTarget::Exact("psl\\async\\all")
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        let awaitables_arg = invocation.get_argument(0, &["awaitables"])?;
        let awaitables_type = context.get_expression_type(awaitables_arg)?;

        let array = awaitables_type.get_single_array()?;

        unwrap_awaitable_array(array, context)
    }
}

/// Unwraps `Awaitable<V>` to `V` for each element in an array type,
/// preserving array shape.
pub(super) fn unwrap_awaitable_array(array: &TArray, context: &ProviderContext<'_, '_, '_>) -> Option<TUnion> {
    match array {
        TArray::List(list) => Some(TUnion::from_atomic(TAtomic::Array(TArray::List(TList {
            element_type: Arc::new(if list.element_type.is_never() {
                get_never()
            } else {
                unwrap_awaitable_type(&list.element_type)?
            }),
            known_count: list.known_count,
            non_empty: list.non_empty,
            known_elements: if let Some(known_elements) = &list.known_elements {
                let mut new_elements = BTreeMap::new();
                for (index, (possibly_undefined, element_type)) in known_elements {
                    let inner = unwrap_awaitable_type(element_type)?;
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
                    let inner = unwrap_awaitable_type(item_type)?;
                    new_items.insert(*key, (*possibly_undefined, inner));
                }

                let parameters = if let Some((key_type, value_type)) = &keyed.parameters {
                    Some((key_type.clone(), Arc::new(unwrap_awaitable_type(value_type)?)))
                } else {
                    None
                };

                return Some(TUnion::from_atomic(TAtomic::Array(TArray::Keyed(TKeyedArray {
                    parameters,
                    non_empty: keyed.non_empty,
                    known_items: Some(new_items),
                }))));
            }

            let (key_type, value_type) = get_array_parameters(array, context.codebase());
            let inner = unwrap_awaitable_type(&value_type)?;

            Some(TUnion::from_atomic(TAtomic::Array(TArray::Keyed(TKeyedArray {
                parameters: Some((Arc::new(key_type), Arc::new(inner))),
                non_empty: keyed.non_empty,
                known_items: None,
            }))))
        }
    }
}

/// Extracts `V` from `Awaitable<V>`.
///
/// Looks for a named object whose name ends with `awaitable` (case-insensitive)
/// and extracts the first type parameter.
fn unwrap_awaitable_type(union: &TUnion) -> Option<TUnion> {
    let mut result_types = Vec::new();
    let mut found_awaitable = false;

    for atomic in union.types.as_ref() {
        match atomic {
            TAtomic::Object(TObject::Named(named))
                if named
                    .name
                    .strip_prefix('\\')
                    .unwrap_or(&named.name)
                    .eq_ignore_ascii_case("Psl\\Async\\Awaitable") =>
            {
                found_awaitable = true;

                if let Some(type_params) = named.type_parameters.as_ref()
                    && let Some(inner) = type_params.first()
                {
                    result_types.extend(inner.types.iter().cloned());
                } else {
                    return None;
                }
            }
            _ => {
                result_types.push(atomic.clone());
            }
        }
    }

    if !found_awaitable || result_types.is_empty() {
        return None;
    }

    Some(TUnion::from_vec(result_types))
}
