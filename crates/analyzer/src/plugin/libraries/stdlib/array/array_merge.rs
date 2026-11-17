//! `array_merge()` return type provider.

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::sync::Arc;

use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::key::ArrayKey;
use mago_codex::ttype::atomic::array::keyed::TKeyedArray;
use mago_codex::ttype::atomic::array::list::TList;
use mago_codex::ttype::combine_union_types;
use mago_codex::ttype::combiner::CombinerOptions;
use mago_codex::ttype::get_array_parameters;
use mago_codex::ttype::get_arraykey;
use mago_codex::ttype::get_int;
use mago_codex::ttype::get_iterable_parameters;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::get_never;
use mago_codex::ttype::union::TUnion;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::function::FunctionReturnTypeProvider;
use crate::plugin::provider::function::FunctionTarget;

static META: ProviderMeta =
    ProviderMeta::new("php::array::array_merge", "array_merge", "Returns merged array with combined types");

static TARGETS: [&[u8]; 2] = [b"array_merge", b"psl\\dict\\merge"];

/// Provider for the `array_merge()` and `Psl\Dict\merge()` functions.
///
/// Returns an array with types combined from all input arrays.
#[derive(Default)]
pub struct ArrayMergeProvider;

impl Provider for ArrayMergeProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl FunctionReturnTypeProvider for ArrayMergeProvider {
    fn targets() -> FunctionTarget {
        FunctionTarget::ExactMultiple(&TARGETS)
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        let arguments = invocation.arguments();
        if arguments.is_empty() {
            return None;
        }

        let codebase = context.codebase();

        let mut merged_items: BTreeMap<ArrayKey, (bool, TUnion)> = BTreeMap::new();
        let mut merged_list_elements: BTreeMap<usize, (bool, TUnion)> = BTreeMap::new();
        // The exact number of elements before the next argument, while it is known.
        let mut next_list_index: Option<usize> = Some(0);
        let mut all_keys_are_int = true;
        let last_argument_index = arguments.len() - 1;
        let mut has_parameters = false;
        let mut merged_key_type: Option<TUnion> = None;
        let mut merged_value_type: Option<TUnion> = None;
        let mut any_argument_non_empty = false;
        let mut all_arguments_are_lists = true;
        let mut all_lists_are_closed = true;
        let mut next_int_key: Option<i64> = Some(0);

        for (argument_index, invocation_argument) in arguments.iter().enumerate() {
            let argument_expr = invocation_argument.value()?;
            let argument_type = context.get_expression_type(argument_expr)?;

            if !argument_type.is_single() {
                return None;
            }

            let argument_type = if invocation_argument.is_unpacked() {
                let inner = get_iterable_parameters(argument_type.get_single(), codebase);
                let (_, value_type) = inner?;
                if !value_type.is_single() {
                    return None;
                }

                if argument_type.get_single().is_non_empty_list() || argument_type.get_single().is_non_empty_array() {
                    any_argument_non_empty = true;
                }

                Cow::Owned(value_type)
            } else {
                Cow::Borrowed(argument_type)
            };

            let mixed_array;
            let iterable = if argument_type.get_single().is_mixed() {
                mixed_array = TAtomic::Array(TArray::Keyed(TKeyedArray::new_with_parameters(
                    Arc::new(get_arraykey()),
                    Arc::new(get_mixed()),
                )));
                &mixed_array
            } else {
                argument_type.get_single()
            };

            if let TAtomic::Array(array) = iterable {
                match array {
                    TArray::Keyed(keyed) => {
                        let is_empty_array = keyed.known_items.is_none() && keyed.parameters.is_none();

                        if !is_empty_array {
                            all_arguments_are_lists = false;
                        }

                        if keyed.non_empty {
                            any_argument_non_empty = true;
                        }

                        all_keys_are_int &= keyed
                            .known_items
                            .as_deref()
                            .is_none_or(|items| items.keys().all(|key| matches!(key, ArrayKey::Integer(_))))
                            && keyed.parameters.as_ref().is_none_or(|(key_type, _)| key_type.is_int());

                        if let Some(items) = keyed.known_items.as_deref() {
                            for (key, (optional, value)) in items {
                                if let ArrayKey::Integer(_) = key {
                                    match next_int_key {
                                        Some(index) if !optional => {
                                            merged_items.insert(ArrayKey::Integer(index), (false, value.clone()));
                                            next_int_key = Some(index + 1);
                                        }
                                        _ => {
                                            next_int_key = None;
                                            has_parameters = true;
                                            merged_key_type = Some(match merged_key_type {
                                                Some(existing) => combine_union_types(
                                                    &existing,
                                                    &get_int(),
                                                    codebase,
                                                    CombinerOptions::default(),
                                                ),
                                                None => get_int(),
                                            });
                                            merged_value_type = Some(match merged_value_type {
                                                Some(existing) => combine_union_types(
                                                    &existing,
                                                    value,
                                                    codebase,
                                                    CombinerOptions::default(),
                                                ),
                                                None => value.clone(),
                                            });
                                        }
                                    }

                                    continue;
                                }

                                let merged_value = match merged_items.get(key) {
                                    Some((earlier_optional, earlier)) if *optional => (
                                        *earlier_optional,
                                        combine_union_types(earlier, value, codebase, CombinerOptions::default()),
                                    ),
                                    _ => (*optional, value.clone()),
                                };

                                merged_items.insert(*key, merged_value);
                            }
                        }

                        if let Some((key_type, value_type)) = &keyed.parameters {
                            if key_type.has_int() {
                                next_int_key = None;
                            }

                            if key_type.has_string() || key_type.is_array_key() {
                                for (key, (_, existing)) in merged_items.iter_mut() {
                                    if !matches!(key, ArrayKey::Integer(_)) {
                                        *existing = combine_union_types(
                                            existing,
                                            value_type,
                                            codebase,
                                            CombinerOptions::default(),
                                        );
                                    }
                                }
                            }

                            has_parameters = true;
                            merged_key_type = Some(match merged_key_type {
                                Some(existing) => {
                                    combine_union_types(&existing, key_type, codebase, CombinerOptions::default())
                                }
                                None => (**key_type).clone(),
                            });
                            merged_value_type = Some(match merged_value_type {
                                Some(existing) => {
                                    combine_union_types(&existing, value_type, codebase, CombinerOptions::default())
                                }
                                None => (**value_type).clone(),
                            });
                        }
                    }
                    TArray::List(list) => {
                        if list.non_empty {
                            any_argument_non_empty = true;
                        }

                        next_int_key = match (next_int_key, list.known_elements.as_deref()) {
                            (Some(index), Some(known_elements))
                                if list.element_type.is_never()
                                    && known_elements.values().all(|(optional, _)| !optional) =>
                            {
                                Some(index + known_elements.len() as i64)
                            }
                            (Some(index), None) if list.element_type.is_never() => Some(index),
                            _ => None,
                        };

                        let is_list_closed = list.element_type.is_never();
                        if !is_list_closed {
                            all_lists_are_closed = false;
                        }

                        // Elements keep their positions only behind a prefix of known length, and an optional
                        // element is only placed when nothing follows it.
                        let mut placed_every_element = true;
                        if let Some(known_elements) = list.known_elements.as_deref() {
                            for (idx, (optional, element_type)) in known_elements {
                                match next_list_index {
                                    Some(start) if !*optional || argument_index == last_argument_index => {
                                        merged_list_elements.insert(start + idx, (*optional, element_type.clone()));
                                    }
                                    _ => placed_every_element = false,
                                }
                            }
                        }

                        // Elements without a fixed position live in the rest of the merged list.
                        if !placed_every_element {
                            all_lists_are_closed = false;
                        }

                        next_list_index = match (next_list_index, list.known_elements.as_deref()) {
                            (Some(start), Some(known_elements))
                                if is_list_closed && known_elements.values().all(|(optional, _)| !optional) =>
                            {
                                Some(start + known_elements.len())
                            }
                            (Some(start), None) if is_list_closed => Some(start),
                            _ => None,
                        };

                        let (_, list_value_type) = get_array_parameters(&TArray::List(list.clone()), codebase);

                        has_parameters = true;
                        merged_value_type = Some(match merged_value_type {
                            Some(existing) => {
                                combine_union_types(&existing, &list_value_type, codebase, CombinerOptions::default())
                            }
                            None => list_value_type,
                        });

                        if !all_arguments_are_lists {
                            let key_type = get_int();
                            merged_key_type = Some(match merged_key_type {
                                Some(existing) => {
                                    combine_union_types(&existing, &key_type, codebase, CombinerOptions::default())
                                }
                                None => key_type,
                            });
                        }
                    }
                }
            } else if let Some((iterable_key, iterable_value)) = get_iterable_parameters(iterable, codebase) {
                all_arguments_are_lists = false;
                all_keys_are_int = false;
                has_parameters = true;
                merged_key_type = Some(match merged_key_type {
                    Some(existing) => {
                        combine_union_types(&existing, &iterable_key, codebase, CombinerOptions::default())
                    }
                    None => iterable_key,
                });
                merged_value_type = Some(match merged_value_type {
                    Some(existing) => {
                        combine_union_types(&existing, &iterable_value, codebase, CombinerOptions::default())
                    }
                    None => iterable_value,
                });
            } else {
                return None;
            }
        }

        if all_arguments_are_lists {
            let element_type =
                if all_lists_are_closed { get_never() } else { merged_value_type.unwrap_or_else(get_mixed) };

            let mut result_list = TList::new(Arc::new(element_type));
            result_list.non_empty = any_argument_non_empty;

            if !merged_list_elements.is_empty() {
                result_list.known_elements = Some(merged_list_elements.into());
            }

            Some(TUnion::from_atomic(TAtomic::Array(TArray::List(result_list))))
        } else if all_keys_are_int {
            // PHP renumbers integer keys, so merging arrays with only integer keys gives a list.
            let element_type = merged_items
                .into_values()
                .map(|(_, value)| value)
                .chain(merged_value_type)
                .reduce(|first, second| combine_union_types(&first, &second, codebase, CombinerOptions::default()))
                .unwrap_or_else(get_never);

            let mut result_list = TList::new(Arc::new(element_type));
            result_list.non_empty = any_argument_non_empty;

            Some(TUnion::from_atomic(TAtomic::Array(TArray::List(result_list))))
        } else {
            let mut result_array = TKeyedArray::new();

            let has_merged_items = !merged_items.is_empty();
            if has_merged_items {
                result_array.known_items = Some(merged_items.into());
            }

            result_array.non_empty = any_argument_non_empty || has_merged_items;

            if has_parameters {
                result_array.parameters = Some((
                    Arc::new(merged_key_type.unwrap_or_else(get_arraykey)),
                    Arc::new(merged_value_type.unwrap_or_else(get_mixed)),
                ));
            }

            Some(TUnion::from_atomic(TAtomic::Array(TArray::Keyed(result_array))))
        }
    }
}
