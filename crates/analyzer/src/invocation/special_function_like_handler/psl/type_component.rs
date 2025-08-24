use std::collections::BTreeMap;

use mago_atom::atom;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::keyed::TKeyedArray;
use mago_codex::ttype::atomic::array::list::TList;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::named::TNamedObject;
use mago_codex::ttype::get_arraykey;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::get_never;
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
            "psl\\type\\shape" => {
                let elements = get_argument(invocation.arguments_source, 0, vec!["elements"])?;
                let elements_type = artifacts.get_expression_type(elements)?;

                let argument_array = if let Some(argument_array) = elements_type.get_single_array()
                    && argument_array.is_sealed()
                {
                    argument_array
                } else {
                    return None;
                };

                let allows_unknown_elements = if let Some(argument) =
                    get_argument(invocation.arguments_source, 1, vec!["allow_unknown_fields"])
                {
                    artifacts
                        .get_expression_type(argument)
                        .and_then(|union| union.get_single_bool())
                        .filter(|boolean| !boolean.is_general())
                        .map(|boolean| boolean.is_true())?
                } else {
                    false // default to false if not provided
                };

                match argument_array {
                    TArray::List(list) => {
                        let mut known_elements = BTreeMap::new();
                        for (index, (possibly_undefined, element)) in list.known_elements.as_ref()? {
                            let inner_type = element
                                .get_single_named_object()?
                                .type_parameters
                                .as_ref()
                                .and_then(|type_parameters| type_parameters.first())
                                .cloned()?;

                            let possibly_undefined = *possibly_undefined || element.possibly_undefined;

                            known_elements.insert(*index, (possibly_undefined, inner_type));
                        }

                        Some(TUnion::from_atomic(TAtomic::Object(TObject::Named(
                            TNamedObject::new_with_type_parameters(
                                atom("Psl\\Type\\TypeInterface"),
                                Some(vec![TUnion::from_atomic(TAtomic::Array(TArray::List(TList {
                                    element_type: if allows_unknown_elements {
                                        Box::new(get_mixed())
                                    } else {
                                        Box::new(get_never())
                                    },
                                    known_count: Some(known_elements.len()),
                                    non_empty: !known_elements.is_empty(),
                                    known_elements: Some(known_elements),
                                })))]),
                            ),
                        ))))
                    }
                    TArray::Keyed(keyed_array) => {
                        let mut known_items = BTreeMap::new();
                        for (key, (possibly_undefined, item)) in keyed_array.known_items.as_ref()? {
                            let inner_type = item
                                .get_single_named_object()?
                                .type_parameters
                                .as_ref()
                                .and_then(|type_parameters| type_parameters.first())
                                .cloned()?;

                            let possibly_undefined = *possibly_undefined || item.possibly_undefined;

                            known_items.insert(*key, (possibly_undefined, inner_type));
                        }

                        Some(TUnion::from_atomic(TAtomic::Object(TObject::Named(
                            TNamedObject::new_with_type_parameters(
                                atom("Psl\\Type\\TypeInterface"),
                                Some(vec![TUnion::from_atomic(TAtomic::Array(TArray::Keyed(TKeyedArray {
                                    parameters: if allows_unknown_elements {
                                        Some((Box::new(get_arraykey()), Box::new(get_mixed())))
                                    } else {
                                        None
                                    },
                                    non_empty: !known_items.is_empty(),
                                    known_items: Some(known_items),
                                })))]),
                            ),
                        ))))
                    }
                }
            }
            _ => None,
        }
    }
}
