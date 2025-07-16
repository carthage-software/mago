use std::collections::BTreeMap;
use std::rc::Rc;

use mago_codex::data_flow::graph::GraphKind;
use mago_codex::data_flow::node::DataFlowNode;
use mago_codex::data_flow::node::DataFlowNodeKind;
use mago_codex::data_flow::path::ArrayDataKind;
use mago_codex::data_flow::path::PathKind;
use mago_codex::misc::VariableIdentifier;
use mago_codex::ttype::add_union_type;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::key::ArrayKey;
use mago_codex::ttype::atomic::array::keyed::TKeyedArray;
use mago_codex::ttype::atomic::array::list::TList;
use mago_codex::ttype::atomic::generic::TGenericParameter;
use mago_codex::ttype::combiner;
use mago_codex::ttype::get_arraykey;
use mago_codex::ttype::get_int;
use mago_codex::ttype::get_iterable_parameters;
use mago_codex::ttype::get_mixed_any;
use mago_codex::ttype::get_never;
use mago_codex::ttype::union::TUnion;
use mago_codex::ttype::wrap_atomic;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::expression::assignment::property_assignment;
use crate::utils::expression::array::ArrayTarget;
use crate::utils::expression::array::get_array_target_type_given_index;
use crate::utils::expression::get_expression_id;
use crate::utils::expression::get_index_id;

pub(crate) fn analyze<'a>(
    context: &mut Context<'a>,
    block_context: &mut BlockContext<'a>,
    artifacts: &mut AnalysisArtifacts,
    array_target: ArrayTarget<'_>,
    assign_value_type: TUnion,
) -> Result<(), AnalysisError> {
    let mut array_target_expressions = Vec::new();
    let mut current_target = array_target;

    loop {
        array_target_expressions.push(current_target);

        let next_inner_target = match current_target.get_array() {
            Expression::ArrayAccess(array_access) => ArrayTarget::Access(array_access),
            Expression::ArrayAppend(array_append) => ArrayTarget::Append(array_append),
            _ => break,
        };

        current_target = next_inner_target;
    }

    let root_array_expression = current_target.get_array();

    let was_inside_general_use = block_context.inside_general_use;
    block_context.inside_general_use = true;
    root_array_expression.analyze(context, block_context, artifacts)?;
    block_context.inside_general_use = was_inside_general_use;

    let mut root_array_type = artifacts.get_expression_type(root_array_expression).cloned().unwrap_or(get_mixed_any());

    if root_array_type.is_mixed() {
        let was_inside_general_use = block_context.inside_general_use;
        block_context.inside_general_use = true;
        array_target.get_array().analyze(context, block_context, artifacts)?;
        if let Some(index) = array_target.get_index() {
            index.analyze(context, block_context, artifacts)?;
        }
        block_context.inside_general_use = was_inside_general_use;
    }

    let mut current_type = root_array_type.clone();

    let root_var_id = get_expression_id(
        root_array_expression,
        block_context.scope.get_class_like_name(),
        context.resolved_names,
        context.interner,
        Some(context.codebase),
    );

    let current_index = analyze_nested_array_assignment(
        context,
        block_context,
        artifacts,
        array_target_expressions,
        assign_value_type,
        root_var_id.clone(),
        &mut root_array_type,
        &mut current_type,
    )?;

    if artifacts.data_flow_graph.kind == GraphKind::FunctionBody
        && let Some(root_var_id) = &root_var_id
        && let Expression::Variable(_) = &root_array_expression
    {
        let interner = context.interner;
        artifacts.data_flow_graph.add_node(DataFlowNode::get_for_variable_source(
            VariableIdentifier(interner.intern(root_var_id)),
            root_array_expression.span(),
            false,
            false,
            false,
        ));
    }

    let root_is_string = root_array_type.has_string();

    let mut key_values = Vec::new();

    let index_type = current_index.map(|current_index| {
        artifacts.get_rc_expression_type(current_index).cloned().unwrap_or(Rc::new(get_arraykey()))
    });

    if let Some(index_type) = &index_type {
        for index_atomic_type in &index_type.types {
            if index_atomic_type.is_literal_int() || index_atomic_type.is_known_literal_string() {
                key_values.push(index_atomic_type.clone());
            }
        }
    }

    root_array_type = if !key_values.is_empty() {
        update_type_with_key_values(context, root_array_type, current_type, &key_values, index_type)
    } else if !root_is_string {
        update_array_assignment_child_type(context, block_context, index_type, current_type, root_array_type)
    } else {
        root_array_type
    };

    if let Expression::Access(Access::Property(property_access)) = &root_array_expression {
        property_assignment::analyze(
            context,
            block_context,
            artifacts,
            property_access,
            &root_array_type,
            Some(root_array_expression.span()),
        )?;
    }

    let root_array_type = Rc::new(root_array_type);
    if let Some(root_var_id) = &root_var_id {
        block_context.locals.insert(root_var_id.clone(), root_array_type.clone());
    }

    artifacts.set_rc_expression_type(&root_array_expression, root_array_type);

    Ok(())
}

