use crate::ast::*;
use crate::utils::control_flow::ControlFlow;

pub mod assignment;
pub mod condition;
pub mod control_flow;
pub mod definition;
pub mod reference;

#[inline]
pub fn find_returns_in_block<'ast, 'arena>(block: &'ast Block<'arena>) -> Vec<&'ast Return<'arena>> {
    let mut returns = vec![];
    for control_flow in control_flow::find_control_flows_in_block(block) {
        if let ControlFlow::Return(r#return) = control_flow {
            returns.push(r#return);
        }
    }

    returns
}

#[inline]
pub fn find_returns_in_statement<'ast, 'arena>(statement: &'ast Statement<'arena>) -> Vec<&'ast Return<'arena>> {
    let mut returns = vec![];
    for control_flow in control_flow::find_control_flows_in_statement(statement) {
        if let ControlFlow::Return(r#return) = control_flow {
            returns.push(r#return);
        }
    }

    returns
}

#[inline]
pub fn block_has_throws<'ast, 'arena>(block: &'ast Block<'arena>) -> bool {
    for control_flow in control_flow::find_control_flows_in_block(block) {
        if let ControlFlow::Throw(_) = control_flow {
            return true;
        }
    }

    false
}

#[inline]
pub fn statement_has_throws<'ast, 'arena>(statement: &'ast Statement<'arena>) -> bool {
    for control_flow in control_flow::find_control_flows_in_statement(statement) {
        if let ControlFlow::Throw(_) = control_flow {
            return true;
        }
    }

    false
}

#[inline]
pub fn expression_has_throws<'ast, 'arena>(expression: &'ast Expression<'arena>) -> bool {
    for control_flow in control_flow::find_control_flows_in_expression(expression) {
        if let ControlFlow::Throw(_) = control_flow {
            return true;
        }
    }

    false
}

#[inline]
pub fn block_has_yield(block: &Block) -> bool {
    for statement in block.statements.iter() {
        if statement_has_yield(statement) {
            return true;
        }
    }

    false
}

#[inline]
pub fn statement_has_yield(statement: &Statement) -> bool {
    match statement {
        Statement::Namespace(namespace) => {
            for statement in namespace.statements().iter() {
                if statement_has_yield(statement) {
                    return true;
                }
            }

            false
        }
        Statement::Block(block) => block_has_yield(block),
        Statement::Try(r#try) => {
            if r#try.block.statements.iter().any(statement_has_yield)
                || r#try.catch_clauses.iter().any(|catch| block_has_yield(&catch.block))
            {
                return true;
            }

            for catch in r#try.catch_clauses.iter() {
                if block_has_yield(&catch.block) {
                    return true;
                }
            }

            if let Some(finally) = &r#try.finally_clause
                && block_has_yield(&finally.block)
            {
                return true;
            }

            false
        }
        Statement::Foreach(foreach) => match &foreach.body {
            ForeachBody::Statement(statement) => statement_has_yield(statement),
            ForeachBody::ColonDelimited(foreach_colon_delimited_body) => {
                foreach_colon_delimited_body.statements.iter().any(statement_has_yield)
            }
        },
        Statement::For(r#for) => match &r#for.body {
            ForBody::Statement(statement) => statement_has_yield(statement),
            ForBody::ColonDelimited(foreach_colon_delimited_body) => {
                foreach_colon_delimited_body.statements.iter().any(statement_has_yield)
            }
        },
        Statement::While(r#while) => match &r#while.body {
            WhileBody::Statement(statement) => statement_has_yield(statement),
            WhileBody::ColonDelimited(foreach_colon_delimited_body) => {
                foreach_colon_delimited_body.statements.iter().any(statement_has_yield)
            }
        },
        Statement::DoWhile(do_while) => statement_has_yield(do_while.statement),
        Statement::Switch(switch) => {
            let cases = match &switch.body {
                SwitchBody::BraceDelimited(switch_brace_delimited_body) => &switch_brace_delimited_body.cases,
                SwitchBody::ColonDelimited(switch_colon_delimited_body) => &switch_colon_delimited_body.cases,
            };

            for case in cases.iter() {
                match &case {
                    SwitchCase::Expression(switch_expression_case) => {
                        for statement in switch_expression_case.statements.iter() {
                            if statement_has_yield(statement) {
                                return true;
                            }
                        }
                    }
                    SwitchCase::Default(switch_default_case) => {
                        for statement in switch_default_case.statements.iter() {
                            if statement_has_yield(statement) {
                                return true;
                            }
                        }
                    }
                }
            }

            false
        }
        Statement::If(r#if) => match &r#if.body {
            IfBody::Statement(if_statement_body) => {
                if statement_has_yield(if_statement_body.statement) {
                    return true;
                }

                for else_if in if_statement_body.else_if_clauses.iter() {
                    if statement_has_yield(else_if.statement) {
                        return true;
                    }
                }

                if let Some(else_clause) = &if_statement_body.else_clause
                    && statement_has_yield(else_clause.statement)
                {
                    return true;
                }

                false
            }
            IfBody::ColonDelimited(if_colon_delimited_body) => {
                if if_colon_delimited_body.statements.iter().any(statement_has_yield) {
                    return true;
                }

                for else_if in if_colon_delimited_body.else_if_clauses.iter() {
                    if else_if.statements.iter().any(statement_has_yield) {
                        return true;
                    }
                }

                if let Some(else_clause) = &if_colon_delimited_body.else_clause
                    && else_clause.statements.iter().any(statement_has_yield)
                {
                    return true;
                }

                false
            }
        },
        Statement::Expression(expression) => expression_has_yield(expression.expression),
        _ => false,
    }
}

#[inline]
pub fn expression_has_yield(expression: &Expression) -> bool {
    match &expression {
        Expression::Parenthesized(parenthesized) => expression_has_yield(parenthesized.expression),
        Expression::Literal(_) => false,
        Expression::CompositeString(_) => false,
        Expression::Binary(operation) => expression_has_yield(operation.lhs) || expression_has_yield(operation.rhs),
        Expression::UnaryPrefix(operation) => expression_has_yield(operation.operand),
        Expression::UnaryPostfix(operation) => expression_has_yield(operation.operand),
        Expression::Assignment(assignment_operation) => {
            expression_has_yield(assignment_operation.lhs) || expression_has_yield(assignment_operation.rhs)
        }
        Expression::Conditional(conditional) => {
            expression_has_yield(conditional.condition)
                || conditional.then.as_ref().map(|e| expression_has_yield(e)).unwrap_or(false)
                || expression_has_yield(conditional.r#else)
        }
        Expression::Array(array) => array.elements.iter().any(|element| match element {
            ArrayElement::KeyValue(key_value_array_element) => {
                expression_has_yield(key_value_array_element.key) || expression_has_yield(key_value_array_element.value)
            }
            ArrayElement::Value(value_array_element) => expression_has_yield(value_array_element.value),
            ArrayElement::Variadic(variadic_array_element) => expression_has_yield(variadic_array_element.value),
            _ => false,
        }),
        Expression::LegacyArray(legacy_array) => legacy_array.elements.iter().any(|element| match element {
            ArrayElement::KeyValue(key_value_array_element) => {
                expression_has_yield(key_value_array_element.key) || expression_has_yield(key_value_array_element.value)
            }
            ArrayElement::Value(value_array_element) => expression_has_yield(value_array_element.value),
            ArrayElement::Variadic(variadic_array_element) => expression_has_yield(variadic_array_element.value),
            _ => false,
        }),
        Expression::List(list) => list.elements.iter().any(|element| match element {
            ArrayElement::KeyValue(key_value_array_element) => {
                expression_has_yield(key_value_array_element.key) || expression_has_yield(key_value_array_element.value)
            }
            ArrayElement::Value(value_array_element) => expression_has_yield(value_array_element.value),
            ArrayElement::Variadic(variadic_array_element) => expression_has_yield(variadic_array_element.value),
            _ => false,
        }),
        Expression::ArrayAccess(array_access) => {
            expression_has_yield(array_access.array) || expression_has_yield(array_access.index)
        }
        Expression::ArrayAppend(array_append) => expression_has_yield(array_append.array),
        Expression::Match(r#match) => {
            expression_has_yield(r#match.expression)
                || r#match.arms.iter().any(|arm| match arm {
                    MatchArm::Expression(match_expression_arm) => {
                        match_expression_arm.conditions.iter().any(expression_has_yield)
                            || expression_has_yield(match_expression_arm.expression)
                    }
                    MatchArm::Default(match_default_arm) => expression_has_yield(match_default_arm.expression),
                })
        }
        Expression::Construct(construct) => match construct {
            Construct::Isset(isset_construct) => isset_construct.values.iter().any(expression_has_yield),
            Construct::Empty(empty_construct) => expression_has_yield(empty_construct.value),
            Construct::Eval(eval_construct) => expression_has_yield(eval_construct.value),
            Construct::Include(include_construct) => expression_has_yield(include_construct.value),
            Construct::IncludeOnce(include_once_construct) => expression_has_yield(include_once_construct.value),
            Construct::Require(require_construct) => expression_has_yield(require_construct.value),
            Construct::RequireOnce(require_once_construct) => expression_has_yield(require_once_construct.value),
            Construct::Print(print_construct) => expression_has_yield(print_construct.value),
            Construct::Exit(exit_construct) => exit_construct
                .arguments
                .as_ref()
                .map(|arguments| {
                    arguments.arguments.iter().any(|argument| match argument {
                        Argument::Positional(positional_argument) => expression_has_yield(&positional_argument.value),
                        Argument::Named(named_argument) => expression_has_yield(&named_argument.value),
                    })
                })
                .unwrap_or(false),
            Construct::Die(die_construct) => die_construct
                .arguments
                .as_ref()
                .map(|arguments| {
                    arguments.arguments.iter().any(|argument| match argument {
                        Argument::Positional(positional_argument) => expression_has_yield(&positional_argument.value),
                        Argument::Named(named_argument) => expression_has_yield(&named_argument.value),
                    })
                })
                .unwrap_or(false),
        },
        Expression::Throw(throw) => expression_has_yield(throw.exception),
        Expression::Clone(clone) => expression_has_yield(clone.object),
        Expression::Call(call) => match call {
            Call::Function(function_call) => {
                expression_has_yield(function_call.function)
                    || function_call.argument_list.arguments.iter().any(|argument| match argument {
                        Argument::Positional(positional_argument) => expression_has_yield(&positional_argument.value),
                        Argument::Named(named_argument) => expression_has_yield(&named_argument.value),
                    })
            }
            Call::Method(method_call) => {
                expression_has_yield(method_call.object)
                    || matches!(&method_call.method, ClassLikeMemberSelector::Expression(selector) if expression_has_yield(selector.expression))
                    || method_call.argument_list.arguments.iter().any(|argument| match argument {
                        Argument::Positional(positional_argument) => expression_has_yield(&positional_argument.value),
                        Argument::Named(named_argument) => expression_has_yield(&named_argument.value),
                    })
            }
            Call::NullSafeMethod(null_safe_method_call) => {
                expression_has_yield(null_safe_method_call.object)
                    || matches!(&null_safe_method_call.method, ClassLikeMemberSelector::Expression(selector) if expression_has_yield(selector.expression))
                    || null_safe_method_call.argument_list.arguments.iter().any(|argument| match argument {
                        Argument::Positional(positional_argument) => expression_has_yield(&positional_argument.value),
                        Argument::Named(named_argument) => expression_has_yield(&named_argument.value),
                    })
            }
            Call::StaticMethod(static_method_call) => {
                expression_has_yield(static_method_call.class)
                    || matches!(&static_method_call.method, ClassLikeMemberSelector::Expression(selector) if expression_has_yield(selector.expression))
                    || static_method_call.argument_list.arguments.iter().any(|argument| match argument {
                        Argument::Positional(positional_argument) => expression_has_yield(&positional_argument.value),
                        Argument::Named(named_argument) => expression_has_yield(&named_argument.value),
                    })
            }
        },
        Expression::Access(access) => match access {
            Access::Property(property_access) => {
                expression_has_yield(property_access.object)
                    || matches!(&property_access.property, ClassLikeMemberSelector::Expression(selector) if expression_has_yield(selector.expression))
            }
            Access::NullSafeProperty(null_safe_property_access) => {
                expression_has_yield(null_safe_property_access.object)
                    || matches!(&null_safe_property_access.property, ClassLikeMemberSelector::Expression(selector) if expression_has_yield(selector.expression))
            }
            Access::StaticProperty(static_property_access) => expression_has_yield(static_property_access.class),
            Access::ClassConstant(class_constant_access) => {
                expression_has_yield(class_constant_access.class)
                    || matches!(&class_constant_access.constant, ClassLikeConstantSelector::Expression(selector) if expression_has_yield(selector.expression))
            }
        },
        Expression::ClosureCreation(closure_creation) => match closure_creation {
            ClosureCreation::Function(function_closure_creation) => {
                expression_has_yield(function_closure_creation.function)
            }
            ClosureCreation::Method(method_closure_creation) => {
                expression_has_yield(method_closure_creation.object)
                    || matches!(&method_closure_creation.method, ClassLikeMemberSelector::Expression(selector) if expression_has_yield(selector.expression))
            }
            ClosureCreation::StaticMethod(static_method_closure_creation) => {
                expression_has_yield(static_method_closure_creation.class)
                    || matches!(&static_method_closure_creation.method, ClassLikeMemberSelector::Expression(selector) if expression_has_yield(selector.expression))
            }
        },
        Expression::Instantiation(instantiation) => {
            expression_has_yield(instantiation.class)
                || instantiation
                    .argument_list
                    .as_ref()
                    .map(|arguments| {
                        arguments.arguments.iter().any(|argument| match argument {
                            Argument::Positional(positional_argument) => {
                                expression_has_yield(&positional_argument.value)
                            }
                            Argument::Named(named_argument) => expression_has_yield(&named_argument.value),
                        })
                    })
                    .unwrap_or(false)
        }
        Expression::Yield(_) => true,
        _ => false,
    }
}
