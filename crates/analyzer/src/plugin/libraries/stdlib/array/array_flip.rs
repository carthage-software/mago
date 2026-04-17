//! `array_flip()` return type provider.
//!
//! Preserves the shape of an input array by swapping each key with its value: for a
//! keyed array with known literal values, the result is a keyed array whose known items
//! are the flipped pairs. Generic `array<K, V>` becomes `array<V, K>` when `V` is an
//! `array-key` subtype.

use std::collections::BTreeMap;
use std::sync::Arc;

use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::key::ArrayKey;
use mago_codex::ttype::atomic::array::keyed::TKeyedArray;
use mago_codex::ttype::atomic::array::list::TList;
use mago_codex::ttype::get_non_negative_int;
use mago_codex::ttype::union::TUnion;
use mago_codex::ttype::wrap_atomic;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::function::FunctionReturnTypeProvider;
use crate::plugin::provider::function::FunctionTarget;

static META: ProviderMeta = ProviderMeta::new(
    "php::array::array_flip",
    "array_flip",
    "Returns an array with keys and values swapped, preserving known shapes when possible",
);

/// Provider for the `array_flip()` function.
#[derive(Default)]
pub struct ArrayFlipProvider;

impl Provider for ArrayFlipProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl FunctionReturnTypeProvider for ArrayFlipProvider {
    fn targets() -> FunctionTarget {
        FunctionTarget::Exact("array_flip")
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        let array_argument = invocation.get_argument(0, &["array"])?;
        let array_type = context.get_expression_type(array_argument)?;

        if !array_type.is_single() {
            return None;
        }

        let TAtomic::Array(array) = array_type.get_single() else {
            return None;
        };

        match array {
            TArray::Keyed(keyed) => flip_keyed_array(keyed),
            TArray::List(list) => flip_list(list),
        }
    }
}

fn flip_keyed_array(keyed: &TKeyedArray) -> Option<TUnion> {
    let mut new_known_items: BTreeMap<ArrayKey, (bool, TUnion)> = BTreeMap::new();

    if let Some(items) = &keyed.known_items {
        for (key, (optional, value_type)) in items {
            let new_key = value_type.get_single_array_key()?;
            new_known_items.insert(new_key, (*optional, key.to_union()));
        }
    }

    let mut new_parameters: Option<(Arc<TUnion>, Arc<TUnion>)> = None;
    if let Some((key_param, value_param)) = &keyed.parameters {
        if !value_param.is_array_key() {
            return None;
        }

        new_parameters = Some((Arc::clone(value_param), Arc::clone(key_param)));
    }

    let mut result = TKeyedArray::new();
    if !new_known_items.is_empty() {
        result = result.with_known_items(new_known_items);
    }

    if let Some((key_type, value_type)) = new_parameters {
        result = result.with_parameters(key_type, value_type);
    }

    result.non_empty = keyed.non_empty;

    Some(wrap_atomic(TAtomic::Array(TArray::Keyed(result))))
}

fn flip_list(list: &TList) -> Option<TUnion> {
    let mut new_known_items: BTreeMap<ArrayKey, (bool, TUnion)> = BTreeMap::new();

    if let Some(elements) = &list.known_elements {
        for (idx, (optional, value_type)) in elements {
            let new_key = value_type.get_single_array_key()?;
            new_known_items.insert(new_key, (*optional, ArrayKey::Integer(*idx as i64).to_union()));
        }
    }

    let mut new_parameters: Option<(Arc<TUnion>, Arc<TUnion>)> = None;
    if !list.element_type.is_never() {
        if !list.element_type.is_array_key() {
            return None;
        }

        new_parameters = Some((Arc::clone(&list.element_type), Arc::new(get_non_negative_int())));
    }

    let mut result = TKeyedArray::new();
    if !new_known_items.is_empty() {
        result = result.with_known_items(new_known_items);
    }

    if let Some((key_type, value_type)) = new_parameters {
        result = result.with_parameters(key_type, value_type);
    }

    result.non_empty = list.non_empty;

    Some(wrap_atomic(TAtomic::Array(TArray::Keyed(result))))
}
