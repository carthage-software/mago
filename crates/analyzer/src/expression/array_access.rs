use std::rc::Rc;

use ahash::HashMap;
use mago_codex::data_flow::graph::DataFlowGraph;
use mago_codex::data_flow::node::DataFlowNode;
use mago_codex::data_flow::path::ArrayDataKind;
use mago_codex::data_flow::path::PathKind;
use mago_codex::ttype::get_arraykey;
use mago_codex::ttype::get_mixed_any;
use mago_codex::ttype::union::TUnion;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::ArrayAccess;
use mago_syntax::ast::Expression;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::artifacts::get_expression_range;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::utils::expression::array::get_array_target_type_given_index;
use crate::utils::expression::get_array_access_id;
use crate::utils::expression::get_expression_id;

impl Analyzable for ArrayAccess {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let keyed_array_var_id = get_array_access_id(
            self,
            block_context.scope.get_class_like_name(),
            context.resolved_names,
            context.interner,
            Some(context.codebase),
        );

        let extended_var_id = get_expression_id(
            &self.array,
            block_context.scope.get_class_like_name(),
            context.resolved_names,
            context.interner,
            Some(context.codebase),
        );

        let was_inside_use = block_context.inside_general_use;
        block_context.inside_general_use = true;
        block_context.inside_unset = false;
        self.index.analyze(context, block_context, artifacts)?;
        block_context.inside_general_use = was_inside_use;

        let mut index_type = artifacts.get_expression_type(&self.index).cloned().unwrap_or_else(get_arraykey);

        let was_inside_general_use = block_context.inside_general_use;
        block_context.inside_general_use = true;
        self.array.analyze(context, block_context, artifacts)?;
        block_context.inside_general_use = was_inside_general_use;

        if let Some(keyed_array_var_id) = &keyed_array_var_id
            && block_context.has_variable(keyed_array_var_id)
        {
            let mut array_access_type = block_context.locals.remove(keyed_array_var_id).unwrap();

            add_array_access_dataflow_rc(
                &artifacts.expression_types,
                &mut artifacts.data_flow_graph,
                &self.array,
                Some(keyed_array_var_id.clone()),
                &mut array_access_type,
                &mut index_type,
            );

            artifacts.set_rc_expression_type(self, array_access_type.clone());

            block_context.locals.insert(keyed_array_var_id.clone(), array_access_type.clone());

            return Ok(());
        }

        let container_type = artifacts.get_rc_expression_type(&self.array).cloned();

        if let Some(container_type) = container_type {
            let mut access_type = get_array_target_type_given_index(
                context,
                block_context,
                artifacts,
                self.span(),
                self.array.span(),
                Some(self.index.span()),
                &container_type,
                &index_type,
                false,
                &extended_var_id,
            );

            if let Some(keyed_array_var_id) = &keyed_array_var_id {
                let can_store_result = block_context.inside_assignment || !container_type.is_mixed();

                if !block_context.inside_isset && can_store_result && keyed_array_var_id.contains("[$") {
                    block_context.locals.insert(keyed_array_var_id.clone(), Rc::new(access_type.clone()));
                }
            }

            add_array_access_dataflow(
                &artifacts.expression_types,
                &mut artifacts.data_flow_graph,
                self.array.span(),
                keyed_array_var_id.clone(),
                &mut access_type,
                &mut index_type,
            );

            artifacts.set_expression_type(self, access_type.clone());
        } else {
            artifacts.set_expression_type(self, get_mixed_any());
        }

        Ok(())
    }
}

/**
 * Used to create a path between a variable $foo and $foo["a"]
 */
pub(crate) fn add_array_access_dataflow_rc(
    expression_types: &HashMap<(usize, usize), Rc<TUnion>>,
    data_flow_graph: &mut DataFlowGraph,
    array_expr: &Expression,
    keyed_array_var_id: Option<String>,
    value_type: &mut Rc<TUnion>,
    key_type: &mut TUnion,
) {
    let value_type_inner = Rc::make_mut(value_type);

    add_array_access_dataflow(
        expression_types,
        data_flow_graph,
        array_expr.span(),
        keyed_array_var_id,
        value_type_inner,
        key_type,
    );
}

pub(crate) fn add_array_access_dataflow(
    expression_types: &HashMap<(usize, usize), Rc<TUnion>>,
    data_flow_graph: &mut DataFlowGraph,
    array_expr_pos: Span,
    keyed_array_var_id: Option<String>,
    value_type: &mut TUnion,
    key_type: &mut TUnion,
) {
    if let Some(stmt_var_type) = expression_types.get(&get_expression_range(&array_expr_pos))
        && !stmt_var_type.parent_nodes.is_empty()
    {
        let node_name = if let Some(keyed_array_var_id) = &keyed_array_var_id {
            keyed_array_var_id.clone()
        } else {
            "arrayvalue-access".to_string()
        };
        let new_parent_node = DataFlowNode::get_for_local_string(node_name, array_expr_pos);
        data_flow_graph.add_node(new_parent_node.clone());

        let key_type_single = if key_type.is_single() { Some(key_type.get_single()) } else { None };

        let dim_value = if let Some(key_type_single) = key_type_single {
            if let Some(v) = key_type_single.get_literal_string_value() {
                Some(v.to_string())
            } else {
                key_type_single.get_literal_int_value().map(|v| v.to_string())
            }
        } else {
            None
        };

        let mut array_key_node = None;

        if keyed_array_var_id.is_none() && dim_value.is_none() {
            let access_node = DataFlowNode::get_for_local_string("arraykey-access".to_string(), array_expr_pos);
            data_flow_graph.add_node(access_node.clone());
            array_key_node = Some(access_node);
            data_flow_graph.add_node(new_parent_node.clone());
        }

        for parent_node in stmt_var_type.parent_nodes.iter() {
            data_flow_graph.add_path(
                parent_node,
                &new_parent_node,
                if let Some(dim_value) = dim_value.clone() {
                    PathKind::ArrayAccess(ArrayDataKind::ArrayValue, dim_value.to_string())
                } else {
                    PathKind::UnknownArrayAccess(ArrayDataKind::ArrayValue)
                },
            );

            if let Some(array_key_node) = array_key_node.clone() {
                data_flow_graph.add_path(
                    parent_node,
                    &array_key_node,
                    PathKind::UnknownArrayAccess(ArrayDataKind::ArrayKey),
                );
            }
        }

        value_type.parent_nodes.push(new_parent_node.clone());

        if let Some(array_key_node) = &array_key_node {
            key_type.parent_nodes.push(array_key_node.clone());
        }
    }
}
