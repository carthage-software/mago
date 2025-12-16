use std::collections::BTreeMap;
use std::collections::BTreeSet;

use mago_atom::Atom;
use mago_syntax::ast::ArgumentList;
use mago_syntax::ast::ArrowFunction;
use mago_syntax::ast::Assignment;
use mago_syntax::ast::Closure;
use mago_syntax::ast::Expression;
use mago_syntax::ast::MethodCall;
use mago_syntax::ast::MethodPartialApplication;
use mago_syntax::ast::Statement;
use mago_syntax::ast::UnaryPostfix;
use mago_syntax::ast::UnaryPrefix;
use mago_syntax::ast::Unset;
use mago_syntax::walker::MutWalker;

use crate::utils::expression::get_root_expression_id;

pub fn get_assignment_map<'ast, 'arena>(
    pre_conditions: &[&'ast Expression<'arena>],
    post_expressions: &[&'ast Expression<'arena>],
    statements: &'ast [Statement<'arena>],
) -> (BTreeMap<Atom, BTreeSet<Atom>>, Option<Atom>) {
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

    let first_variable_id = walker.assignment_map.first_key_value().map(|(key, _)| *key);

    (walker.assignment_map, first_variable_id)
}

#[derive(Debug, Clone, Default)]
struct AssignmentMapWalker {
    assignment_map: BTreeMap<Atom, BTreeSet<Atom>>,
}

impl<'ast, 'arena> MutWalker<'ast, 'arena, ()> for AssignmentMapWalker {
    fn walk_unary_postfix(&mut self, unary_postfix: &'ast UnaryPostfix<'arena>, _context: &mut ()) {
        let root_expression_id = get_root_expression_id(unary_postfix.operand);

        if let Some(root_expression_id) = root_expression_id {
            self.assignment_map.entry(root_expression_id).or_default().insert(root_expression_id);
        }
    }

    fn walk_unary_prefix(&mut self, unary_prefix: &'ast UnaryPrefix<'arena>, context: &mut ()) {
        if unary_prefix.operator.is_increment_or_decrement() {
            let root_expression_id = get_root_expression_id(unary_prefix.operand);

            if let Some(root_expression_id) = root_expression_id {
                self.assignment_map.entry(root_expression_id).or_default().insert(root_expression_id);
            }
        } else {
            self.walk_expression(unary_prefix.operand, context);
        }
    }

    fn walk_assignment(&mut self, assignment: &'ast Assignment<'arena>, _context: &mut ()) {
        let right_expression_id = get_root_expression_id(assignment.rhs).unwrap_or_else(|| Atom::from("isset"));

        if let Some(array_elements) = assignment.lhs.get_array_like_elements() {
            for array_element in array_elements {
                if let Some(expression) = array_element.get_value() {
                    let left_expression_id = get_root_expression_id(expression);

                    if let Some(left_expression_id) = left_expression_id {
                        self.assignment_map.entry(left_expression_id).or_default().insert(right_expression_id);
                    }
                }
            }
        } else {
            let left_expression_id = get_root_expression_id(assignment.lhs);

            if let Some(left_expression_id) = left_expression_id {
                self.assignment_map.entry(left_expression_id).or_default().insert(right_expression_id);
            }
        }
    }

    fn walk_in_argument_list(&mut self, argument_list: &'ast ArgumentList<'arena>, _context: &mut ()) {
        for argument in &argument_list.arguments {
            let root_expression_id = get_root_expression_id(argument.value());

            if let Some(root_expression_id) = root_expression_id {
                self.assignment_map.entry(root_expression_id).or_default().insert(root_expression_id);
            }
        }
    }

    fn walk_out_method_call(&mut self, method_call: &'ast MethodCall<'arena>, _context: &mut ()) {
        let root_expression_id = get_root_expression_id(method_call.object);

        if let Some(root_expression_id) = root_expression_id {
            self.assignment_map.entry(root_expression_id).or_default().insert(Atom::from("isset"));
        }
    }

    fn walk_out_method_partial_application(
        &mut self,
        method_partial_application: &'ast MethodPartialApplication<'arena>,
        _context: &mut (),
    ) {
        let root_expression_id = get_root_expression_id(method_partial_application.object);

        if let Some(root_expression_id) = root_expression_id {
            self.assignment_map.entry(root_expression_id).or_default().insert(Atom::from("isset"));
        }
    }

    fn walk_in_unset(&mut self, unset: &'ast Unset<'arena>, _context: &mut ()) {
        for unset_value in &unset.values {
            let root_expression_id = get_root_expression_id(unset_value);

            if let Some(root_expression_id) = root_expression_id {
                self.assignment_map.entry(root_expression_id).or_default().insert(root_expression_id);
            }
        }
    }

    // Prevent walking into closure and arrow function bodies
    fn walk_closure(&mut self, _closure: &'ast Closure<'arena>, _context: &mut ()) {}
    fn walk_arrow_function(&mut self, _arrow_function: &'ast ArrowFunction<'arena>, _context: &mut ()) {}
}
