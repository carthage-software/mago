use mago_allocator::Arena;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::Arc;

use mago_bytes::BytesDisplay;
use mago_codex::ttype::TType;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::key::ArrayKey;
use mago_codex::ttype::atomic::array::keyed::TKeyedArray;
use mago_codex::ttype::atomic::array::list::TList;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::get_int;
use mago_codex::ttype::union::TUnion;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::cst::Access;
use mago_syntax::cst::ClassLikeMemberSelector;
use mago_syntax::cst::Expression;
use mago_syntax::cst::Unset;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::utils::expression::get_block_expression_id;
use crate::utils::expression::get_root_expression_id;
use crate::visibility::ProtectedMember;
use crate::visibility::is_member_visible_from_scope;
use mago_word::concat_word;
use mago_word::word;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for Unset<'arena> {
    fn analyze<'ctx, A>(
        &'ast self,
        context: &mut Context<'ctx, 'arena, A>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError>
    where
        A: Arena,
    {
        let was_inside_unset = block_context.flags.inside_unset();
        block_context.flags.set_inside_unset(true);

        for value in &self.values {
            let was_inside_general_use = block_context.flags.inside_general_use();
            block_context.flags.set_inside_general_use(true);
            value.analyze(context, block_context, artifacts)?;
            block_context.flags.set_inside_general_use(was_inside_general_use);

            let value_id = get_block_expression_id(value, context, block_context);

            if let Some(value_id) = &value_id {
                artifacts.record_loop_assignment_target(*value_id);

                block_context.remove_variable(value_id.as_bytes(), true, context);
                block_context.possibly_assigned_variable_ids.insert(*value_id);
                block_context.references_possibly_from_confusing_scope.remove(value_id);
            }

            check_property_unset(context, block_context, artifacts, self.unset.span, value);
            forget_shape_property(context, block_context, value);

            'array_access: {
                let Expression::ArrayAccess(array_access) = value else {
                    break 'array_access;
                };

                let Some(array_id) = get_block_expression_id(array_access.array, context, block_context) else {
                    break 'array_access;
                };

                // Unsetting an offset writes the array that holds it, and the variable at its root.
                for written_id in [Some(array_id), get_root_expression_id(array_access.array)].into_iter().flatten() {
                    artifacts.record_loop_assignment_target(written_id);
                    block_context.possibly_assigned_variable_ids.insert(written_id);
                }

                let Some(key_type) = artifacts.get_expression_type(array_access.index) else {
                    break 'array_access;
                };

                let Some(array_variable) = block_context.locals.remove(&array_id).or_else(|| {
                    matches!(array_access.array, Expression::ArrayAccess(_))
                        .then(|| artifacts.get_rc_expression_type(array_access.array).cloned())
                        .flatten()
                }) else {
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
                    }

                    let TAtomic::Array(array) = atomic else {
                        atomics.push(atomic);

                        continue;
                    };

                    match array {
                        TArray::List(array) => {
                            let TList { element_type, known_elements, known_count, non_empty } = array;

                            let Some(known_elements) = known_elements else {
                                atomics.push(TAtomic::Array(TArray::Keyed(TKeyedArray {
                                    known_items: None,
                                    parameters: Some((Arc::new(get_int()), element_type)),
                                    non_empty: if let Some(known_count) = known_count {
                                        known_count > 1 // we removed 1, so we are non-empty if we had more than 1
                                    } else {
                                        false
                                    },
                                    known_non_list: true,
                                })));

                                continue;
                            };

                            let mut known_elements = Arc::unwrap_or_clone(known_elements);

                            let Some(ArrayKey::Integer(target_index)) = &array_key else {
                                if array_key.is_none() {
                                    let required_count =
                                        known_elements.values().filter(|(optional, _)| !optional).count();
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
                                                .collect::<BTreeMap<_, _>>()
                                                .into(),
                                        ),
                                        element_type,
                                        known_count,
                                        non_empty: non_empty && required_count >= 2,
                                    })));
                                } else {
                                    // Keep everything as is, attempting to remove a string key from a list
                                    // makes no sense.
                                    // An error will be emitted when we are analyzing the expression above, so ignore it here.
                                    atomics.push(TAtomic::Array(TArray::List(TList {
                                        known_elements: Some(known_elements.into()),
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

                                    if index == *target_index {
                                        element_removed = true;

                                        false
                                    } else {
                                        true
                                    }
                                }
                            });

                            if !element_removed {
                                atomics.push(TAtomic::Array(TArray::List(TList {
                                    known_elements: Some(known_elements.into()),
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
                                    known_elements: known_elements.map(Into::into),
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
                                            .collect::<BTreeMap<_, _>>()
                                            .into()
                                    }),
                                    parameters: if element_type.is_never() {
                                        None
                                    } else {
                                        Some((Arc::new(get_int()), element_type))
                                    },
                                    non_empty,
                                    known_non_list: true,
                                })));
                            }
                        }
                        TArray::Keyed(array) => {
                            let TKeyedArray { known_items, parameters, non_empty, known_non_list } = array;
                            let Some(mut known_items) = known_items else {
                                atomics.push(TAtomic::Array(TArray::Keyed(TKeyedArray {
                                    known_items: None,
                                    parameters,
                                    // We don't have any known items, so we can't unset anything.
                                    // Mark the keyed array as potentially empty.
                                    non_empty: false,
                                    known_non_list,
                                })));

                                continue;
                            };

                            let Some(array_key) = &array_key else {
                                let required_count = known_items.values().filter(|(optional, _)| !optional).count();
                                atomics.push(TAtomic::Array(TArray::Keyed(TKeyedArray {
                                    known_items: Some(
                                        // We don't know the key value, so we can't unset it.
                                        // Mark all items as potentially undefined.
                                        Arc::unwrap_or_clone(known_items)
                                            .into_iter()
                                            .map(|(key, mut item)| {
                                                // Mark the item as potentially undefined.
                                                item.0 = true;

                                                (key, item)
                                            })
                                            .collect::<BTreeMap<_, _>>()
                                            .into(),
                                    ),
                                    parameters,
                                    non_empty: non_empty && required_count >= 2,
                                    known_non_list,
                                })));

                                continue;
                            };

                            let Some(_) = Arc::make_mut(&mut known_items).remove(array_key) else {
                                atomics.push(TAtomic::Array(TArray::Keyed(TKeyedArray {
                                    known_items: Some(known_items),
                                    parameters,
                                    non_empty,
                                    known_non_list,
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
                                known_non_list,
                            })));
                        }
                    }
                }

                let rc = Rc::new(TUnion::from_vec(atomics));

                block_context.locals.insert(array_id, Rc::clone(&rc));
                block_context.remove_variable_from_conflicting_clauses(context, array_id, Some(rc.as_ref()));
                write_back_to_holding_arrays(context, block_context, artifacts, array_access.array, rc);
            };
        }

        block_context.flags.set_inside_unset(was_inside_unset);

        Ok(())
    }
}