pub(crate) fn update_type_with_key_values(
    context: &Context<'_>,
    mut new_type: TUnion,
    current_type: TUnion,
    key_values: &Vec<TAtomic>,
    key_type: Option<Rc<TUnion>>,
) -> TUnion {
    let mut has_matching_item = false;

    new_type.types = new_type
        .types
        .into_iter()
        .map(|atomic_type| {
            update_atomic_given_key(
                context,
                atomic_type,
                key_values,
                key_type.clone(),
                &mut has_matching_item,
                &current_type,
            )
        })
        .collect();

    new_type
}

fn update_atomic_given_key(
    context: &Context<'_>,
    mut atomic_type: TAtomic,
    key_values: &Vec<TAtomic>,
    key_type: Option<Rc<TUnion>>,
    has_matching_item: &mut bool,
    current_type: &TUnion,
) -> TAtomic {
    if let TAtomic::GenericParameter(TGenericParameter { constraint, .. }) = &atomic_type
        && constraint.types.len() == 1
    {
        return update_atomic_given_key(
            context,
            constraint.types[0].clone(),
            key_values,
            key_type,
            has_matching_item,
            current_type,
        );
    }

    if !key_values.is_empty() {
        for key_value in key_values {
            if let TAtomic::Array(array) = &mut atomic_type {
                let array_key = if let Some(str) = key_value.get_literal_string_value() {
                    ArrayKey::String(str.to_owned())
                } else if let Some(int) = key_value.get_literal_int_value() {
                    ArrayKey::Integer(int)
                } else {
                    continue;
                };

                match array {
                    TArray::List(list) => {
                        if let ArrayKey::Integer(key_value) = array_key {
                            *has_matching_item = true;

                            if let Some(known_elements) = list.known_elements.as_mut() {
                                if let Some((pu, entry)) = known_elements.get_mut(&(key_value as usize)) {
                                    *entry = current_type.clone();
                                    *pu = false;
                                } else {
                                    known_elements.insert(key_value as usize, (false, current_type.clone()));
                                }
                            } else {
                                list.known_elements =
                                    Some(BTreeMap::from([(key_value as usize, (false, current_type.clone()))]));
                            }

                            list.non_empty = true;
                        }
                    }
                    TArray::Keyed(keyed_array) => {
                        *has_matching_item = true;

                        if let Some(known_items) = keyed_array.known_items.as_mut() {
                            if let Some((pu, entry)) = known_items.get_mut(&array_key) {
                                *entry = current_type.clone();
                                *pu = false;
                            } else {
                                known_items.insert(array_key, (false, current_type.clone()));
                            }
                        } else {
                            keyed_array.known_items =
                                Some(BTreeMap::from([(array_key, (false, current_type.clone()))]));
                        }

                        keyed_array.non_empty = true;
                    }
                }
            }
        }
    } else {
        let Some((array_key_type, array_value_type)) =
            get_iterable_parameters(&atomic_type, context.codebase, context.interner)
        else {
            return atomic_type;
        };

        let TAtomic::Array(array) = &mut atomic_type else {
            return atomic_type;
        };

        let combined_value_type =
            add_union_type(array_value_type, current_type, context.codebase, context.interner, false);

        if array.is_empty() && key_type.is_none() {
            *array = TArray::List(TList {
                element_type: Box::new(combined_value_type),
                known_elements: None,
                known_count: None,
                non_empty: true,
            });
        } else {
            match array {
                TArray::List(list) => {
                    list.element_type = Box::new(combined_value_type);

                    list.known_elements = None;
                    list.known_count = None;
                    list.non_empty = true;
                }
                TArray::Keyed(keyed_array) => {
                    keyed_array.parameters = Some((
                        Box::new(add_union_type(
                            array_key_type,
                            &key_type.unwrap_or_else(|| Rc::new(get_int())),
                            context.codebase,
                            context.interner,
                            false,
                        )),
                        Box::new(combined_value_type),
                    ));

                    keyed_array.known_items = None;
                    keyed_array.non_empty = true;
                }
            }
        }
    }

    atomic_type
}

