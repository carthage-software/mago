use std::collections::BTreeMap;

use ahash::HashSet;

use mago_syntax::ast::*;
use mago_syntax::walker::MutWalker;

use crate::utils::expression::get_root_expression_id;

pub fn get_assignment_map<'ast, 'arena>(
    pre_conditions: &Vec<&'ast Expression<'arena>>,
    post_expressions: &Vec<&'ast Expression<'arena>>,
    statements: &'ast [Statement<'arena>],
) -> (BTreeMap<String, HashSet<String>>, Option<String>) {
    let mut walker = AssignmentMapWalker::default();

    for pre_condition in pre_conditions {
        walker.walk_expression(pre_condition, &mut ());
    }

    for statement in statements {
        walker.walk_statement(statement, &mut ());
    }

    for post_expression in post_expressions {
        walker.walk_expression(post_expression, &mut ());
    }

    let first_variable_id = walker.assignment_map.first_key_value().map(|(key, _)| key.clone());

    (walker.assignment_map, first_variable_id)
}

#[derive(Debug, Clone, Default)]
struct AssignmentMapWalker {
    assignment_map: BTreeMap<String, HashSet<String>>,
}

impl<'ast, 'arena> MutWalker<'ast, 'arena, ()> for AssignmentMapWalker {
    fn walk_unary_postfix(&mut self, unary_postfix: &'ast UnaryPostfix<'arena>, _context: &mut ()) {
        let root_expression_id = get_root_expression_id(unary_postfix.operand);

        if let Some(root_expression_id) = root_expression_id {
            self.assignment_map.entry(root_expression_id.clone()).or_default().insert(root_expression_id);
        }
    }

    fn walk_unary_prefix(&mut self, unary_prefix: &'ast UnaryPrefix<'arena>, context: &mut ()) {
        if unary_prefix.operator.is_increment_or_decrement() {
            let root_expression_id = get_root_expression_id(unary_prefix.operand);

            if let Some(root_expression_id) = root_expression_id {
                self.assignment_map.entry(root_expression_id.clone()).or_default().insert(root_expression_id);
            }
        } else {
            self.walk_expression(unary_prefix.operand, context);
        }
    }

    fn walk_assignment(&mut self, assignment: &'ast Assignment<'arena>, _context: &mut ()) {
        let right_expression_id = get_root_expression_id(assignment.rhs).unwrap_or_else(|| "isset".to_string());

        if let Some(array_elements) = assignment.lhs.get_array_like_elements() {
            for array_element in array_elements {
                if let Some(expression) = array_element.get_value() {
                    let left_expression_id = get_root_expression_id(expression);

                    if let Some(left_expression_id) = &left_expression_id {
                        self.assignment_map
                            .entry(left_expression_id.clone())
                            .or_default()
                            .insert(right_expression_id.clone());
                    }
                }
            }
        } else {
            let left_expression_id = get_root_expression_id(assignment.lhs);

            if let Some(left_expression_id) = &left_expression_id {
                self.assignment_map.entry(left_expression_id.clone()).or_default().insert(right_expression_id);
            }
        }
    }

    fn walk_in_argument_list(&mut self, argument_list: &'ast ArgumentList<'arena>, _context: &mut ()) {
        for argument in argument_list.arguments.iter() {
            let root_expression_id = get_root_expression_id(argument.value());

            if let Some(root_expression_id) = &root_expression_id {
                self.assignment_map.entry(root_expression_id.clone()).or_default().insert(root_expression_id.clone());
            }
        }
    }

    fn walk_out_method_call(&mut self, method_call: &'ast MethodCall<'arena>, _context: &mut ()) {
        let root_expression_id = get_root_expression_id(method_call.object);

        if let Some(root_expression_id) = &root_expression_id {
            self.assignment_map.entry(root_expression_id.clone()).or_default().insert("isset".to_string());
        }
    }

    fn walk_out_method_closure_creation(
        &mut self,
        method_closure_creation: &'ast MethodClosureCreation<'arena>,
        _context: &mut (),
    ) {
        let root_expression_id = get_root_expression_id(method_closure_creation.object);

        if let Some(root_expression_id) = &root_expression_id {
            self.assignment_map.entry(root_expression_id.clone()).or_default().insert("isset".to_string());
        }
    }

    fn walk_in_unset(&mut self, unset: &'ast Unset<'arena>, _context: &mut ()) {
        for unset_value in unset.values.iter() {
            let root_expression_id = get_root_expression_id(unset_value);

            if let Some(root_expression_id) = &root_expression_id {
                self.assignment_map.entry(root_expression_id.clone()).or_default().insert(root_expression_id.clone());
            }
        }
    }

    // Prevent walking into closure and arrow function bodies
    fn walk_closure(&mut self, _closure: &'ast Closure<'arena>, _context: &mut ()) {}
    fn walk_arrow_function(&mut self, _arrow_function: &'ast ArrowFunction<'arena>, _context: &mut ()) {}
}
