use std::collections::BTreeMap;
use std::sync::Arc;

use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::list::TList;
use mago_codex::ttype::combine_union_types;
use mago_codex::ttype::combiner::CombinerOptions;
use mago_codex::ttype::get_array_parameters;
use mago_codex::ttype::union::TUnion;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::function::FunctionReturnTypeProvider;
use crate::plugin::provider::function::FunctionTarget;

static META: ProviderMeta = ProviderMeta::new(
    "php::array::array_pad",
    "array_pad",
    "Preserves list shapes and known padding lengths through array_pad",
);

const MAX_TRACKED_ELEMENTS: usize = 128;

#[derive(Default)]
pub struct ArrayPadProvider;

impl Provider for ArrayPadProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl FunctionReturnTypeProvider for ArrayPadProvider {
    fn targets() -> FunctionTarget {
        FunctionTarget::Exact(b"array_pad")
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        let array = invocation.get_argument(0, &[b"array"])?;
        let length = invocation.get_argument(1, &[b"length"])?;
        let value = invocation.get_argument(2, &[b"value"])?;

        let array_type = context.get_expression_type(array)?;
        let TArray::List(list) = array_type.get_single_array()? else {
            return None;
        };

        let length = context.get_expression_type(length)?.get_single_literal_int_value()?;
        let target_count = usize::try_from(length.unsigned_abs()).ok()?;
        let array = TArray::List(list.clone());

        if target_count <= array.get_minimum_size() {
            return Some(array_type.clone());
        }

        if target_count > MAX_TRACKED_ELEMENTS {
            return None;
        }

        let padding_type = context.get_expression_type(value)?;
        let (_, input_element_type) = get_array_parameters(&array, context.codebase());
        let element_type =
            combine_union_types(&input_element_type, padding_type, context.codebase(), CombinerOptions::default());
        let (result_elements, known_count) = match list.known_count {
            Some(known_count) => {
                let padding_count = target_count - known_count;
                let mut result_elements = BTreeMap::new();

                if length < 0 {
                    for index in 0..padding_count {
                        result_elements.insert(index, (false, padding_type.clone()));
                    }

                    for index in 0..known_count {
                        let input_type = list
                            .known_elements
                            .as_ref()
                            .and_then(|elements| elements.get(&index))
                            .map_or_else(|| input_element_type.clone(), |(_, ty)| ty.clone());
                        result_elements.insert(index + padding_count, (false, input_type));
                    }
                } else {
                    for index in 0..known_count {
                        let input_type = list
                            .known_elements
                            .as_ref()
                            .and_then(|elements| elements.get(&index))
                            .map_or_else(|| input_element_type.clone(), |(_, ty)| ty.clone());
                        result_elements.insert(index, (false, input_type));
                    }

                    for index in known_count..target_count {
                        result_elements.insert(index, (false, padding_type.clone()));
                    }
                }

                (result_elements, Some(target_count))
            }
            None => {
                let mut result_elements = list.known_elements.clone().unwrap_or_default();

                for index in 0..target_count {
                    let result_type = if length >= 0 {
                        result_elements.get(&index).and_then(|(optional, ty)| (!optional).then(|| ty.clone()))
                    } else {
                        None
                    };

                    result_elements.insert(index, (false, result_type.unwrap_or_else(|| element_type.clone())));
                }

                (result_elements, None)
            }
        };

        Some(TUnion::from_atomic(TAtomic::Array(TArray::List(TList {
            element_type: Arc::new(element_type),
            known_elements: Some(result_elements),
            known_count,
            non_empty: true,
        }))))
    }
}