fn add_array_assignment_dataflow(
    context: &mut Context<'_>,
    artifacts: &mut AnalysisArtifacts,
    expr_var_pos: &Span,
    mut parent_expr_type: TUnion,
    child_expr_type: &TUnion,
    var_var_id: Option<String>,
    key_values: &Vec<TAtomic>,
    inside_general_use: bool,
) -> TUnion {
    let parent_node = if let Some(var_var_id) = var_var_id {
        if let Some(var_id) = context.interner.get(&var_var_id) {
            DataFlowNode::get_for_lvar(VariableIdentifier(var_id), *expr_var_pos)
        } else {
            DataFlowNode::get_for_array_assignment(*expr_var_pos)
        }
    } else {
        DataFlowNode::get_for_array_assignment(*expr_var_pos)
    };

    if inside_general_use && artifacts.data_flow_graph.kind == GraphKind::FunctionBody {
        let assignment_node = DataFlowNode {
            id: parent_node.id.clone(),
            kind: DataFlowNodeKind::VariableUseSink { span: *expr_var_pos },
        };

        artifacts.data_flow_graph.add_path(&parent_node, &assignment_node, PathKind::Default);

        artifacts.data_flow_graph.add_node(assignment_node);
    }

    artifacts.data_flow_graph.add_node(parent_node.clone());

    let old_parent_nodes = parent_expr_type.parent_nodes.clone();

    parent_expr_type.parent_nodes = vec![parent_node.clone()];

    for old_parent_node in old_parent_nodes {
        artifacts.data_flow_graph.add_path(&old_parent_node, &parent_node, PathKind::Default);
    }

    for child_parent_node in &child_expr_type.parent_nodes {
        if !key_values.is_empty() {
            for key_value in key_values {
                let key_value = if let Some(str) = key_value.get_literal_string_value() {
                    str.to_owned()
                } else if let Some(int) = key_value.get_literal_int_value() {
                    int.to_string()
                } else {
                    continue;
                };

                artifacts.data_flow_graph.add_path(
                    child_parent_node,
                    &parent_node,
                    PathKind::ArrayAssignment(ArrayDataKind::ArrayValue, key_value),
                );
            }
        } else {
            artifacts.data_flow_graph.add_path(
                child_parent_node,
                &parent_node,
                PathKind::UnknownArrayAssignment(ArrayDataKind::ArrayValue),
            );
        }
    }

    parent_expr_type
}

