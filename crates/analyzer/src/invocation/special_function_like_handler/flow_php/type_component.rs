use std::collections::BTreeMap;

use mago_atom::atom;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::keyed::TKeyedArray;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::named::TNamedObject;
use mago_codex::ttype::get_arraykey;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::union::TUnion;

use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::invocation::Invocation;
use crate::invocation::special_function_like_handler::SpecialFunctionLikeHandlerTrait;
use crate::invocation::special_function_like_handler::utils::get_argument;

#[derive(Debug)]
pub struct TypeComponentFunctionsHandler;

impl SpecialFunctionLikeHandlerTrait for TypeComponentFunctionsHandler {
    fn get_return_type<'ctx, 'ast, 'arena>(
        &self,
        _context: &mut Context<'ctx, 'arena>,
        _block_context: &BlockContext<'ctx>,
        artifacts: &AnalysisArtifacts,
        function_like_name: &str,
        invocation: &Invocation<'ctx, 'ast, 'arena>,
    ) -> Option<TUnion> {
        match function_like_name {
            "flow\\types\\dsl\\type_structure" => {
                // Get required elements (first argument)
                let elements = get_argument(invocation.arguments_source, 0, vec!["elements"])?;
                let elements_type = artifacts.get_expression_type(elements)?;

                let elements_array = if let Some(elements_array) = elements_type.get_single_array()
                    && elements_array.is_sealed()
                {
                    elements_array
                } else {
                    return None;
                };

                // Get optional elements (second argument)
                let optional_elements_array = if let Some(optional_elements) =
                    get_argument(invocation.arguments_source, 1, vec!["optional_elements"])
                {
                    let optional_elements_type = artifacts.get_expression_type(optional_elements)?;
                    if let Some(optional_array) = optional_elements_type.get_single_array()
                        && optional_array.is_sealed()
                    {
                        Some(optional_array)
                    } else {
                        None
                    }
                } else {
                    None
                };

                // Get allow_extra flag (third argument, defaults to false)
                let allows_extra_fields =
                    if let Some(argument) = get_argument(invocation.arguments_source, 2, vec!["allow_extra"]) {
                        artifacts
                            .get_expression_type(argument)
                            .and_then(|union| union.get_single_bool())
                            .filter(|boolean| !boolean.is_general())
                            .map(|boolean| boolean.is_true())?
                    } else {
                        false // default to false if not provided
                    };

                // Flow-PHP's StructureType only supports keyed arrays (string keys), not lists
                let keyed_array = if let TArray::Keyed(keyed_array) = elements_array {
                    keyed_array
                } else {
                    return None;
                };

                let mut known_items = BTreeMap::new();
                for (key, (possibly_undefined, item)) in keyed_array.known_items.as_ref()? {
                    let inner_type = item
                        .get_single_named_object()?
                        .type_parameters
                        .as_ref()
                        .and_then(|type_parameters| type_parameters.first())
                        .cloned()?;

                    let possibly_undefined = *possibly_undefined || inner_type.possibly_undefined;

                    known_items.insert(*key, (possibly_undefined, inner_type));
                }

                // Process optional elements (possibly undefined)
                if let Some(TArray::Keyed(optional_keyed_array)) = optional_elements_array
                    && let Some(optional_items) = optional_keyed_array.known_items.as_ref()
                {
                    for (key, (_possibly_undefined, item)) in optional_items {
                        let inner_type = item
                            .get_single_named_object()?
                            .type_parameters
                            .as_ref()
                            .and_then(|type_parameters| type_parameters.first())
                            .cloned()?;

                        // Optional elements are possibly undefined
                        known_items.insert(*key, (true, inner_type));
                    }
                }

                Some(TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new_with_type_parameters(
                    atom("Flow\\Types\\Type"),
                    Some(vec![TUnion::from_atomic(TAtomic::Array(TArray::Keyed(TKeyedArray {
                        parameters: if allows_extra_fields {
                            Some((Box::new(get_arraykey()), Box::new(get_mixed())))
                        } else {
                            None
                        },
                        non_empty: !known_items.is_empty(),
                        known_items: Some(known_items),
                    })))]),
                )))))
            }

            _ => None,
        }
    }
}
