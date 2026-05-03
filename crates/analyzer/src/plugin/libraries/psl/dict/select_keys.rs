//! `Psl\Dict\select_keys()` return type provider.

use std::collections::BTreeMap;

use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::key::ArrayKey;
use mago_codex::ttype::atomic::array::keyed::TKeyedArray;
use mago_codex::ttype::combine_union_types;
use mago_codex::ttype::combiner::CombinerOptions;
use mago_codex::ttype::get_iterable_parameters;
use mago_codex::ttype::union::TUnion;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::function::FunctionReturnTypeProvider;
use crate::plugin::provider::function::FunctionTarget;

static META: ProviderMeta = ProviderMeta::new(
    "psl::dict::select_keys",
    "Psl\\Dict\\select_keys",
    "Returns array with only selected keys from input",
);

/// Provider for the `Psl\Dict\select_keys()` function.
///
/// Narrows the return type based on the literal keys in the `$keys` argument:
/// - If the input is a keyed array with known items AND keys are literals,
///   returns only the matching known items.
/// - If the input is a generic iterable/array and keys are literals,
///   returns a shaped array with optional entries for each key.
/// - Falls back to `None` (generic return type) otherwise.
#[derive(Default)]
pub struct SelectKeysProvider;

impl Provider for SelectKeysProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl FunctionReturnTypeProvider for SelectKeysProvider {
    fn targets() -> FunctionTarget {
        FunctionTarget::Exact("psl\\dict\\select_keys")
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        let iterable_expr = invocation.get_argument(0, &["iterable"])?;
        let keys_expr = invocation.get_argument(1, &["keys"])?;

        let iterable_type = context.get_expression_type(iterable_expr)?;
        let keys_type = context.get_expression_type(keys_expr)?;

        if !keys_type.is_single() {
            return None;
        }

        let selected_keys = extract_literal_keys(keys_type.get_single())?;
        if selected_keys.is_empty() {
            return None;
        }

        let codebase = context.codebase();

        let mut known_items: BTreeMap<ArrayKey, (bool, TUnion)> = BTreeMap::new();
        let mut generic_value_type: Option<TUnion> = None;

        for atomic in iterable_type.types.as_ref() {
            if let TAtomic::Array(TArray::Keyed(keyed)) = atomic {
                if let Some(items) = &keyed.known_items {
                    for key in &selected_keys {
                        if let Some((optional, value_type)) = items.get(key) {
                            known_items
                                .entry(*key)
                                .and_modify(|(is_optional, existing)| {
                                    *is_optional = *is_optional || *optional;
                                    *existing =
                                        combine_union_types(existing, value_type, codebase, CombinerOptions::default());
                                })
                                .or_insert_with(|| (*optional, value_type.clone()));
                        }
                    }
                }

                if let Some((_, value_param)) = &keyed.parameters {
                    generic_value_type = Some(match generic_value_type {
                        Some(existing) => {
                            combine_union_types(&existing, value_param, codebase, CombinerOptions::default())
                        }
                        None => (**value_param).clone(),
                    });
                }

                continue;
            }

            if let Some((_, value_type)) = get_iterable_parameters(atomic, codebase) {
                generic_value_type = Some(match generic_value_type {
                    Some(existing) => combine_union_types(&existing, &value_type, codebase, CombinerOptions::default()),
                    None => value_type,
                });
            }
        }

        let mut result_items: BTreeMap<ArrayKey, (bool, TUnion)> = BTreeMap::new();
        for key in &selected_keys {
            if let Some(item) = known_items.get(key) {
                result_items.insert(*key, item.clone());
            } else if let Some(value_type) = generic_value_type.as_ref() {
                result_items.insert(*key, (true, value_type.clone()));
            } else {
                // key isn't known and there's no fallback value type; drop it from the result
            }
        }

        if result_items.is_empty() {
            return None;
        }

        let mut result = TKeyedArray::new();
        result.known_items = Some(result_items);
        result.non_empty = known_items.values().any(|(optional, _)| !optional);

        Some(TUnion::from_atomic(TAtomic::Array(TArray::Keyed(result))))
    }
}

/// Extracts literal key values from a list or keyed array type (the `$keys` argument).
fn extract_literal_keys(atomic: &TAtomic) -> Option<Vec<ArrayKey>> {
    match atomic {
        TAtomic::Array(TArray::List(list)) => {
            let known_elements = list.known_elements.as_ref()?;
            let mut keys = Vec::new();

            for (_, element_type) in known_elements.values() {
                keys.push(union_to_array_key(element_type)?);
            }

            if keys.is_empty() { None } else { Some(keys) }
        }
        TAtomic::Array(TArray::Keyed(keyed)) => {
            let known_items = keyed.known_items.as_ref()?;
            let mut keys = Vec::new();

            for (_, value_type) in known_items.values() {
                keys.push(union_to_array_key(value_type)?);
            }

            if keys.is_empty() { None } else { Some(keys) }
        }
        _ => None,
    }
}

/// Converts a union type to an ArrayKey if it's a single literal string or int.
fn union_to_array_key(union: &TUnion) -> Option<ArrayKey> {
    if !union.is_single() {
        return None;
    }

    let atomic = union.get_single();
    if let Some(value) = atomic.get_literal_string_value() {
        Some(ArrayKey::String(mago_atom::atom(value)))
    } else {
        atomic.get_literal_int_value().map(ArrayKey::Integer)
    }
}