/// Writes the new type of a nested array into the arrays that hold it, up to the root variable.
fn write_back_to_holding_arrays<A>(
    context: &Context<'_, '_, A>,
    block_context: &mut BlockContext<'_>,
    artifacts: &AnalysisArtifacts,
    mut array: &Expression<'_>,
    mut array_type: Rc<TUnion>,
) where
    A: Arena,
{
    while let Expression::ArrayAccess(holder_access) = array {
        let Some(key) = artifacts.get_expression_type(holder_access.index).and_then(TUnion::get_single_array_key)
        else {
            return;
        };

        let Some(holder_id) = get_block_expression_id(holder_access.array, context, block_context) else {
            return;
        };

        let Some(holder_type) =
            block_context.locals.get(&holder_id).or_else(|| artifacts.get_rc_expression_type(holder_access.array))
        else {
            return;
        };

        let mut holder_type = (**holder_type).clone();
        let mut updated = false;
        for atomic in holder_type.types.to_mut() {
            let item = match atomic {
                TAtomic::Array(TArray::Keyed(keyed)) => {
                    keyed.known_items.as_mut().and_then(|items| Arc::make_mut(items).get_mut(&key))
                }
                TAtomic::Array(TArray::List(list)) => match key {
                    ArrayKey::Integer(index) => usize::try_from(index).ok().and_then(|index| {
                        list.known_elements.as_mut().and_then(|elements| Arc::make_mut(elements).get_mut(&index))
                    }),
                    _ => None,
                },
                _ => None,
            };

            if let Some((_, item_type)) = item {
                *item_type = (*array_type).clone();
                updated = true;
            }
        }

        if !updated {
            return;
        }

        array_type = Rc::new(holder_type);
        block_context.locals.insert(holder_id, Rc::clone(&array_type));
        array = holder_access.array;
    }
}

fn forget_shape_property<A>(context: &Context<'_, '_, A>, block_context: &mut BlockContext<'_>, value: &Expression<'_>)
where
    A: Arena,
{
    let mut target = value;
    let mut member_type: Option<TUnion> = None;
    while let Expression::Access(Access::Property(property_access)) = target
        && let ClassLikeMemberSelector::Identifier(property_name) = &property_access.property
        && let Some(object_id) = get_block_expression_id(property_access.object, context, block_context)
        && let Some(object_type) = block_context.locals.get(&object_id)
    {
        let property_name = word(property_name.value);
        let mut object_type = (**object_type).clone();
        let mut changed = false;
        for atomic in object_type.types.to_mut() {
            let TAtomic::Object(TObject::WithProperties(shape)) = atomic else {
                continue;
            };

            match &member_type {
                None => changed |= shape.known_properties.remove(&property_name).is_some(),
                Some(member_type) => {
                    if let Some((_, property_type)) = shape.known_properties.get_mut(&property_name) {
                        *property_type = member_type.clone();
                        changed = true;
                    }
                }
            }
        }

        if !changed {
            break;
        }

        block_context.locals.insert(object_id, Rc::new(object_type.clone()));
        member_type = Some(object_type);
        target = property_access.object;
    }
}