fn update_array_assignment_child_type(
    context: &mut Context<'_>,
    block_context: &mut BlockContext,
    key_type: Option<Rc<TUnion>>,
    value_type: TUnion,
    root_type: TUnion,
) -> TUnion {
    let mut collection_types = Vec::new();

    if let Some(key_type) = &key_type {
        let key_type = if key_type.is_mixed() { Rc::new(get_arraykey()) } else { key_type.clone() };

        for original_type in &root_type.types {
            if let TAtomic::Array(array_type) = original_type {
                match array_type {
                    TArray::List(list) => {
                        collection_types.push(TAtomic::Array(TArray::List(TList {
                            element_type: Box::new(value_type.clone()),
                            known_elements: list.known_elements.clone(),
                            known_count: None,
                            non_empty: true,
                        })));
                    }
                    TArray::Keyed(keyed_array) => {
                        collection_types.push(TAtomic::Array(TArray::Keyed(TKeyedArray {
                            parameters: Some((Box::new((*key_type).clone()), Box::new(value_type.clone()))),
                            known_items: keyed_array.get_known_items().map(|known_items| {
                                known_items
                                    .iter()
                                    .map(|(k, v)| (k.clone(), (v.0, value_type.clone())))
                                    .collect::<BTreeMap<_, _>>()
                            }),
                            non_empty: true,
                        })));
                    }
                }
            }
        }
    } else {
        for original_type in &root_type.types {
            if let TAtomic::Array(array) = original_type {
                match array {
                    TArray::List(list) => {
                        if !block_context.inside_loop && list.element_type.is_never() {
                            collection_types.push(TAtomic::Array(TArray::List(TList {
                                element_type: Box::new(get_never()),
                                known_elements: Some(BTreeMap::from([(
                                    if let Some(known_elements) = list.known_elements.as_ref() {
                                        known_elements.len()
                                    } else {
                                        0
                                    },
                                    (false, value_type.clone()),
                                )])),
                                known_count: None,
                                non_empty: true,
                            })));
                        } else {
                            collection_types.push(TAtomic::Array(TArray::List(TList {
                                element_type: Box::new(value_type.clone()),
                                known_elements: None,
                                known_count: None,
                                non_empty: true,
                            })));
                        }
                    }
                    TArray::Keyed(existing_array) => {
                        let next_index = if array.is_empty() {
                            None
                        } else if existing_array.parameters.is_none() {
                            if let Some(known_items) = existing_array.known_items.as_ref() {
                                let indeces = known_items
                                    .keys()
                                    .map(|k| k.get_integer())
                                    .collect::<Option<Vec<_>>>()
                                    .unwrap_or_default();

                                if indeces.is_empty() || indeces.iter().any(|&i| i >= 0) {
                                    indeces.last().cloned()
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                        if let Some(index) = next_index.filter(|index| *index > 0) {
                            collection_types.push(TAtomic::Array(TArray::List(TList {
                                element_type: Box::new(value_type.clone()),
                                known_elements: Some(BTreeMap::from([(index as usize, (false, value_type.clone()))])),
                                known_count: None,
                                non_empty: true,
                            })));
                        } else {
                            collection_types.push(TAtomic::Array(TArray::List(TList {
                                element_type: Box::new(value_type.clone()),
                                known_elements: None,
                                known_count: None,
                                non_empty: true,
                            })));
                        }
                    }
                }
            }
        }
    }

    let collection_type = TUnion::new(combiner::combine(collection_types, context.codebase, context.interner, false));

    add_union_type(root_type, &collection_type, context.codebase, context.interner, true)
}

pub(crate) fn analyze_nested_array_assignment<'a, 's>(
    context: &mut Context<'a>,
    block_context: &mut BlockContext<'a>,
    artifacts: &mut AnalysisArtifacts,
    mut array_target_expressions: Vec<ArrayTarget<'s>>,
    assign_value_type: TUnion,
    root_var_id: Option<String>,
    root_type: &mut TUnion,
    last_array_expr_type: &mut TUnion,
) -> Result<Option<&'s Expression>, AnalysisError> {
    let mut var_id_additions = Vec::new();
    let mut last_array_expression_index = None;
    let mut extended_var_id = None;
    let mut parent_var_id: Option<String> = None;
    let mut full_var_id = true;

    array_target_expressions.reverse();
    for (i, array_target) in array_target_expressions.iter().copied().enumerate() {
        let mut array_target_index_type = None;
        let mut array_target_index_atomic_types = vec![];

        if let Some(index) = array_target.get_index() {
            let was_inside_general_use = block_context.inside_general_use;
            block_context.inside_general_use = true;
            index.analyze(context, block_context, artifacts)?;
            block_context.inside_general_use = was_inside_general_use;
            let index_type = artifacts.get_rc_expression_type(&index).cloned();

            array_target_index_type = if let Some(index_type) = index_type {
                array_target_index_atomic_types = get_index_literal_types(&index_type);

                Some(index_type)
            } else {
                Some(Rc::new(get_arraykey()))
            };

            var_id_additions.push(
                if let Some(index_expression_id) = get_index_id(
                    index,
                    block_context.scope.get_class_like_name(),
                    context.resolved_names,
                    context.interner,
                    Some(context.codebase),
                ) {
                    format!("[{index_expression_id}]")
                } else {
                    full_var_id = false;

                    "[-unknown-]".to_string()
                },
            );
        } else {
            var_id_additions.push("[-unknown-]".to_string());
            full_var_id = false;
        }

        let Some(mut array_expression_type) = artifacts.get_rc_expression_type(array_target.get_array()).cloned()
        else {
            return Ok(array_target.get_index());
        };

        if array_expression_type.is_never() && !block_context.inside_loop {
            let atomic = wrap_atomic(TAtomic::Array(TArray::Keyed(TKeyedArray {
                known_items: None,
                parameters: None,
                non_empty: false,
            })));

            array_expression_type = Rc::new(atomic);

            artifacts.set_rc_expression_type(array_target.get_array(), array_expression_type.clone());
        } else if let Some(parent_var_id) = parent_var_id.to_owned()
            && block_context.locals.contains_key(&parent_var_id)
        {
            let scoped_type = block_context.locals.get(&parent_var_id).unwrap();
            artifacts.set_rc_expression_type(array_target.get_array(), scoped_type.clone());

            array_expression_type = scoped_type.clone();
        }

        let new_index_type = array_target_index_type.clone().unwrap_or(Rc::new(get_int()));

        block_context.inside_assignment = true;

        let mut array_expr_type = get_array_target_type_given_index(
            context,
            block_context,
            artifacts,
            array_target.span(),
            array_target.get_array().span(),
            array_target.get_index().map(|index| index.span()),
            &array_expression_type,
            &new_index_type,
            true,
            &extended_var_id,
        );

        block_context.inside_assignment = false;

        let is_last = i == array_target_expressions.len() - 1;
        let mut array_expression_type_inner = (*array_expression_type).clone();

        if is_last {
            array_expr_type = assign_value_type.clone();
            artifacts.set_expression_type(&array_target, assign_value_type.clone());

            array_expression_type_inner = add_array_assignment_dataflow(
                context,
                artifacts,
                &array_target.get_array().span(),
                array_expression_type_inner,
                &assign_value_type,
                get_expression_id(
                    array_target.get_array(),
                    block_context.scope.get_class_like_name(),
                    context.resolved_names,
                    context.interner,
                    Some(context.codebase),
                ),
                &array_target_index_atomic_types,
                block_context.inside_general_use
                    || if let Some(root_var_id) = &root_var_id { root_var_id.starts_with("$_") } else { false },
            );
        } else {
            artifacts.set_expression_type(&array_target, array_expr_type.clone());
        }

        artifacts.set_expression_type(array_target.get_array(), array_expression_type_inner.clone());

        if let Some(root_var_id) = &root_var_id {
            extended_var_id = Some(root_var_id.to_owned() + &var_id_additions.join("").to_string());

            if let Some(parent_var_id) = &parent_var_id {
                if full_var_id && parent_var_id.contains("[$") {
                    block_context.locals.insert(parent_var_id.clone(), Rc::new(array_expression_type_inner.clone()));
                    block_context.possibly_assigned_variable_ids.insert(parent_var_id.clone());
                }
            } else {
                *root_type = array_expression_type_inner.clone();

                block_context.locals.insert(root_var_id.clone(), Rc::new(array_expression_type_inner.clone()));
                block_context.possibly_assigned_variable_ids.insert(root_var_id.clone());
            }
        }

        *last_array_expr_type = array_expr_type;
        last_array_expression_index = array_target.get_index();

        parent_var_id.clone_from(&extended_var_id);
    }

    array_target_expressions.reverse();

    let first_array_target = &array_target_expressions.remove(0);

    if let Some(root_var_id) = &root_var_id
        && artifacts.get_expression_type(first_array_target.get_array()).is_some()
    {
        let extended_var_id = root_var_id.clone() + var_id_additions.join("").as_str();

        if full_var_id && extended_var_id.contains("[$") {
            block_context.locals.insert(extended_var_id.clone(), Rc::new(assign_value_type.clone()));
            block_context.possibly_assigned_variable_ids.insert(extended_var_id);
        }
    }

    var_id_additions.pop();

    for (i, array_target) in array_target_expressions.iter().enumerate() {
        let mut array_expr_type = artifacts.get_expression_type(array_target).unwrap().clone();

        let index_type = if let Some(current_index) = last_array_expression_index {
            artifacts.get_rc_expression_type(current_index).cloned()
        } else {
            None
        };

        let key_values =
            if let Some(index_type) = index_type.as_ref() { get_index_literal_types(index_type) } else { vec![] };

        let mut parent_array_var_id = None;

        let array_expr_id = if let Some(var_var_id) = get_expression_id(
            array_target.get_array(),
            block_context.scope.get_class_like_name(),
            context.resolved_names,
            context.interner,
            Some(context.codebase),
        ) {
            parent_array_var_id = Some(var_var_id.clone());
            Some(format!("{}{}", var_var_id, var_id_additions.last().unwrap()))
        } else {
            None
        };

        array_expr_type = update_type_with_key_values(
            context,
            array_expr_type,
            last_array_expr_type.clone(),
            &key_values,
            index_type,
        );

        *last_array_expr_type = array_expr_type.clone();
        last_array_expression_index = array_target.get_index();

        if let Some(array_expr_id) = &array_expr_id
            && array_expr_id.contains("[$")
        {
            block_context.locals.insert(array_expr_id.clone(), Rc::new(array_expr_type.clone()));
            block_context.possibly_assigned_variable_ids.insert(array_expr_id.clone());
        }

        let array_type = artifacts.get_expression_type(array_target.get_array()).cloned().unwrap_or(get_mixed_any());

        let index_type = array_target.get_index().and_then(|index| artifacts.get_rc_expression_type(index));

        let key_values = if let Some(index_type) = index_type { get_index_literal_types(index_type) } else { vec![] };

        let array_type = add_array_assignment_dataflow(
            context,
            artifacts,
            &array_target.get_array().span(),
            array_type,
            &array_expr_type,
            parent_array_var_id,
            &key_values,
            block_context.inside_general_use,
        );

        let is_first = i == array_target_expressions.len() - 1;

        if is_first {
            *root_type = array_type;
        } else {
            artifacts.set_expression_type(array_target.get_array(), array_type);
        }

        var_id_additions.pop();
    }

    Ok(last_array_expression_index)
}

fn get_index_literal_types(expression_index_type: &TUnion) -> Vec<TAtomic> {
    let mut valid_offset_types = vec![];
    for single_atomic in &expression_index_type.types {
        if single_atomic.is_literal_int() || single_atomic.is_known_literal_string() {
            valid_offset_types.push(single_atomic.clone());
        }
    }

    valid_offset_types
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::test_analysis;

    test_analysis! {
        name = array_chunk_with_keys,
        code = indoc! {r#"
            <?php

            /**
             * @template Tv
             *
             * @param array<Tv>
             *
             * @return list<Tv>
             */
            function array_values(array $array): array {
                return array_values($array);
            }

            /**
             * @template Tk as array-key
             * @template Tv
             *
             * @param iterable<Tk, Tv> $iterable The iterable to chunk
             * @param positive-int $size The size of each chunk
             *
             * @return list<array<Tk, Tv>>
             */
            function chunk_with_keys(iterable $iterable, int $size): array
            {
                $result = [];
                $ii = 0;
                $chunk_number = -1;
                foreach ($iterable as $k => $value) {
                    if (($ii % $size) === 0) {
                        $chunk_number++;
                        $result[$chunk_number] = [];
                    }

                    $result[$chunk_number][$k] = $value;
                    $ii++;
                }

                return array_values($result);
            }
        "#}
    }

    test_analysis! {
        name = array_chunk,
        code = indoc! {r#"
            <?php

            /**
             * @template Tv
             *
             * @param array<Tv>
             *
             * @return list<Tv>
             */
            function array_values(array $array): array {
                return array_values($array);
            }

            /**
             * @template T
             *
             * @param iterable<T> $iterable
             * @param positive-int $size
             *
             * @return list<list<T>>
             */
            function chunk(iterable $iterable, int $size): array
            {
                $result = [];
                $ii = 0;
                $chunk_number = -1;
                foreach ($iterable as $value) {
                    if (($ii % $size) === 0) {
                        $result[] = [];
                        $chunk_number++;
                    }

                    $result[$chunk_number][] = $value;
                    $ii++;
                }

                return array_values($result);
            }
        "#}
    }
}
