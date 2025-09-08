use std::rc::Rc;

use mago_codex::ttype::TType;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::key::ArrayKey;
use mago_codex::ttype::atomic::array::keyed::TKeyedArray;
use mago_codex::ttype::atomic::array::list::TList;
use mago_codex::ttype::get_int;
use mago_codex::ttype::union::TUnion;
use mago_reporting::*;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::utils::expression::get_expression_id;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for Unset<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let was_inside_unset = block_context.inside_unset;
        block_context.inside_unset = true;

        for value in self.values.iter() {
            let was_inside_general_use = block_context.inside_general_use;
            block_context.inside_general_use = true;
            value.analyze(context, block_context, artifacts)?;
            block_context.inside_general_use = was_inside_general_use;

            let value_id = get_expression_id(
                value,
                block_context.scope.get_class_like_name(),
                context.resolved_names,
                Some(context.codebase),
            );

            if let Some(value_id) = &value_id {
                block_context.remove_variable(value_id, true, context);
                block_context.references_possibly_from_confusing_scope.remove(value_id);
            }

            'array_access: {
                let Expression::ArrayAccess(array_access) = value else {
                    break 'array_access;
                };

                let Some(array_id) = get_expression_id(
                    array_access.array,
                    block_context.scope.get_class_like_name(),
                    context.resolved_names,
                    Some(context.codebase),
                ) else {
                    break 'array_access;
                };

                let Some(key_type) = artifacts.get_expression_type(array_access.index) else {
                    break 'array_access;
                };

                let Some(array_variable) = block_context.locals.remove(&array_id) else {
                    break 'array_access;
                };

                let mut atomics = vec![];

                let array_key = key_type.get_single_array_key();
                for atomic in Rc::unwrap_or_clone(array_variable).types.into_owned() {
                    if let TAtomic::Scalar(scalar) = &atomic {
                        let scalar_str = scalar.get_id();

                        context.collector.report_with_code(
                            IssueCode::InvalidUnset,
                            Issue::error(format!(
                                "Cannot apply `unset` to an offset of non-array type `{scalar_str}`."
                            ))
                            .with_annotation(
                                Annotation::primary(array_access.array.span())
                                    .with_message(format!("This has type `{scalar_str}`, not an array."))
                            )
                            .with_annotation(
                                Annotation::secondary(self.unset.span)
                                    .with_message("`unset` used here on a non-array type.")
                            )
                            .with_note(
                                "`unset` on an array offset requires the base variable to be an array or an object that implements `ArrayAccess`."
                            )
                            .with_note(
                                format!("Using `unset` on an offset of type `{scalar_str}` will cause a runtime error.")
                            )
                            .with_help(
                                "Ensure the variable is an array before attempting to unset an element."
                            ),
                        );

                        atomics.push(atomic);
                        continue;
                    };

                    let TAtomic::Array(array) = atomic else {
                        atomics.push(atomic);

                        continue;
                    };

                    match array {
                        TArray::List(array) => {
                            let TList { element_type, known_elements, known_count, non_empty } = array;

                            let Some(mut known_elements) = known_elements else {
                                atomics.push(TAtomic::Array(TArray::Keyed(TKeyedArray {
                                    known_items: None,
                                    parameters: Some((Box::new(get_int()), element_type)),
                                    non_empty: if let Some(known_count) = known_count {
                                        known_count > 1 // we removed 1, so we are non-empty if we had more than 1
                                    } else {
                                        false
                                    },
                                })));

                                continue;
                            };

                            let Some(ArrayKey::Integer(target_index)) = &array_key else {
                                if array_key.is_none() {
                                    atomics.push(TAtomic::Array(TArray::List(TList {
                                        known_elements: Some(
                                            // We don't know the key value, so we can't unset it.
                                            // Mark all items as potentially undefined.
                                            known_elements
                                                .into_iter()
                                                .map(|(index, mut element)| {
                                                    // Mark the item as potentially undefined.
                                                    element.0 = true;

                                                    (index, element)
                                                })
                                                .collect(),
                                        ),
                                        element_type,
                                        known_count,
                                        // Mark the list as potentially undefined.
                                        non_empty: true,
                                    })));
                                } else {
                                    // Keep everything as is, attempting to remove a string key from a list
                                    // makes no sense.
                                    // An error will be emitted when we are analyzing the expression above, so ignore it here.
                                    atomics.push(TAtomic::Array(TArray::List(TList {
                                        known_elements: Some(known_elements),
                                        element_type,
                                        known_count,
                                        non_empty,
                                    })));
                                }

                                continue;
                            };

                            let is_fixed_list = element_type.is_never();
                            let maintain_list = is_fixed_list && {
                                let elements_count = known_elements.len();
                                let last_index = elements_count - 1;

                                *target_index == (last_index as i64)
                            };

                            let mut element_removed = false;
                            known_elements.retain(|index, _| {
                                if *target_index < 0 {
                                    // this is a negative index, which means we can't remove it
                                    // from the list
                                    true
                                } else {
                                    let index = *index as i64;

                                    if index != *target_index {
                                        true
                                    } else {
                                        element_removed = true;

                                        false
                                    }
                                }
                            });

                            if !element_removed {
                                atomics.push(TAtomic::Array(TArray::List(TList {
                                    known_elements: Some(known_elements),
                                    element_type,
                                    known_count,
                                    non_empty,
                                })));

                                continue;
                            }

                            let non_empty = !known_elements.is_empty();
                            let known_count = Some(known_elements.len());
                            let known_elements = if known_elements.is_empty() {
                                // Completely empty now.
                                None
                            } else {
                                Some(known_elements)
                            };

                            if maintain_list {
                                atomics.push(TAtomic::Array(TArray::List(TList {
                                    known_count,
                                    non_empty,
                                    known_elements,
                                    element_type,
                                })));
                            } else {
                                // we removed an integer index from a list such as:
                                // `list{1, 2, 3, ...<int>}`, which means the keys are no longer sequential.
                                // this makes the list no longer a list array, but rather a keyed array.
                                // we need to convert the list into a keyed array.
                                atomics.push(TAtomic::Array(TArray::Keyed(TKeyedArray {
                                    known_items: known_elements.map(|known_elements| {
                                        known_elements
                                            .into_iter()
                                            .map(|(index, element)| (ArrayKey::Integer(index as i64), element))
                                            .collect()
                                    }),
                                    parameters: if element_type.is_never() {
                                        None
                                    } else {
                                        Some((Box::new(get_int()), element_type))
                                    },
                                    non_empty,
                                })));
                            }
                        }
                        TArray::Keyed(array) => {
                            let TKeyedArray { known_items, parameters, non_empty } = array;
                            let Some(mut known_items) = known_items else {
                                atomics.push(TAtomic::Array(TArray::Keyed(TKeyedArray {
                                    known_items: None,
                                    parameters,
                                    // We don't have any known items, so we can't unset anything.
                                    // Mark the keyed array as potentially empty.
                                    non_empty: false,
                                })));

                                continue;
                            };

                            let Some(array_key) = &array_key else {
                                atomics.push(TAtomic::Array(TArray::Keyed(TKeyedArray {
                                    known_items: Some(
                                        // We don't know the key value, so we can't unset it.
                                        // Mark all items as potentially undefined.
                                        known_items
                                            .into_iter()
                                            .map(|(key, mut item)| {
                                                // Mark the item as potentially undefined.
                                                item.0 = true;

                                                (key, item)
                                            })
                                            .collect(),
                                    ),
                                    parameters,
                                    non_empty,
                                })));

                                continue;
                            };

                            let Some(_) = known_items.remove(array_key) else {
                                atomics.push(TAtomic::Array(TArray::Keyed(TKeyedArray {
                                    known_items: Some(known_items),
                                    parameters,
                                    non_empty,
                                })));

                                continue;
                            };

                            let (known_items, non_empty) = if known_items.is_empty() {
                                // Completely empty now.
                                (None, false)
                            } else {
                                (Some(known_items), true)
                            };

                            atomics.push(TAtomic::Array(TArray::Keyed(TKeyedArray {
                                known_items,
                                parameters,
                                non_empty,
                            })));
                        }
                    }
                }

                let rc = Rc::new(TUnion::from_vec(atomics));

                block_context.locals.insert(array_id.clone(), rc.clone());
                block_context.remove_variable_from_conflicting_clauses(context, &array_id, Some(rc.as_ref()));
            };
        }

        block_context.inside_unset = was_inside_unset;

        Ok(())
    }
}
