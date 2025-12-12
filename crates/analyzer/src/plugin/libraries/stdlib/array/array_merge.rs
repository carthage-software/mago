//! array_merge() return type provider.

use std::collections::BTreeMap;

use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::key::ArrayKey;
use mago_codex::ttype::atomic::array::keyed::TKeyedArray;
use mago_codex::ttype::atomic::array::list::TList;
use mago_codex::ttype::combine_union_types;
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

static TARGETS: [&str; 2] = ["array_merge", "psl\\dict\\merge"];

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
        let mut next_list_index: usize = 0;
        let mut has_parameters = false;
        let mut merged_key_type: Option<TUnion> = None;
        let mut merged_value_type: Option<TUnion> = None;
        let mut any_argument_non_empty = false;
        let mut all_arguments_are_lists = true;
        let mut all_lists_are_closed = true;

        for invocation_argument in arguments {
            if invocation_argument.is_unpacked() {
                return None;
            }

            let argument_expr = invocation_argument.value()?;
            let argument_type = context.get_expression_type(argument_expr)?;
            if !argument_type.is_single() {
                return None;
            }

            let iterable = argument_type.get_single();

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

                        if let Some(ref items) = keyed.known_items {
                            for (key, value) in items.iter() {
                                merged_items.insert(*key, value.clone());
                            }
                        }

                        if let Some((key_type, value_type)) = &keyed.parameters {
                            has_parameters = true;
                            merged_key_type = Some(match merged_key_type {
                                Some(existing) => combine_union_types(&existing, key_type, codebase, false),
                                None => (**key_type).clone(),
                            });
                            merged_value_type = Some(match merged_value_type {
                                Some(existing) => combine_union_types(&existing, value_type, codebase, false),
                                None => (**value_type).clone(),
                            });
                        }
                    }
                    TArray::List(list) => {
                        if list.non_empty {
                            any_argument_non_empty = true;
                        }

                        let is_list_closed = list.element_type.is_never();
                        if !is_list_closed {
                            all_lists_are_closed = false;
                        }

                        if let Some(ref known_elements) = list.known_elements {
                            for (idx, (optional, element_type)) in known_elements {
                                let new_idx = next_list_index + idx;
                                merged_list_elements.insert(new_idx, (*optional, element_type.clone()));
                            }
                            if let Some(max_idx) = known_elements.keys().max() {
                                next_list_index += max_idx + 1;
                            }
                        } else if list.non_empty {
                            next_list_index += 1; // At least one element
                        }

                        let (_, list_value_type) = get_array_parameters(&TArray::List(list.clone()), codebase);

                        has_parameters = true;
                        merged_value_type = Some(match merged_value_type {
                            Some(existing) => combine_union_types(&existing, &list_value_type, codebase, false),
                            None => list_value_type,
                        });

                        if !all_arguments_are_lists {
                            let key_type = get_int();
                            merged_key_type = Some(match merged_key_type {
                                Some(existing) => combine_union_types(&existing, &key_type, codebase, false),
                                None => key_type,
                            });
                        }
                    }
                }
            } else if let Some((iterable_key, iterable_value)) = get_iterable_parameters(iterable, codebase) {
                all_arguments_are_lists = false;
                has_parameters = true;
                merged_key_type = Some(match merged_key_type {
                    Some(existing) => combine_union_types(&existing, &iterable_key, codebase, false),
                    None => iterable_key,
                });
                merged_value_type = Some(match merged_value_type {
                    Some(existing) => combine_union_types(&existing, &iterable_value, codebase, false),
                    None => iterable_value,
                });
            } else {
                return None;
            }
        }

        if all_arguments_are_lists {
            let element_type =
                if all_lists_are_closed { get_never() } else { merged_value_type.unwrap_or_else(get_mixed) };

            let mut result_list = TList::new(Box::new(element_type));
            result_list.non_empty = any_argument_non_empty;

            if !merged_list_elements.is_empty() {
                result_list.known_elements = Some(merged_list_elements);
            }

            Some(TUnion::from_atomic(TAtomic::Array(TArray::List(result_list))))
        } else {
            let mut result_array = TKeyedArray::new();

            let has_merged_items = !merged_items.is_empty();
            if has_merged_items {
                result_array.known_items = Some(merged_items);
            }

            result_array.non_empty = any_argument_non_empty || has_merged_items;

            if has_parameters {
                result_array.parameters = Some((
                    Box::new(merged_key_type.unwrap_or_else(get_arraykey)),
                    Box::new(merged_value_type.unwrap_or_else(get_mixed)),
                ));
            }

            Some(TUnion::from_atomic(TAtomic::Array(TArray::Keyed(result_array))))
        }
    }
}
