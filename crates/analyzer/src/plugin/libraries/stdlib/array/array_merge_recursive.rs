//! `array_merge_recursive()` return type provider.

use std::sync::Arc;

use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::keyed::TKeyedArray;
use mago_codex::ttype::atomic::array::list::TList;
use mago_codex::ttype::combine_union_types;
use mago_codex::ttype::combiner::CombinerOptions;
use mago_codex::ttype::get_array_parameters;
use mago_codex::ttype::get_arraykey;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::union::TUnion;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::libraries::stdlib::array::ArrayMergeProvider;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::function::FunctionReturnTypeProvider;
use crate::plugin::provider::function::FunctionTarget;

static META: ProviderMeta = ProviderMeta::new(
    "php::array::array_merge_recursive",
    "array_merge_recursive",
    "Returns merged array, with values of shared string keys gathered in arrays",
);

#[derive(Default)]
pub struct ArrayMergeRecursiveProvider;

impl Provider for ArrayMergeRecursiveProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl FunctionReturnTypeProvider for ArrayMergeRecursiveProvider {
    fn targets() -> FunctionTarget {
        FunctionTarget::Exact(b"array_merge_recursive")
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        let mut arrays: Vec<&TKeyedArray> = Vec::new();
        for argument in invocation.arguments() {
            let argument_type = context.get_expression_type(argument.value()?)?;
            if argument.is_unpacked() || !argument_type.is_single() {
                return None;
            }

            match argument_type.get_single() {
                TAtomic::Array(TArray::Keyed(keyed)) => arrays.push(keyed),
                TAtomic::Array(TArray::List(_)) => {}
                _ => return None,
            }
        }

        let merged = ArrayMergeProvider.get_return_type(context, invocation)?;
        let TAtomic::Array(TArray::Keyed(keyed)) = merged.get_single() else {
            return Some(merged);
        };

        let codebase = context.codebase();
        let combine =
            |left: &TUnion, right: &TUnion| combine_union_types(left, right, codebase, CombinerOptions::default());
        let gather = |values: Vec<&TUnion>, always_shared: bool| {
            let combined = values.into_iter().cloned().reduce(|left, right| combine(&left, &right))?;
            let gathered = if combined
                .types
                .iter()
                .any(|atomic| atomic.is_keyed_array() || atomic.is_mixed() || atomic.is_generic_parameter())
            {
                TArray::Keyed(TKeyedArray::new_with_parameters(Arc::new(get_arraykey()), Arc::new(get_mixed())))
            } else {
                let mut element: Option<TUnion> = None;
                let mut non_empty = true;
                for atomic in combined.types.iter() {
                    let part = if let TAtomic::Array(array) = atomic {
                        non_empty &= array.is_non_empty();
                        get_array_parameters(array, codebase).1
                    } else {
                        TUnion::from_atomic(atomic.clone())
                    };

                    element = Some(match element {
                        Some(element) => combine(&element, &part),
                        None => part,
                    });
                }

                let mut list = TList::new(Arc::new(element?));
                list.non_empty = non_empty;
                TArray::List(list)
            };

            let gathered = TUnion::from_atomic(TAtomic::Array(gathered));
            Some(if always_shared { gathered } else { combine(&combined, &gathered) })
        };

        let mut keyed = keyed.clone();
        if let Some(items) = keyed.known_items.as_mut() {
            for (key, (_, value)) in Arc::make_mut(items).iter_mut() {
                if key.is_integer() {
                    continue;
                }

                let values: Vec<&TUnion> = arrays
                    .iter()
                    .filter_map(|array| {
                        array
                            .known_items
                            .as_ref()
                            .and_then(|items| items.get(key))
                            .map(|(_, value)| value)
                            .or_else(|| string_parameters(array))
                    })
                    .collect();

                if values.len() > 1 {
                    let always_shared = arrays
                        .iter()
                        .filter(|array| {
                            array
                                .known_items
                                .as_ref()
                                .and_then(|items| items.get(key))
                                .is_some_and(|(optional, _)| !optional)
                        })
                        .count()
                        > 1;

                    *value = gather(values, always_shared)?;
                }
            }
        }

        let parameter_values: Vec<&TUnion> = arrays.iter().filter_map(string_parameters).collect();
        if parameter_values.len() > 1
            && let Some((_, value)) = keyed.parameters.as_mut()
        {
            *value = Arc::new(combine(value, &gather(parameter_values, false)?));
        }

        Some(TUnion::from_atomic(TAtomic::Array(TArray::Keyed(keyed))))
    }
}

fn string_parameters<'array>(array: &&'array TKeyedArray) -> Option<&'array TUnion> {
    array.parameters.as_ref().filter(|(key, _)| key.has_string() || key.has_array_key()).map(|(_, value)| &**value)
}
