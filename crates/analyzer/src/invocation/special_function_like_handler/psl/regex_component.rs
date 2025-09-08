use std::collections::BTreeMap;

use mago_atom::atom;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::key::ArrayKey;
use mago_codex::ttype::atomic::array::keyed::TKeyedArray;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::named::TNamedObject;
use mago_codex::ttype::get_arraykey;
use mago_codex::ttype::get_string;
use mago_codex::ttype::union::TUnion;

use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::invocation::Invocation;
use crate::invocation::special_function_like_handler::SpecialFunctionLikeHandlerTrait;
use crate::invocation::special_function_like_handler::utils::get_argument;

#[derive(Debug)]
pub struct RegexComponentFunctionsHandler;

impl SpecialFunctionLikeHandlerTrait for RegexComponentFunctionsHandler {
    fn get_return_type<'ctx, 'ast, 'arena>(
        &self,
        _context: &mut Context<'ctx, 'arena>,
        _block_context: &BlockContext<'ctx>,
        artifacts: &AnalysisArtifacts,
        function_like_name: &str,
        invocation: &Invocation<'ctx, 'ast, 'arena>,
    ) -> Option<TUnion> {
        match function_like_name {
            "psl\\regex\\capture_groups" => {
                let Some(groups) = get_argument(invocation.arguments_source, 0, vec!["groups"]) else {
                    return Some(capture_groups_fallback_type());
                };

                let Some(groups_type) = artifacts.get_expression_type(groups) else {
                    return Some(capture_groups_fallback_type());
                };

                let Some(array_atomic) = groups_type.get_single_array() else {
                    return Some(capture_groups_fallback_type());
                };

                let mut known_items = BTreeMap::from([(ArrayKey::Integer(0), (false, get_string()))]);

                let has_extra = match array_atomic {
                    TArray::Keyed(keyed_array) => {
                        let Some(groups_known_items) = keyed_array.known_items.as_ref() else {
                            return Some(capture_groups_fallback_type());
                        };

                        let mut has_unknown = false;
                        for (optional, group_known_item) in groups_known_items.values() {
                            let Some(key) = group_known_item.get_single_array_key() else {
                                has_unknown = true;
                                continue;
                            };

                            known_items.insert(key, (*optional, get_string()));
                        }

                        has_unknown || keyed_array.parameters.is_some()
                    }
                    TArray::List(list) => {
                        let Some(groups_known_elements) = list.known_elements.as_ref() else {
                            return Some(capture_groups_fallback_type());
                        };

                        let mut has_unknown = false;
                        for (optional, groups_known_element) in groups_known_elements.values() {
                            let Some(key) = groups_known_element.get_single_array_key() else {
                                has_unknown = true;
                                continue;
                            };

                            known_items.insert(key, (*optional, get_string()));
                        }

                        has_unknown || !list.element_type.is_never()
                    }
                };

                Some(TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new_with_type_parameters(
                    atom("Psl\\Type\\TypeInterface"),
                    Some(vec![TUnion::from_atomic(TAtomic::Array(TArray::Keyed(TKeyedArray {
                        parameters: if has_extra {
                            Some((Box::new(get_arraykey()), Box::new(get_string())))
                        } else {
                            None
                        },
                        non_empty: true,
                        known_items: Some(known_items),
                    })))]),
                )))))
            }
            _ => None,
        }
    }
}

fn capture_groups_fallback_type() -> TUnion {
    TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new_with_type_parameters(
        atom("Psl\\Type\\TypeInterface"),
        Some(vec![TUnion::from_atomic(TAtomic::Array(TArray::Keyed(TKeyedArray::new_with_parameters(
            Box::new(get_arraykey()),
            Box::new(get_string()),
        ))))]),
    ))))
}