fn check_property_unset<'ctx, 'arena, A>(
    context: &mut Context<'ctx, 'arena, A>,
    block_context: &BlockContext<'ctx>,
    artifacts: &AnalysisArtifacts,
    unset_keyword_span: Span,
    value: &Expression<'arena>,
) where
    A: Arena,
{
    let mut property = value;
    while let Expression::ArrayAccess(array_access) = property {
        property = array_access.array;
    }

    let unsets_item = !std::ptr::eq(property, value);
    let Expression::Access(Access::Property(property_access)) = property else {
        return;
    };

    let ClassLikeMemberSelector::Identifier(property_name) = &property_access.property else {
        return;
    };

    let object_type = match artifacts.get_expression_type(property_access.object) {
        Some(object_type) => object_type,
        None => match get_block_expression_id(property_access.object, context, block_context)
            .and_then(|object_id| block_context.locals.get(&object_id))
        {
            Some(object_type) => object_type.as_ref(),
            None => return,
        },
    };

    for atomic in object_type.types.as_ref() {
        let TAtomic::Object(object) = atomic else {
            continue;
        };

        let Some(class_name) = object.get_name() else {
            continue;
        };

        let qualified_property_name = concat_word!(b"$", property_name.value);
        let Some(property_metadata) =
            context.codebase.get_declaring_property(class_name.as_bytes(), qualified_property_name.as_bytes())
        else {
            continue;
        };

        let class_is_readonly =
            context.codebase.get_class_like(class_name.as_bytes()).is_some_and(|class| class.flags.is_readonly());

        let qualified = format!("{}::${}", BytesDisplay(class_name.as_bytes()), BytesDisplay(property_name.value));

        if unsets_item {
            if (property_metadata.flags.is_readonly() || class_is_readonly)
                && artifacts
                    .get_expression_type(property)
                    .is_some_and(|property_type| property_type.types.iter().all(TAtomic::is_array))
            {
                context.collector.report_with_code(
                    IssueCode::InvalidUnset,
                    Issue::error(format!("Cannot unset an item of readonly property `{qualified}`."))
                        .with_annotation(
                            Annotation::primary(value.span()).with_message("This item belongs to a readonly property."),
                        )
                        .with_annotation(Annotation::secondary(unset_keyword_span).with_message("`unset` used here."))
                        .with_note(
                            "Unsetting an item modifies the readonly property, which throws an `Error` at runtime.",
                        )
                        .with_help("Build a new array without the item where the property is initialized."),
                );
            }

            return;
        }

        if !property_metadata.flags.is_readonly()
            && !class_is_readonly
            && property_metadata.write_visibility != property_metadata.read_visibility
            && let Some(declaring_class) =
                context.codebase.get_declaring_property_class(class_name.as_bytes(), qualified_property_name.as_bytes())
            && !is_member_visible_from_scope(
                context.codebase,
                property_metadata.write_visibility,
                declaring_class.as_bytes(),
                ProtectedMember::Property(qualified_property_name),
                block_context.scope.get_class_like_name(),
            )
        {
            let visibility = property_metadata.write_visibility;
            context.collector.report_with_code(
                IssueCode::InvalidUnset,
                Issue::error(format!("Cannot unset {visibility}(set) property `{qualified}` from this scope."))
                    .with_annotation(
                        Annotation::primary(value.span())
                            .with_message(format!("This property can only be modified from {visibility} scope.")),
                    )
                    .with_annotation(Annotation::secondary(unset_keyword_span).with_message("`unset` used here."))
                    .with_note("Unsetting a property is a write, and its set visibility forbids writes here.")
                    .with_help("Remove this property from the `unset` statement, or unset it inside the class."),
            );

            return;
        }

        if property_metadata.flags.is_readonly() || class_is_readonly {
            context.collector.report_with_code(
                IssueCode::InvalidUnset,
                Issue::error(format!("Cannot unset readonly property `{qualified}`."))
                    .with_annotation(
                        Annotation::primary(value.span()).with_message("This readonly property cannot be unset."),
                    )
                    .with_annotation(Annotation::secondary(unset_keyword_span).with_message("`unset` used here."))
                    .with_note("Unsetting a readonly property throws an `Error` at runtime.")
                    .with_help("Remove this property from the `unset` statement."),
            );

            return;
        }

        if !property_metadata.hooks.is_empty() || property_metadata.flags.is_virtual_property() {
            context.collector.report_with_code(
                IssueCode::InvalidUnset,
                Issue::error(format!("Cannot unset hooked property `{qualified}`."))
                    .with_annotation(
                        Annotation::primary(value.span()).with_message("This hooked property cannot be unset."),
                    )
                    .with_annotation(Annotation::secondary(unset_keyword_span).with_message("`unset` used here."))
                    .with_note("Unsetting a property that declares hooks throws an `Error` at runtime.")
                    .with_help("Remove this property from the `unset` statement."),
            );

            return;
        }
    }
}
